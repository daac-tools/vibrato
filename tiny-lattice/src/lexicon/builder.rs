use std::collections::BTreeMap;

use super::parser::RawLexiconEntry;
use super::trie::Trie;
use super::word_param::{WordParam, WordParamArrays};
use super::Lexicon;

#[derive(Default)]
pub struct LexiconBuilder {
    map: BTreeMap<String, Vec<WordParam>>,
}

impl LexiconBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn extend_from_raw_entries<I>(&mut self, entries: I)
    where
        I: IntoIterator<Item = RawLexiconEntry>,
    {
        for e in entries {
            self.add(&e.surface, WordParam::from_raw_entry(&e));
        }
    }

    pub fn add(&mut self, key: &str, param: WordParam) {
        self.map.entry(key.to_owned()).or_default().push(param);
    }

    pub fn build(&self) -> Lexicon {
        let mut trie_entries = vec![];
        let mut param_arrays = vec![];
        for (i, (key, params)) in self.map.iter().enumerate() {
            trie_entries.push((key, i as u32));
            param_arrays.push(params.to_owned());
        }
        let trie_data = yada::builder::DoubleArrayBuilder::build(&trie_entries).unwrap();
        Lexicon {
            trie: Trie::new(trie_data),
            word_params: WordParamArrays::new(param_arrays),
        }
    }
}
