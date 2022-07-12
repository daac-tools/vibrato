pub mod simple;

pub use simple::SimpleOovGenerator;

use crate::dictionary::lexicon::WordParam;

pub struct OovWord {
    word_id: u32,
    word_len: u16,
    word_param: WordParam,
}

impl OovWord {
    pub fn word_id(&self) -> u32 {
        self.word_id
    }

    pub fn word_len(&self) -> usize {
        self.word_len as usize
    }

    pub fn word_param(&self) -> WordParam {
        self.word_param
    }
}
