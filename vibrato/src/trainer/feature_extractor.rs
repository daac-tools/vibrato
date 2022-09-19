use std::{num::NonZeroU32, ops::Range};

use bincode::{
    de::Decoder,
    enc::Encoder,
    error::{DecodeError, EncodeError},
    Decode, Encode,
};
use hashbrown::HashMap;
use regex::Regex;

#[derive(Debug, Decode, Encode)]
enum FeatureType {
    Index(usize),
    CharacterType,
}

#[derive(Debug, Decode, Encode)]
struct ParsedTemplate {
    raw_template: String,
    required_indices: Vec<usize>,
    captures: Vec<(Range<usize>, FeatureType)>,
}

pub struct FeatureExtractor {
    unigram_feature_ids: HashMap<String, NonZeroU32>,
    left_feature_ids: HashMap<String, NonZeroU32>,
    right_feature_ids: HashMap<String, NonZeroU32>,
    unigram_templates: Vec<ParsedTemplate>,
    left_templates: Vec<ParsedTemplate>,
    right_templates: Vec<ParsedTemplate>,
}

impl FeatureExtractor {
    pub fn new<S>(unigram_templates: &[S], bigram_templates: &[(S, S)]) -> Self
    where
        S: ToString,
    {
        let unigram_feature_pattern = Regex::new(r"%((F|F\?)\[([0-9]+)\]|t)").unwrap();
        let left_feature_pattern = Regex::new(r"%(L|L\?)\[([0-9]+)\]").unwrap();
        let right_feature_pattern = Regex::new(r"%(R|R\?)\[([0-9]+)\]").unwrap();

        let mut unigram_parsed_templates = vec![];
        for template in unigram_templates {
            let raw_template = template.to_string();
            let mut required_indices = vec![];
            let mut captures = vec![];
            for m in unigram_feature_pattern.captures_iter(&raw_template) {
                let pattern = m.get(0).unwrap();
                if m.get(1).unwrap().as_str() == "t" {
                    captures.push((pattern.start()..pattern.end(), FeatureType::CharacterType));
                } else {
                    let idx: usize = m.get(3).unwrap().as_str().parse().unwrap();
                    match m.get(2).unwrap().as_str() {
                        "F" => {
                            captures
                                .push((pattern.start()..pattern.end(), FeatureType::Index(idx)));
                        }
                        "F?" => {
                            required_indices.push(idx);
                            captures
                                .push((pattern.start()..pattern.end(), FeatureType::Index(idx)));
                        }
                        _ => unreachable!(),
                    }
                }
            }
            unigram_parsed_templates.push(ParsedTemplate {
                raw_template,
                required_indices,
                captures,
            });
        }

        let mut left_parsed_templates = vec![];
        let mut right_parsed_templates = vec![];
        for (left_template, right_template) in bigram_templates {
            {
                let raw_template = left_template.to_string();
                let mut required_indices = vec![];
                let mut captures = vec![];
                for m in left_feature_pattern.captures_iter(&raw_template) {
                    let pattern = m.get(0).unwrap();
                    let idx: usize = m.get(2).unwrap().as_str().parse().unwrap();
                    match m.get(1).unwrap().as_str() {
                        "L" => {
                            captures
                                .push((pattern.start()..pattern.end(), FeatureType::Index(idx)));
                        }
                        "L?" => {
                            required_indices.push(idx);
                            captures
                                .push((pattern.start()..pattern.end(), FeatureType::Index(idx)));
                        }
                        _ => unreachable!(),
                    }
                }
                left_parsed_templates.push(ParsedTemplate {
                    raw_template,
                    required_indices,
                    captures,
                });
            }
            {
                let raw_template = right_template.to_string();
                let mut required_indices = vec![];
                let mut captures = vec![];
                for m in right_feature_pattern.captures_iter(&raw_template) {
                    let pattern = m.get(0).unwrap();
                    let idx: usize = m.get(2).unwrap().as_str().parse().unwrap();
                    match m.get(1).unwrap().as_str() {
                        "R" => {
                            captures
                                .push((pattern.start()..pattern.end(), FeatureType::Index(idx)));
                        }
                        "R?" => {
                            required_indices.push(idx);
                            captures
                                .push((pattern.start()..pattern.end(), FeatureType::Index(idx)));
                        }
                        _ => unreachable!(),
                    }
                }
                right_parsed_templates.push(ParsedTemplate {
                    raw_template,
                    required_indices,
                    captures,
                });
            }
        }

        Self {
            unigram_feature_ids: HashMap::new(),
            left_feature_ids: HashMap::new(),
            right_feature_ids: HashMap::new(),
            unigram_templates: unigram_parsed_templates,
            left_templates: left_parsed_templates,
            right_templates: right_parsed_templates,
        }
    }

