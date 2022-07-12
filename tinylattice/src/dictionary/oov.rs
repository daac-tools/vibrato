pub mod simple;

pub use simple::SimpleOovGenerator;

use crate::dictionary::{WordIdx, WordParam};

pub struct OovWord {
    word_len: u16,
    word_idx: WordIdx,
    word_param: WordParam,
}

impl OovWord {
    #[inline(always)]
    pub fn word_len(&self) -> usize {
        self.word_len as usize
    }

    #[inline(always)]
    pub fn word_idx(&self) -> WordIdx {
        self.word_idx
    }

    #[inline(always)]
    pub fn word_param(&self) -> WordParam {
        self.word_param
    }
}
