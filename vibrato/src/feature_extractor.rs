use std::ops::Range;

use hashbrown::HashMap;
use regex::Regex;

#[derive(Debug)]
enum FeatureType {
    Index(usize),
    CharacterType,
}

#[derive(Debug)]
struct ParsedTemplate {
    raw_template: String,
    required_indices: Vec<usize>,
    captures: Vec<(Range<usize>, FeatureType)>,
}

#[derive(Debug)]
pub struct FeatureExtractor {
    unigram_feature_ids: HashMap<String, usize>,
    left_feature_ids: HashMap<String, usize>,
    right_feature_ids: HashMap<String, usize>,
    unigram_templates: Vec<ParsedTemplate>,
    left_templates: Vec<ParsedTemplate>,
    right_templates: Vec<ParsedTemplate>,
}

impl FeatureExtractor {
    #[allow(unused)]
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

    #[allow(unused)]
    fn extract_feature_ids<S>(
        features: &[S],
        templates: &[ParsedTemplate],
        feature_ids: &mut HashMap<String, usize>,
        char_types: &str,
    ) -> Vec<Option<usize>>
    where
        S: AsRef<str>,
    {
        let mut result = vec![];
        'a: for template in templates {
            for &required_idx in &template.required_indices {
                if features[required_idx].as_ref() == "*" {
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
                        feature_string.push_str(features[*idx].as_ref());
                    }
                    FeatureType::CharacterType => {
                        feature_string.push_str(char_types);
                    }
                }
                start = range.end;
            }
            feature_string.push_str(&template.raw_template[start..]);
            let new_id = feature_ids.len();
            let feature_id = *feature_ids.entry(feature_string).or_insert(new_id);
            result.push(Some(feature_id));
        }
        result
    }

    #[allow(unused)]
    pub fn extract_unigram_feature_ids<S>(&mut self, features: &[S], char_types: &str) -> Vec<usize>
    where
        S: AsRef<str>,
    {
        Self::extract_feature_ids(
            features,
            &self.unigram_templates,
            &mut self.unigram_feature_ids,
            char_types,
        )
        .into_iter()
        .flatten()
        .collect()
    }

    #[allow(unused)]
    pub fn extract_left_feature_ids<S>(&mut self, features: &[S]) -> Vec<Option<usize>>
    where
        S: AsRef<str>,
    {
        Self::extract_feature_ids(
            features,
            &self.left_templates,
            &mut self.left_feature_ids,
            "",
        )
    }

    #[allow(unused)]
    pub fn extract_right_feature_ids<S>(&mut self, features: &[S]) -> Vec<Option<usize>>
    where
        S: AsRef<str>,
    {
        Self::extract_feature_ids(
            features,
            &self.right_templates,
            &mut self.right_feature_ids,
            "",
        )
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

        let feature_ids = extractor.extract_unigram_feature_ids(&["人", "名詞", "ヒト"], "KANJI");
        assert_eq!(vec![0, 1, 2, 3, 4], feature_ids);

        let feature_ids = extractor.extract_unigram_feature_ids(&["人", "接尾辞", "ジン"], "KANJI");
        assert_eq!(vec![0, 5, 6, 7, 4], feature_ids);

        assert_eq!(
            hashmap![
                "word:人".to_string() => 0,
                "word-pos:人,名詞".to_string() => 1,
                "word-pron:人,ヒト".to_string() => 2,
                "word-pos-pron:人,名詞,ヒト".to_string() => 3,
                "word-type:人,KANJI".to_string() => 4,
                "word-pos:人,接尾辞".to_string() => 5,
                "word-pron:人,ジン".to_string() => 6,
                "word-pos-pron:人,接尾辞,ジン".to_string() => 7,
            ],
            extractor.unigram_feature_ids
        );
    }

    #[test]
    fn test_unigram_feature_extraction_undefined() {
        let mut extractor = prepare_extractor();

        let feature_ids = extractor.extract_unigram_feature_ids(&["。", "補助記号", "*"], "OTHER");
        assert_eq!(vec![0, 1, 2], feature_ids);

        let feature_ids = extractor.extract_unigram_feature_ids(&["、", "補助記号", "*"], "OTHER");
        assert_eq!(vec![3, 4, 5], feature_ids);

        assert_eq!(
            hashmap![
                "word:。".to_string() => 0,
                "word-pos:。,補助記号".to_string() => 1,
                "word-type:。,OTHER".to_string() => 2,
                "word:、".to_string() => 3,
                "word-pos:、,補助記号".to_string() => 4,
                "word-type:、,OTHER".to_string() => 5,
            ],
            extractor.unigram_feature_ids
        );
    }

    #[test]
    fn test_bigram_feature_extraction() {
        let mut extractor = prepare_extractor();

        let left_feature_ids = extractor.extract_left_feature_ids(&["火星", "名詞", "カセイ"]);
        let right_feature_ids = extractor.extract_right_feature_ids(&["人", "接尾辞", "ジン"]);
        assert_eq!(vec![Some(0), Some(1), Some(2)], left_feature_ids);
        assert_eq!(vec![Some(0), Some(1), Some(2)], right_feature_ids);

        let left_feature_ids = extractor.extract_left_feature_ids(&["火星", "名詞", "カセイ"]);
        let right_feature_ids = extractor.extract_right_feature_ids(&["猫", "名詞", "ネコ"]);
        assert_eq!(vec![Some(0), Some(1), Some(2)], left_feature_ids);
        assert_eq!(vec![Some(3), Some(4), Some(5)], right_feature_ids);

        assert_eq!(
            hashmap![
                "pos:名詞".to_string() => 0,
                "pron:カセイ".to_string() => 1,
                "pos-pron:名詞,カセイ".to_string() => 2,
            ],
            extractor.left_feature_ids
        );

        assert_eq!(
            hashmap![
                "pos:接尾辞".to_string() => 0,
                "pron:ジン".to_string() => 1,
                "pos-pron:接尾辞,ジン".to_string() => 2,
                "pos:名詞".to_string() => 3,
                "pron:ネコ".to_string() => 4,
                "pos-pron:名詞,ネコ".to_string() => 5,
            ],
            extractor.right_feature_ids
        );
    }

    #[test]
    fn test_bigram_feature_extraction_undefined() {
        let mut extractor = prepare_extractor();

        let left_feature_ids = extractor.extract_left_feature_ids(&["です", "助動詞", "デス"]);
        let right_feature_ids = extractor.extract_right_feature_ids(&["。", "補助記号", "*"]);
        assert_eq!(vec![Some(0), Some(1), Some(2)], left_feature_ids);
        assert_eq!(vec![Some(0), None, None], right_feature_ids);

        let left_feature_ids = extractor.extract_left_feature_ids(&["「", "補助記号", "*"]);
        let right_feature_ids = extractor.extract_right_feature_ids(&["猫", "名詞", "ネコ"]);
        assert_eq!(vec![Some(3), None, None], left_feature_ids);
        assert_eq!(vec![Some(1), Some(2), Some(3)], right_feature_ids);

        assert_eq!(
            hashmap![
                "pos:助動詞".to_string() => 0,
                "pron:デス".to_string() => 1,
                "pos-pron:助動詞,デス".to_string() => 2,
                "pos:補助記号".to_string() => 3,
            ],
            extractor.left_feature_ids
        );

        assert_eq!(
            hashmap![
                "pos:補助記号".to_string() => 0,
                "pos:名詞".to_string() => 1,
                "pron:ネコ".to_string() => 2,
                "pos-pron:名詞,ネコ".to_string() => 3,
            ],
            extractor.right_feature_ids
        );
    }
}