    /// Inserts feature patterns matched to the input templates in the hash map,
    /// while incrementally assigning new feature ids.
    /// Returns a sequence of ids of found features.
    fn extract_feature_ids<S>(
        features: &[S],
        templates: &[ParsedTemplate],
        feature_ids: &mut HashMap<String, NonZeroU32>,
        category_id: u32,
    ) -> Vec<Option<NonZeroU32>>
    where
        S: AsRef<str>,
    {
        let mut result = vec![];
        'a: for template in templates {
            for &required_idx in &template.required_indices {
                if features.get(required_idx).map_or("*", |f| f.as_ref()) == "*" {
                    result.push(None);
                    continue 'a;
                }
            }
            let mut feature_string = String::new();
            let mut start = 0;
            for (range, feature) in &template.captures {
                feature_string.push_str(&template.raw_template[start..range.start]);
                match feature {
                    FeatureType::Index(idx) => {
                        feature_string.push_str(features.get(*idx).map_or("*", |f| f.as_ref()));
                    }
                    FeatureType::CharacterType => {
                        feature_string.push_str(&category_id.to_string());
                    }
                }
                start = range.end;
            }
            feature_string.push_str(&template.raw_template[start..]);
            let new_id = NonZeroU32::new(u32::try_from(feature_ids.len() + 1).unwrap()).unwrap();
            let feature_id = *feature_ids.entry(feature_string).or_insert(new_id);
            result.push(Some(feature_id));
        }
        result
    }

    pub fn extract_unigram_feature_ids<S>(
        &mut self,
        features: &[S],
        category_id: u32,
    ) -> Vec<NonZeroU32>
    where
        S: AsRef<str>,
    {
        Self::extract_feature_ids(
            features,
            &self.unigram_templates,
            &mut self.unigram_feature_ids,
            category_id,
        )
        .into_iter()
        .flatten()
        .collect()
    }

    pub fn extract_left_feature_ids<S>(&mut self, features: &[S]) -> Vec<Option<NonZeroU32>>
    where
        S: AsRef<str>,
    {
        Self::extract_feature_ids(
            features,
            &self.left_templates,
            &mut self.left_feature_ids,
            0,
        )
    }

    pub fn extract_right_feature_ids<S>(&mut self, features: &[S]) -> Vec<Option<NonZeroU32>>
    where
        S: AsRef<str>,
    {
        Self::extract_feature_ids(
            features,
            &self.right_templates,
            &mut self.right_feature_ids,
            0,
        )
    }

    pub const fn left_feature_ids(&self) -> &HashMap<String, NonZeroU32> {
        &self.left_feature_ids
    }

    pub const fn right_feature_ids(&self) -> &HashMap<String, NonZeroU32> {
        &self.right_feature_ids
    }
}

impl Decode for FeatureExtractor {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        let unigram_feature_ids: Vec<(String, NonZeroU32)> = Decode::decode(decoder)?;
        let left_feature_ids: Vec<(String, NonZeroU32)> = Decode::decode(decoder)?;
        let right_feature_ids: Vec<(String, NonZeroU32)> = Decode::decode(decoder)?;
        let unigram_templates = Decode::decode(decoder)?;
        let left_templates = Decode::decode(decoder)?;
        let right_templates = Decode::decode(decoder)?;
        Ok(Self {
            unigram_feature_ids: unigram_feature_ids.into_iter().collect(),
            left_feature_ids: left_feature_ids.into_iter().collect(),
            right_feature_ids: right_feature_ids.into_iter().collect(),
            unigram_templates,
            left_templates,
            right_templates,
        })
    }
}

impl Encode for FeatureExtractor {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        let unigram_feature_ids: Vec<(String, NonZeroU32)> =
            self.unigram_feature_ids.clone().into_iter().collect();
        let left_feature_ids: Vec<(String, NonZeroU32)> =
            self.left_feature_ids.clone().into_iter().collect();
        let right_feature_ids: Vec<(String, NonZeroU32)> =
            self.right_feature_ids.clone().into_iter().collect();
        Encode::encode(&unigram_feature_ids, encoder)?;
        Encode::encode(&left_feature_ids, encoder)?;
        Encode::encode(&right_feature_ids, encoder)?;
        Encode::encode(&self.unigram_templates, encoder)?;
        Encode::encode(&self.left_templates, encoder)?;
        Encode::encode(&self.right_templates, encoder)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::test_utils::hashmap;

