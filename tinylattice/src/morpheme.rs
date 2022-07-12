use std::ops::Range;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Morpheme {
    pub(crate) byte_begin: u16,
    pub(crate) byte_end: u16,
    pub(crate) char_begin: u16,
    pub(crate) char_end: u16,
    pub(crate) word_id: u32,
    pub(crate) total_cost: i32,
}

impl Morpheme {
    #[inline(always)]
    pub fn byte_range(&self) -> Range<usize> {
        usize::from(self.byte_begin)..usize::from(self.byte_end)
    }

    #[inline(always)]
    pub fn char_range(&self) -> Range<usize> {
        usize::from(self.char_begin)..usize::from(self.char_end)
    }

    #[inline(always)]
    pub fn word_id(&self) -> u32 {
        self.word_id
    }

    #[inline(always)]
    pub fn total_cost(&self) -> i32 {
        self.total_cost
    }
}
