use anyhow::Result;

use super::{UnkEntry, UnkHandler, UnkWord};
use crate::dictionary::CategoryTypes;
use crate::Sentence;

pub struct SimpleUnkHandler {
    entry: UnkEntry,
}

impl SimpleUnkHandler {
    pub fn new(entry: UnkEntry) -> Self {
        Self { entry }
    }

    // tmp
    pub fn from_lines<I, L>(unk_def: I) -> Result<Self>
    where
        I: IntoIterator<Item = L>,
        L: AsRef<str>,
    {
        let unk_entries = UnkHandler::parse_unk_def(unk_def)?;
        assert_eq!(unk_entries[0].cate_type, CategoryTypes::DEFAULT);
        Ok(Self {
            entry: unk_entries[0].clone(),
        })
    }

    pub fn unk_words(&self, sentence: &Sentence, pos_char: usize) -> Vec<UnkWord> {
        let len = sentence.get_word_candidate_length(pos_char);
        vec![UnkWord {
            begin_char: pos_char as u16,
            end_char: (pos_char + len) as u16,
            left_id: self.entry.left_id,
            right_id: self.entry.right_id,
            word_cost: self.entry.word_cost,
            word_id: 0, // will be not used
        }]
    }
}