    fn prepare_extractor() -> FeatureExtractor {
        let unigram_templates = vec![
            "word:%F[0]",
            "word-pos:%F[0],%F[1]",
            "word-pron:%F[0],%F?[2]",
            "word-pos-pron:%F[0],%F[1],%F?[2]",
            "word-type:%F[0],%t",
        ];
        let bigram_templates = vec![
            ("pos:%L[1]", "pos:%R[1]"),
            ("pron:%L?[2]", "pron:%R?[2]"),
            ("pos-pron:%L[1],%L?[2]", "pos-pron:%R[1],%R?[2]"),
        ];

        FeatureExtractor::new(&unigram_templates, &bigram_templates)
    }

    #[test]
    fn test_unigram_feature_extraction() {
        let mut extractor = prepare_extractor();

        let feature_ids = extractor.extract_unigram_feature_ids(&["人", "名詞", "ヒト"], 3);
        assert_eq!(
            vec![
                NonZeroU32::new(1).unwrap(),
                NonZeroU32::new(2).unwrap(),
                NonZeroU32::new(3).unwrap(),
                NonZeroU32::new(4).unwrap(),
                NonZeroU32::new(5).unwrap()
            ],
            feature_ids
        );

        let feature_ids = extractor.extract_unigram_feature_ids(&["人", "接尾辞", "ジン"], 3);
        assert_eq!(
            vec![
                NonZeroU32::new(1).unwrap(),
                NonZeroU32::new(6).unwrap(),
                NonZeroU32::new(7).unwrap(),
                NonZeroU32::new(8).unwrap(),
                NonZeroU32::new(5).unwrap()
            ],
            feature_ids
        );

        assert_eq!(
            hashmap![
                "word:人".to_string() => NonZeroU32::new(1).unwrap(),
                "word-pos:人,名詞".to_string() => NonZeroU32::new(2).unwrap(),
                "word-pron:人,ヒト".to_string() => NonZeroU32::new(3).unwrap(),
                "word-pos-pron:人,名詞,ヒト".to_string() => NonZeroU32::new(4).unwrap(),
                "word-type:人,3".to_string() => NonZeroU32::new(5).unwrap(),
                "word-pos:人,接尾辞".to_string() => NonZeroU32::new(6).unwrap(),
                "word-pron:人,ジン".to_string() => NonZeroU32::new(7).unwrap(),
                "word-pos-pron:人,接尾辞,ジン".to_string() => NonZeroU32::new(8).unwrap(),
            ],
            extractor.unigram_feature_ids
        );
    }

    #[test]
    fn test_unigram_feature_extraction_undefined() {
        let mut extractor = prepare_extractor();

        let feature_ids = extractor.extract_unigram_feature_ids(&["。", "補助記号", "*"], 4);
        assert_eq!(
            vec![
                NonZeroU32::new(1).unwrap(),
                NonZeroU32::new(2).unwrap(),
                NonZeroU32::new(3).unwrap()
            ],
            feature_ids
        );

        let feature_ids = extractor.extract_unigram_feature_ids(&["、", "補助記号", "*"], 4);
        assert_eq!(
            vec![
                NonZeroU32::new(4).unwrap(),
                NonZeroU32::new(5).unwrap(),
                NonZeroU32::new(6).unwrap()
            ],
            feature_ids
        );

        assert_eq!(
            hashmap![
                "word:。".to_string() => NonZeroU32::new(1).unwrap(),
                "word-pos:。,補助記号".to_string() => NonZeroU32::new(2).unwrap(),
                "word-type:。,4".to_string() => NonZeroU32::new(3).unwrap(),
                "word:、".to_string() => NonZeroU32::new(4).unwrap(),
                "word-pos:、,補助記号".to_string() => NonZeroU32::new(5).unwrap(),
                "word-type:、,4".to_string() => NonZeroU32::new(6).unwrap(),
            ],
            extractor.unigram_feature_ids
        );
    }

