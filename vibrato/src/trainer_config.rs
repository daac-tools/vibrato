use std::io::{BufRead, BufReader, Read};

use crate::dictionary::character::CharProperty;
use crate::errors::{Result, VibratoError};
use crate::feature_extractor::FeatureExtractor;
use crate::feature_rewriter::{FeatureRewriter, FeatureRewriterBuilder};

/// Configuration for a trainer.
#[allow(unused)]
pub struct TrainerConfig {
    feature_extractor: FeatureExtractor,
    unigram_rewriter: FeatureRewriter,
    left_rewriter: FeatureRewriter,
    right_rewriter: FeatureRewriter,
    char_property: CharProperty,
}

impl TrainerConfig {
    fn parse_feature_config<R>(rdr: R) -> Result<FeatureExtractor>
    where
        R: Read,
    {
        let reader = BufReader::new(rdr);

        let mut unigram_templates = vec![];
        let mut bigram_templates = vec![];

        for line in reader.lines() {
            let line = line?;
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(template) = line.strip_prefix("UNIGRAM ") {
                unigram_templates.push(template.to_string());
            } else if let Some(template) = line.strip_prefix("BIGRAM ") {
                let mut spl = template.split('/');
                let left = spl.next();
                let right = spl.next();
                let rest = spl.next();
                if let (Some(left), Some(right), None) = (left, right, rest) {
                    bigram_templates.push((left.to_string(), right.to_string()));
                } else {
                    return Err(VibratoError::invalid_format(
                        "feature.def",
                        "Invalid bigram template",
                    ));
                }
            } else {
                return Err(VibratoError::invalid_format("feature", ""));
            }
        }

        Ok(FeatureExtractor::new(&unigram_templates, &bigram_templates))
    }

    fn parse_rewrite_rule(line: &str) -> Result<(Vec<&str>, Vec<&str>)> {
        let mut spl = line.split_ascii_whitespace();
        let pattern = spl.next();
        let rewrite = spl.next();
        let rest = spl.next();
        if let (Some(pattern), Some(rewrite), None) = (pattern, rewrite, rest) {
            Ok((pattern.split(',').collect(), rewrite.split(',').collect()))
        } else {
            Err(VibratoError::invalid_format(
                "rewrite.def",
                "invalid rewrite rule",
            ))
        }
    }

    fn parse_rewrite_config<R>(
        rdr: R,
    ) -> Result<(FeatureRewriter, FeatureRewriter, FeatureRewriter)>
    where
        R: Read,
    {
        let reader = BufReader::new(rdr);

        let mut unigram_rewriter_builder = FeatureRewriterBuilder::new();
        let mut left_rewriter_builder = FeatureRewriterBuilder::new();
        let mut right_rewriter_builder = FeatureRewriterBuilder::new();

        let mut builder = None;
        for line in reader.lines() {
            let line = line?;
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            match line {
                "[unigram rewrite]" => builder = Some(&mut unigram_rewriter_builder),
                "[left rewrite]" => builder = Some(&mut left_rewriter_builder),
                "[right rewrite]" => builder = Some(&mut right_rewriter_builder),
                line => {
                    if let Some(builder) = builder.as_mut() {
                        let (pattern, rewrite) = Self::parse_rewrite_rule(line)?;
                        builder.add_rule(&pattern, &rewrite)?;
                    } else {
                        return Err(VibratoError::invalid_format(
                            "rewrite.def",
                            "Invalid rewrite rule",
                        ));
                    }
                }
            }
        }

        Ok((
            FeatureRewriter::from(unigram_rewriter_builder),
            FeatureRewriter::from(left_rewriter_builder),
            FeatureRewriter::from(right_rewriter_builder),
        ))
    }

