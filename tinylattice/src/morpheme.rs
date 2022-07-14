use std::ops::Range;

use crate::dictionary::WordIdx;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Morpheme {
    pub(crate) begin_byte: u16,
    pub(crate) end_byte: u16,
    pub(crate) begin_char: u16,
    pub(crate) end_char: u16,
    pub(crate) word_idx: WordIdx,
    pub(crate) total_cost: i32,
}

impl Morpheme {
    #[inline(always)]
    pub fn range_byte(&self) -> Range<usize> {
        usize::from(self.begin_byte)..usize::from(self.end_byte)
    }

    #[inline(always)]
    pub fn range_char(&self) -> Range<usize> {
        usize::from(self.begin_char)..usize::from(self.end_char)
    }

    #[inline(always)]
    pub fn word_idx(&self) -> WordIdx {
        self.word_idx
    }

    #[inline(always)]
    pub fn total_cost(&self) -> i32 {
        self.total_cost
    }
}
