mod builder;
mod feature;
mod map;
mod param;

use bincode::{Decode, Encode};

use super::mapper::ConnIdMapper;
use super::{LexType, WordIdx};
use crate::utils::FromU32;
use feature::WordFeatures;
use map::WordMap;
use param::WordParams;

pub use param::WordParam;

/// Lexicon of words.
#[derive(Decode, Encode)]
pub struct Lexicon {
    map: WordMap,
    params: WordParams,
    features: WordFeatures,
    lex_type: LexType,
}

impl Lexicon {
    #[inline(always)]
    pub(crate) fn common_prefix_iterator<'a>(
        &'a self,
        input: &'a [char],
    ) -> impl Iterator<Item = LexMatch> + 'a {
        self.map
            .common_prefix_iterator(input)
            .map(move |(word_id, end_char)| {
                LexMatch::new(
                    WordIdx::new(self.lex_type, word_id),
                    self.params.param(usize::from_u32(word_id)),
                    end_char,
                )
            })
    }

    /// Do NOT make this function public to maintain consistency in
    /// the connection-id mapping among members of `Dictionary`.
    /// The consistency is managed in `Dictionary`.
    pub(crate) fn do_mapping(&mut self, mapper: &ConnIdMapper) {
        self.params.do_mapping(mapper);
    }

    #[inline(always)]
    pub(crate) fn word_feature(&self, word_idx: WordIdx) -> &str {
        debug_assert_eq!(word_idx.lex_type(), self.lex_type);
        self.features.feature(usize::from_u32(word_idx.word_id()))
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct LexMatch {
    word_idx: WordIdx,
    word_param: WordParam,
    end_char: u32,
}

impl LexMatch {
    #[inline(always)]
    pub const fn new(word_idx: WordIdx, word_param: WordParam, end_char: u32) -> Self {
        Self {
            word_idx,
            word_param,
            end_char,
        }
    }

    #[inline(always)]
    pub const fn end_char(&self) -> usize {
        self.end_char as usize
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
            map: WordMap::new(["東京", "東京都", "東京", "京都"]).unwrap(),
            params: WordParams::new([
                WordParam::new(1, 2, 3),
                WordParam::new(4, 5, 6),
                WordParam::new(7, 8, 9),
                WordParam::new(10, 11, 12),
            ]),
            features: WordFeatures::default(),
            lex_type: LexType::System,
        };
        let input: Vec<_> = "東京都".chars().collect();
        let mut it = lexicon.common_prefix_iterator(&input);
        assert_eq!(
            it.next().unwrap(),
            LexMatch {
                end_char: 2,
                word_idx: WordIdx::new(LexType::System, 0),
                word_param: WordParam::new(1, 2, 3),
            }
        );
        assert_eq!(
            it.next().unwrap(),
            LexMatch {
                end_char: 2,
                word_idx: WordIdx::new(LexType::System, 2),
                word_param: WordParam::new(7, 8, 9),
            }
        );
        assert_eq!(
            it.next().unwrap(),
            LexMatch {
                end_char: 3,
                word_idx: WordIdx::new(LexType::System, 1),
                word_param: WordParam::new(4, 5, 6),
            }
        );
        assert_eq!(it.next(), None);
    }
}