    /// Loads a training configuration from readers.
    ///
    /// # Arguments
    ///
    /// * `feature_templates_rdr` - A reader of the feature definition file `feature.def`.
    /// * `rewrite_rules_rdr` - A reader of the rewrite definition file `rewrite.def`.
    /// * `char_prop_rdr` - A reader of the character definition file `char.def`.
    ///
    /// # Errors
    ///
    /// An error variant returns if the file format is invalid.
    #[allow(unused)]
    pub fn from_readers<F, R, C>(
        feature_templates_rdr: F,
        rewrite_rules_rdr: R,
        char_prop_rdr: C,
    ) -> Result<Self>
    where
        F: Read,
        R: Read,
        C: Read,
    {
        let feature_extractor = Self::parse_feature_config(feature_templates_rdr)?;
        let (unigram_rewriter, left_rewriter, right_rewriter) =
            Self::parse_rewrite_config(rewrite_rules_rdr)?;
        let char_property = CharProperty::from_reader(char_prop_rdr)?;

        Ok(Self {
            feature_extractor,
            unigram_rewriter,
            left_rewriter,
            right_rewriter,
            char_property,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_feature_config() {
        let config = "
            # feature 1
            UNIGRAM uni:%F[0]
            BIGRAM bi:%L[0]/%R[1]

            # feature 2
            UNIGRAM uni:%F[0]/%t
            BIGRAM bi:%L[0],%L[1]/%R[1],%R[0]
        ";
        let mut feature_extractor = TrainerConfig::parse_feature_config(config.as_bytes()).unwrap();

        // unigram features
        assert_eq!(
            vec![0, 1],
            feature_extractor.extract_unigram_feature_ids(&["a", "b"], "c")
        );
        assert_eq!(
            vec![2, 3],
            feature_extractor.extract_unigram_feature_ids(&["b", "c"], "c")
        );
        assert_eq!(
            vec![0, 1],
            feature_extractor.extract_unigram_feature_ids(&["a", "c"], "c")
        );
        assert_eq!(
            vec![2, 4],
            feature_extractor.extract_unigram_feature_ids(&["b", "c"], "d")
        );

        // left features
        assert_eq!(
            vec![Some(0), Some(1)],
            feature_extractor.extract_left_feature_ids(&["a", "b"])
        );
        assert_eq!(
            vec![Some(2), Some(3)],
            feature_extractor.extract_left_feature_ids(&["b", "c"])
        );
        assert_eq!(
            vec![Some(0), Some(4)],
            feature_extractor.extract_left_feature_ids(&["a", "c"])
        );
        assert_eq!(
            vec![Some(2), Some(3)],
            feature_extractor.extract_left_feature_ids(&["b", "c"])
        );

        // right features
        assert_eq!(
            vec![Some(0), Some(1)],
            feature_extractor.extract_right_feature_ids(&["a", "b"])
        );
        assert_eq!(
            vec![Some(2), Some(3)],
            feature_extractor.extract_right_feature_ids(&["b", "c"])
        );
        assert_eq!(
            vec![Some(2), Some(4)],
            feature_extractor.extract_right_feature_ids(&["a", "c"])
        );
        assert_eq!(
            vec![Some(2), Some(3)],
            feature_extractor.extract_right_feature_ids(&["b", "c"])
        );
    }

    #[test]
    fn test_parse_rewrite_config() {
        let config = "
            # unigram feature
            [unigram rewrite]
            a,*,*  $1,$2,$3
            *,*,*  $1,$3,$2

            # left feature
            [left rewrite]
            a,*,*  $2,$1,$3
            *,*,*  $2,$3,$1

            # right feature
            [right rewrite]
            a,*,*  $3,$1,$2
            *,*,*  $3,$2,$1
        ";
        let (unigram_rewriter, left_rewriter, right_rewriter) =
            TrainerConfig::parse_rewrite_config(config.as_bytes()).unwrap();

        // unigram features
        assert_eq!(
            vec!["a", "b", "c"],
            unigram_rewriter.rewrite(&["a", "b", "c"]).unwrap()
        );
        assert_eq!(
            vec!["x", "c", "b"],
            unigram_rewriter.rewrite(&["x", "b", "c"]).unwrap()
        );

        // left features
        assert_eq!(
            vec!["b", "a", "c"],
            left_rewriter.rewrite(&["a", "b", "c"]).unwrap()
        );
        assert_eq!(
            vec!["b", "c", "x"],
            left_rewriter.rewrite(&["x", "b", "c"]).unwrap()
        );

        // right features
        assert_eq!(
            vec!["c", "a", "b"],
            right_rewriter.rewrite(&["a", "b", "c"]).unwrap()
        );
        assert_eq!(
            vec!["c", "b", "x"],
            right_rewriter.rewrite(&["x", "b", "c"]).unwrap()
        );
    }
}
