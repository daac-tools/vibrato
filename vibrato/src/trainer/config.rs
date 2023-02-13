use std::io::{BufRead, BufReader, Read};

use bincode::{
    de::Decoder,
    enc::Encoder,
    error::{DecodeError, EncodeError},
    Decode, Encode,
};

use crate::dictionary::character::CharProperty;
use crate::dictionary::connector::{ConnectorWrapper, MatrixConnector};
use crate::dictionary::lexicon::Lexicon;
use crate::dictionary::unknown::UnkHandler;
use crate::dictionary::{Dictionary, SystemDictionaryBuilder};
use crate::errors::{Result, VibratoError};
use crate::trainer::feature_extractor::FeatureExtractor;
use crate::trainer::feature_rewriter::{FeatureRewriter, FeatureRewriterBuilder};

/// Configuration for a trainer.
pub struct TrainerConfig {
    pub(crate) feature_extractor: FeatureExtractor,
    pub(crate) unigram_rewriter: FeatureRewriter,
    pub(crate) left_rewriter: FeatureRewriter,
    pub(crate) right_rewriter: FeatureRewriter,
    pub(crate) dict: Dictionary,
    pub(crate) surfaces: Vec<String>,
}

impl Decode for TrainerConfig {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        let feature_extractor = Decode::decode(decoder)?;
        let unigram_rewriter = Decode::decode(decoder)?;
        let left_rewriter = Decode::decode(decoder)?;
        let right_rewriter = Decode::decode(decoder)?;
        let dict = Dictionary {
            data: Decode::decode(decoder)?,
            need_check: true,
        };
        let surfaces = Decode::decode(decoder)?;
        Ok(Self {
            feature_extractor,
            unigram_rewriter,
            left_rewriter,
            right_rewriter,
            dict,
            surfaces,
        })
    }
}
bincode::impl_borrow_decode!(TrainerConfig);

impl Encode for TrainerConfig {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        Encode::encode(&self.feature_extractor, encoder)?;
        Encode::encode(&self.unigram_rewriter, encoder)?;
        Encode::encode(&self.left_rewriter, encoder)?;
        Encode::encode(&self.right_rewriter, encoder)?;
        Encode::encode(&self.dict.data, encoder)?;
        Encode::encode(&self.surfaces, encoder)?;
        Ok(())
    }
}

impl TrainerConfig {
    pub(crate) fn parse_feature_config<R>(rdr: R) -> Result<FeatureExtractor>
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
                        builder.add_rule(&pattern, &rewrite);
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
    /// [`VibratoError`] is returned when an input format is invalid.
    pub fn from_readers<L, C, U, F, R>(
        mut lexicon_rdr: L,
        char_prop_rdr: C,
        unk_handler_rdr: U,
        feature_templates_rdr: F,
        rewrite_rules_rdr: R,
    ) -> Result<Self>
    where
        L: Read,
        C: Read,
        U: Read,
        F: Read,
        R: Read,
    {
        let feature_extractor = Self::parse_feature_config(feature_templates_rdr)?;
        let (unigram_rewriter, left_rewriter, right_rewriter) =
            Self::parse_rewrite_config(rewrite_rules_rdr)?;

        let mut lexicon_data = vec![];
        lexicon_rdr.read_to_end(&mut lexicon_data)?;
        let lex_entries = Lexicon::parse_csv(&lexicon_data, "lex.csv")?;
        let connector = MatrixConnector::from_reader(b"1 1\n0 0 0".as_slice())?;
        let char_prop = CharProperty::from_reader(char_prop_rdr)?;
        let unk_handler = UnkHandler::from_reader(unk_handler_rdr, &char_prop)?;

        let dict = SystemDictionaryBuilder::build(
            &lex_entries,
            ConnectorWrapper::Matrix(connector),
            char_prop,
            unk_handler,
        )?;

        let surfaces = lex_entries.into_iter().map(|e| e.surface).collect();

        Ok(Self {
            feature_extractor,
            unigram_rewriter,
            left_rewriter,
            right_rewriter,
            dict,
            surfaces,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::num::NonZeroU32;

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
            vec![NonZeroU32::new(1).unwrap(), NonZeroU32::new(2).unwrap()],
            feature_extractor.extract_unigram_feature_ids(&["a", "b"], 2)
        );
        assert_eq!(
            vec![NonZeroU32::new(3).unwrap(), NonZeroU32::new(4).unwrap()],
            feature_extractor.extract_unigram_feature_ids(&["b", "c"], 2)
        );
        assert_eq!(
            vec![NonZeroU32::new(1).unwrap(), NonZeroU32::new(2).unwrap()],
            feature_extractor.extract_unigram_feature_ids(&["a", "c"], 2)
        );
        assert_eq!(
            vec![NonZeroU32::new(3).unwrap(), NonZeroU32::new(5).unwrap()],
            feature_extractor.extract_unigram_feature_ids(&["b", "c"], 3)
        );

        // left features
        assert_eq!(
            vec![NonZeroU32::new(1), NonZeroU32::new(2)],
            feature_extractor.extract_left_feature_ids(&["a", "b"])
        );
        assert_eq!(
            vec![NonZeroU32::new(3), NonZeroU32::new(4)],
            feature_extractor.extract_left_feature_ids(&["b", "c"])
        );
        assert_eq!(
            vec![NonZeroU32::new(1), NonZeroU32::new(5)],
            feature_extractor.extract_left_feature_ids(&["a", "c"])
        );
        assert_eq!(
            vec![NonZeroU32::new(3), NonZeroU32::new(4)],
            feature_extractor.extract_left_feature_ids(&["b", "c"])
        );

        // right features
        assert_eq!(
            vec![NonZeroU32::new(1), NonZeroU32::new(2)],
            feature_extractor.extract_right_feature_ids(&["a", "b"])
        );
        assert_eq!(
            vec![NonZeroU32::new(3), NonZeroU32::new(4)],
            feature_extractor.extract_right_feature_ids(&["b", "c"])
        );
        assert_eq!(
            vec![NonZeroU32::new(3), NonZeroU32::new(5)],
            feature_extractor.extract_right_feature_ids(&["a", "c"])
        );
        assert_eq!(
            vec![NonZeroU32::new(3), NonZeroU32::new(4)],
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
