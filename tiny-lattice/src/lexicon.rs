pub mod builder;
pub mod parser;
pub mod trie;
pub mod word_param;

use trie::Trie;

pub use word_param::{WordParam, WordParamArrays};

pub struct Lexicon {
    trie: Trie,
    word_params: WordParamArrays,
}

/// Result of the Lexicon lookup
#[derive(Eq, PartialEq, Debug)]
pub struct LexiconEntry {
    /// Id of the returned word
    pub word_param: WordParam,
    /// Byte index of the word end
    pub end_byte: usize,
}

impl LexiconEntry {
    pub fn new(word_param: WordParam, end_byte: usize) -> Self {
        Self {
            word_param,
            end_byte,
        }
    }
}

impl Lexicon {
    /// Returns an iterator of word_id and end of words that matches given input
    #[inline]
    pub fn common_prefix_iterator<'a>(
        &'a self,
        input: &'a [u8],
    ) -> impl Iterator<Item = LexiconEntry> + 'a {
        self.trie.common_prefix_iterator(input).flat_map(move |e| {
            self.word_params
                .iter(e.value)
                .map(move |wp| LexiconEntry::new(wp, e.end))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_prefix_iterator() {
        let mut b = builder::LexiconBuilder::new();
        b.add("東京", WordParam::new(1, 2, 3));
        b.add("東京都", WordParam::new(4, 5, 6));
        b.add("東京", WordParam::new(7, 8, 9));
        b.add("京都", WordParam::new(10, 11, 12));
        let lex = b.build();
        let mut it = lex.common_prefix_iterator("東京都".as_bytes());
        assert_eq!(
            it.next().unwrap(),
            LexiconEntry {
                word_param: WordParam::new(1, 2, 3),
                end_byte: 6
            }
        );
        assert_eq!(
            it.next().unwrap(),
            LexiconEntry {
                word_param: WordParam::new(7, 8, 9),
                end_byte: 6
            }
        );
        assert_eq!(
            it.next().unwrap(),
            LexiconEntry {
                word_param: WordParam::new(4, 5, 6),
                end_byte: 9
            }
        );
        assert_eq!(it.next(), None);
    }
}
