use super::OovWord;
use crate::dictionary::{WordIdx, WordParam};
use crate::sentence::Sentence;

pub struct SimpleOovProvider {
    word_idx: WordIdx,
    word_param: WordParam,
    word_feat: String,
}

impl SimpleOovProvider {
    pub fn new(lex_id: u32, word_param: WordParam, word_feat: String) -> Self {
        Self {
            word_idx: WordIdx::new(lex_id, 0),
            word_param,
            word_feat,
        }
    }

    #[inline(always)]
    pub fn oov_word(&self, sent: &Sentence, char_pos: usize) -> OovWord {
        let oov_len = sent.get_word_candidate_length(char_pos);
        OovWord {
            word_idx: self.word_idx,
            word_len: oov_len as u16,
            word_param: self.word_param,
        }
    }

    #[inline(always)]
    pub fn word_feature(&self, word_idx: WordIdx) -> &str {
        debug_assert_eq!(word_idx, self.word_idx);
        &self.word_feat
    }
}
