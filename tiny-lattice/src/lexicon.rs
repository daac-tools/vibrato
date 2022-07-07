pub mod builder;
pub mod parser;
pub mod trie;
pub mod word_param;

use trie::Trie;
use word_param::{WordParam, WordParamArrays};

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
    pub fn lookup<'a>(
        &'a self,
        input: &'a [u8],
        offset: usize,
    ) -> impl Iterator<Item = LexiconEntry> + 'a {
        self.trie
            .common_prefix_iterator(input, offset)
            .flat_map(move |e| {
                self.word_params
                    .iter(e.value)
                    .map(move |param| LexiconEntry::new(param, e.end))
            })
    }
}
