mod builder;
mod feature;
mod map;
mod param;

use bincode::{Decode, Encode};

use super::connector::Connector;
use super::mapper::ConnIdMapper;
use super::{LexType, WordIdx};
use crate::utils::FromU32;
use feature::WordFeatures;
use map::WordMap;
pub use param::WordParam;
use param::WordParams;

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
                    self.params.get(usize::from_u32(word_id)),
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
        debug_assert_eq!(word_idx.lex_type, self.lex_type);
        self.features.get(usize::from_u32(word_idx.word_id))
    }

    /// Checks if left/right-ids are valid with connector.
    pub(crate) fn verify(&self, conn: &Connector) -> bool {
        for i in 0..self.params.len() {
            let p = self.params.get(i);
            if conn.num_left() <= usize::from(p.left_id) {
                return false;
            }
            if conn.num_right() <= usize::from(p.right_id) {
                return false;
            }
        }
        true
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct LexMatch {
    pub(crate) word_idx: WordIdx,
    pub(crate) word_param: WordParam,
    pub(crate) end_char: u16,
}

impl LexMatch {
    #[inline(always)]
    pub const fn new(word_idx: WordIdx, word_param: WordParam, end_char: u16) -> Self {
        Self {
            word_idx,
            word_param,
            end_char,
        }
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
