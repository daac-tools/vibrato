pub mod parser;
pub mod word_feats;
pub mod word_map;
pub mod word_params;

pub use word_feats::WordFeats;
pub use word_map::WordMap;
pub use word_params::{WordParam, WordParams};

pub struct Lexicon {
    map: WordMap,
    params: WordParams,
    feats: WordFeats,
}

impl Lexicon {
    pub fn new(entries: &[(&str, WordParam, &str)]) -> Self {
        let map = WordMap::from_iter(entries.iter().map(|e| e.0));
        let params = WordParams::from_iter(entries.iter().map(|e| e.1));
        let feats = WordFeats::from_iter(entries.iter().map(|e| e.2));
        Self { map, params, feats }
    }

    #[inline(always)]
    pub fn common_prefix_iterator<'a>(
        &'a self,
        input: &'a [u8],
    ) -> impl Iterator<Item = LexiconMatch> + 'a {
        self.map.common_prefix_iterator(input).map(move |e| {
            LexiconMatch::new(e.word_id, self.params.get(e.word_id as usize), e.end_byte)
        })
    }

    #[inline(always)]
    pub fn get_word_feature(&self, word_id: u32) -> &str {
        self.feats.get(word_id as usize)
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct LexiconMatch {
    end_byte: u32,
    word_id: u32,
    word_param: WordParam,
}

impl LexiconMatch {
    #[inline(always)]
    pub fn new(word_id: u32, word_param: WordParam, end_byte: u32) -> Self {
        Self {
            word_id,
            word_param,
            end_byte,
        }
    }

    #[inline(always)]
    pub fn end_byte(&self) -> usize {
        self.end_byte as usize
    }

    #[inline(always)]
    pub fn word_id(&self) -> u32 {
        self.word_id
    }

    #[inline(always)]
    pub fn word_param(&self) -> WordParam {
        self.word_param
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RawWordEntry {
    pub surface: String,
    pub param: WordParam,
    pub feat: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_prefix_iterator() {
        let entries = vec![
            ("東京", WordParam::new(1, 2, 3), ""),
            ("東京都", WordParam::new(4, 5, 6), ""),
            ("東京", WordParam::new(7, 8, 9), ""),
            ("京都", WordParam::new(10, 11, 12), ""),
        ];
        let lexicon = Lexicon::new(&entries);
        let mut it = lexicon.common_prefix_iterator("東京都".as_bytes());
        assert_eq!(
            it.next().unwrap(),
            LexiconMatch {
                end_byte: 6,
                word_id: 0,
                word_param: WordParam::new(1, 2, 3),
            }
        );
        assert_eq!(
            it.next().unwrap(),
            LexiconMatch {
                end_byte: 6,
                word_id: 2,
                word_param: WordParam::new(7, 8, 9),
            }
        );
        assert_eq!(
            it.next().unwrap(),
            LexiconMatch {
                end_byte: 9,
                word_id: 1,
                word_param: WordParam::new(4, 5, 6),
            }
        );
        assert_eq!(it.next(), None);
    }
}