use super::OovWord;
use crate::dictionary::lexicon::WordParam;
use crate::sentence::Sentence;

pub struct SimpleOovGenerator {
    word_id: u32,
    word_param: WordParam,
}

impl SimpleOovGenerator {
    pub fn new(word_id: u32, word_param: WordParam) -> Self {
        Self {
            word_id,
            word_param,
        }
    }

    pub fn gen_oov_word(&self, sent: &Sentence, char_pos: usize) -> OovWord {
        let oov_len = sent.get_word_candidate_length(char_pos);
        OovWord {
            word_id: self.word_id,
            word_len: oov_len as u16,
            word_param: self.word_param,
        }
    }
}