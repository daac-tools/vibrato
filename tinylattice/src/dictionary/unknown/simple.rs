use super::{UnkEntry, UnkWord};
use crate::Sentence;

pub struct SimpleUnkHandler {
    entry: UnkEntry,
}

impl SimpleUnkHandler {
    pub fn new(entry: UnkEntry) -> Self {
        Self { entry }
    }

    pub fn unk_words(&self, sentence: &Sentence, char_pos: usize) -> Vec<UnkWord> {
        let len = sentence.get_word_candidate_length(char_pos);
        vec![UnkWord {
            char_begin: char_pos as u16,
            char_end: (char_pos + len) as u16,
            left_id: self.entry.left_id,
            right_id: self.entry.right_id,
            word_cost: self.entry.word_cost,
            word_id: 0, // will be not used
        }]
    }
}
