use super::OovWord;
use crate::dictionary::{WordIdx, WordParam};
use crate::sentence::Sentence;

pub struct SimpleOovGenerator {
    word_idx: WordIdx,
    word_param: WordParam,
}

impl SimpleOovGenerator {
    pub fn new(word_idx: WordIdx, word_param: WordParam) -> Self {
        Self {
            word_idx,
            word_param,
        }
    }

    pub fn gen_oov_word(&self, sent: &Sentence, char_pos: usize) -> OovWord {
        let oov_len = sent.get_word_candidate_length(char_pos);
        OovWord {
            word_idx: self.word_idx,
            word_len: oov_len as u16,
            word_param: self.word_param,
        }
    }
}
