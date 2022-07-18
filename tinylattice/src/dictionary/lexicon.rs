pub mod builder;
pub mod word_feature;
pub mod word_map;
pub mod word_param;

pub use super::{LexType, WordIdx};
pub use word_feature::WordFeatures;
pub use word_map::WordMap;
pub use word_param::{WordParam, WordParams};

pub struct Lexicon {
    map: WordMap,
    params: WordParams,
    features: WordFeatures,
    lex_type: LexType,
}

impl Lexicon {
    #[inline(always)]
    pub fn common_prefix_iterator<'a>(
        &'a self,
        input: &'a [u8],
    ) -> impl Iterator<Item = LexMatch> + 'a {
        self.map
            .common_prefix_iterator(input)
            .map(move |(word_id, end_byte)| {
                LexMatch::new(
                    WordIdx::new(self.lex_type, word_id),
                    self.params.param(word_id as usize),
                    end_byte,
                )
            })
    }

    #[inline(always)]
    pub fn word_feature(&self, word_idx: WordIdx) -> &str {
        debug_assert_eq!(word_idx.lex_type(), self.lex_type);
        self.features.feature(word_idx.word_id() as usize)
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct LexMatch {
    word_idx: WordIdx,
    word_param: WordParam,
    end_byte: u32,
}

impl LexMatch {
    #[inline(always)]
    pub fn new(word_idx: WordIdx, word_param: WordParam, end_byte: u32) -> Self {
        Self {
            word_idx,
            word_param,
            end_byte,
        }
    }

    #[inline(always)]
    pub const fn end_byte(&self) -> usize {
        self.end_byte as usize
    }

    #[inline(always)]
    pub const fn word_idx(&self) -> WordIdx {
        self.word_idx
    }

    #[inline(always)]
    pub const fn word_param(&self) -> WordParam {
        self.word_param
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RawWordEntry {
    pub surface: String,
    pub param: WordParam,
    pub feature: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_prefix_iterator() {
        let lexicon = Lexicon {
            map: WordMap::from_iter(["東京", "東京都", "東京", "京都"]),
            params: WordParams::from_iter([
                WordParam::new(1, 2, 3),
                WordParam::new(4, 5, 6),
                WordParam::new(7, 8, 9),
                WordParam::new(10, 11, 12),
            ]),
            features: WordFeatures::default(),
            lex_type: LexType::System,
        };
        let mut it = lexicon.common_prefix_iterator("東京都".as_bytes());
        assert_eq!(
            it.next().unwrap(),
            LexMatch {
                end_byte: 6,
                word_idx: WordIdx::new(LexType::System, 0),
                word_param: WordParam::new(1, 2, 3),
            }
        );
        assert_eq!(
            it.next().unwrap(),
            LexMatch {
                end_byte: 6,
                word_idx: WordIdx::new(LexType::System, 2),
                word_param: WordParam::new(7, 8, 9),
            }
        );
        assert_eq!(
            it.next().unwrap(),
            LexMatch {
                end_byte: 9,
                word_idx: WordIdx::new(LexType::System, 1),
                word_param: WordParam::new(4, 5, 6),
            }
        );
        assert_eq!(it.next(), None);
    }
}
