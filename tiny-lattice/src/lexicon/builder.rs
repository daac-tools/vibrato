use std::collections::BTreeMap;

use super::trie::Trie;
use super::word_param::{WordParam, WordParamArrays};
use super::Lexicon;

#[derive(Default)]
pub struct LexiconBuilder<'a> {
    map: BTreeMap<&'a str, Vec<WordParam>>,
}

impl<'a> LexiconBuilder<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, key: &'a str, param: WordParam) {
        self.map.entry(key).or_default().push(param);
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
