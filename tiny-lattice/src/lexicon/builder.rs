use std::collections::BTreeMap;

use super::trie::Trie;
use super::word_param::{WordParam, WordParamArrays};

#[derive(Default)]
pub struct LexiconBuilder<'a> {
    map: BTreeMap<&'a str, Vec<i16>>,
}

impl<'a> LexiconBuilder<'a> {
    pub fn add(&mut self, key: &'a str, param: WordParam) {
        self.map.entry(key).or_default().extend_from_slice(&[
            param.left_id,
            param.right_id,
            param.cost,
        ])
    }

    pub fn build(&self) -> (Trie, WordParamArrays) {
        let mut trie_entries = vec![];
        let mut param_arrays = vec![];
        for (i, (key, params)) in self.map.iter().enumerate() {
            trie_entries.push((key, i as u32));
            param_arrays.push(params.to_owned());
        }
        let trie_data = yada::builder::DoubleArrayBuilder::build(&trie_entries).unwrap();
        (Trie::new(trie_data), WordParamArrays::new(param_arrays))
    }
}
