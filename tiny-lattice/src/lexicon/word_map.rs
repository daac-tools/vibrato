use std::collections::BTreeMap;

use super::id_lists::{IdLists, IdListsBuilder};
use super::trie::Trie;

pub struct WordMap {
    trie: Trie,
    id_lists: IdLists,
}

impl WordMap {
    pub fn from_iter<I, W>(words: I) -> Self
    where
        I: IntoIterator<Item = W>,
        W: AsRef<str>,
    {
        let mut b = WordMapBuilder::new();
        for (i, w) in words.into_iter().enumerate() {
            b.add_record(w.as_ref().to_owned(), i as u32);
        }
        b.build()
    }

    pub fn common_prefix_iterator<'a>(
        &'a self,
        input: &'a [u8],
    ) -> impl Iterator<Item = WordMapMatch> + 'a {
        self.trie.common_prefix_iterator(input).flat_map(move |e| {
            self.id_lists
                .get(e.value as usize)
                .iter()
                .map(move |wi| WordMapMatch::new(*wi, e.end_byte))
        })
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct WordMapMatch {
    pub word_id: u32,
    pub end_byte: u32,
}

impl WordMapMatch {
    pub fn new(word_id: u32, end_byte: u32) -> Self {
        Self { word_id, end_byte }
    }
}

#[derive(Default)]
pub struct WordMapBuilder {
    map: BTreeMap<String, Vec<u32>>,
}

impl WordMapBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_record(&mut self, word: String, id: u32) {
        self.map.entry(word).or_default().push(id);
    }

    pub fn build(self) -> WordMap {
        let mut entries = vec![];
        let mut ilb = IdListsBuilder::new();
        for (word, ids) in self.map {
            let offset = ilb.push(&ids);
            entries.push((word, offset as u32));
        }
        let trie_data = yada::builder::DoubleArrayBuilder::build(&entries).unwrap();
        WordMap {
            trie: Trie::new(trie_data),
            id_lists: ilb.build(),
        }
    }
}
