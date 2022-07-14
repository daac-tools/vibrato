use super::{UnkEntry, UnkWord};
use crate::Sentence;

pub struct SimpleUnkHandler {
    entry: UnkEntry,
}

impl SimpleUnkHandler {
    pub fn new(entry: UnkEntry) -> Self {
        Self { entry }
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