    #[test]
    fn test_bigram_feature_extraction() {
        let mut extractor = prepare_extractor();

        let left_feature_ids = extractor.extract_left_feature_ids(&["火星", "名詞", "カセイ"]);
        let right_feature_ids = extractor.extract_right_feature_ids(&["人", "接尾辞", "ジン"]);
        assert_eq!(
            vec![NonZeroU32::new(1), NonZeroU32::new(2), NonZeroU32::new(3)],
            left_feature_ids
        );
        assert_eq!(
            vec![NonZeroU32::new(1), NonZeroU32::new(2), NonZeroU32::new(3)],
            right_feature_ids
        );

        let left_feature_ids = extractor.extract_left_feature_ids(&["火星", "名詞", "カセイ"]);
        let right_feature_ids = extractor.extract_right_feature_ids(&["猫", "名詞", "ネコ"]);
        assert_eq!(
            vec![NonZeroU32::new(1), NonZeroU32::new(2), NonZeroU32::new(3)],
            left_feature_ids
        );
        assert_eq!(
            vec![NonZeroU32::new(4), NonZeroU32::new(5), NonZeroU32::new(6)],
            right_feature_ids
        );

        assert_eq!(
            hashmap![
                "pos:名詞".to_string() => NonZeroU32::new(1).unwrap(),
                "pron:カセイ".to_string() => NonZeroU32::new(2).unwrap(),
                "pos-pron:名詞,カセイ".to_string() => NonZeroU32::new(3).unwrap(),
            ],
            extractor.left_feature_ids
        );

        assert_eq!(
            hashmap![
                "pos:接尾辞".to_string() => NonZeroU32::new(1).unwrap(),
                "pron:ジン".to_string() => NonZeroU32::new(2).unwrap(),
                "pos-pron:接尾辞,ジン".to_string() => NonZeroU32::new(3).unwrap(),
                "pos:名詞".to_string() => NonZeroU32::new(4).unwrap(),
                "pron:ネコ".to_string() => NonZeroU32::new(5).unwrap(),
                "pos-pron:名詞,ネコ".to_string() => NonZeroU32::new(6).unwrap(),
            ],
            extractor.right_feature_ids
        );
    }

    #[test]
    fn test_bigram_feature_extraction_undefined() {
        let mut extractor = prepare_extractor();

        let left_feature_ids = extractor.extract_left_feature_ids(&["です", "助動詞", "デス"]);
        let right_feature_ids = extractor.extract_right_feature_ids(&["。", "補助記号", "*"]);
        assert_eq!(
            vec![NonZeroU32::new(1), NonZeroU32::new(2), NonZeroU32::new(3)],
            left_feature_ids
        );
        assert_eq!(vec![NonZeroU32::new(1), None, None], right_feature_ids);

        let left_feature_ids = extractor.extract_left_feature_ids(&["「", "補助記号", "*"]);
        let right_feature_ids = extractor.extract_right_feature_ids(&["猫", "名詞", "ネコ"]);
        assert_eq!(vec![NonZeroU32::new(4), None, None], left_feature_ids);
        assert_eq!(
            vec![NonZeroU32::new(2), NonZeroU32::new(3), NonZeroU32::new(4)],
            right_feature_ids
        );

        assert_eq!(
            hashmap![
                "pos:助動詞".to_string() => NonZeroU32::new(1).unwrap(),
                "pron:デス".to_string() => NonZeroU32::new(2).unwrap(),
                "pos-pron:助動詞,デス".to_string() => NonZeroU32::new(3).unwrap(),
                "pos:補助記号".to_string() => NonZeroU32::new(4).unwrap(),
            ],
            extractor.left_feature_ids
        );

        assert_eq!(
            hashmap![
                "pos:補助記号".to_string() => NonZeroU32::new(1).unwrap(),
                "pos:名詞".to_string() => NonZeroU32::new(2).unwrap(),
                "pron:ネコ".to_string() => NonZeroU32::new(3).unwrap(),
                "pos-pron:名詞,ネコ".to_string() => NonZeroU32::new(4).unwrap(),
            ],
            extractor.right_feature_ids
        );
    }

    #[test]
    fn test_fill_aster() {
        let mut extractor = prepare_extractor();

        extractor.extract_unigram_feature_ids(&["。"], 4);

        assert_eq!(
            hashmap![
                "word:。".to_string() => NonZeroU32::new(1).unwrap(),
                "word-pos:。,*".to_string() => NonZeroU32::new(2).unwrap(),
                "word-type:。,4".to_string() => NonZeroU32::new(3).unwrap(),
            ],
            extractor.unigram_feature_ids
        );
    }
}
