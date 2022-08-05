use bincode::{Decode, Encode};

use super::ConnIdMapper;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Decode, Encode)]
pub struct WordParam {
    pub left_id: u16,
    pub right_id: u16,
    pub word_cost: i16,
}

impl WordParam {
    #[inline(always)]
    pub const fn new(left_id: u16, right_id: u16, word_cost: i16) -> Self {
        Self {
            left_id,
            right_id,
            word_cost,
        }
    }
}

#[derive(Decode, Encode)]
pub struct WordParams {
    params: Vec<WordParam>,
}

impl WordParams {
    pub fn new<I>(params: I) -> Self
    where
        I: IntoIterator<Item = WordParam>,
    {
        Self {
            params: params.into_iter().collect(),
        }
    }

    #[inline(always)]
    pub fn param(&self, word_id: usize) -> WordParam {
        self.params[word_id]
    }

    pub fn do_mapping(&mut self, mapper: &ConnIdMapper) {
        for p in &mut self.params {
            p.left_id = mapper.left(p.left_id);
            p.right_id = mapper.right(p.right_id);
        }
    }
}
