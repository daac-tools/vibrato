pub mod posting;
pub mod trie;

use std::collections::BTreeMap;

use anyhow::Result;

use posting::{Postings, PostingsBuilder};
use trie::Trie;

pub struct WordMap {
    trie: Trie,
    postings: Postings,
}

impl WordMap {
    pub fn new<I, W>(words: I) -> Result<Self>
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

    #[inline(always)]
    pub fn common_prefix_iterator<'a>(
        &'a self,
        input: &'a [u8],
    ) -> impl Iterator<Item = (u32, u32)> + 'a {
        unsafe {
            self.trie.common_prefix_iterator(input).flat_map(move |e| {
                self.postings
                    .ids(e.value as usize)
                    .map(move |word_id| (word_id, e.end_byte))
            })
        }
    }
}

#[derive(Default)]
pub struct WordMapBuilder {
    map: BTreeMap<String, Vec<u32>>,
}

impl WordMapBuilder {
    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline(always)]
    pub fn add_record(&mut self, word: String, id: u32) {
        self.map.entry(word).or_default().push(id);
    }

    pub fn build(self) -> Result<WordMap> {
        let mut entries = vec![];
        let mut builder = PostingsBuilder::new();
        for (word, ids) in self.map {
            let offset = builder.push(&ids)?;
            entries.push((word, offset as u32));
        }
        let trie_data = yada::builder::DoubleArrayBuilder::build(&entries).unwrap();
        Ok(WordMap {
            trie: Trie::new(trie_data),
            postings: builder.build(),
        })
    }
}
