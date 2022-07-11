#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Morpheme {
    pub(crate) begin_byte: usize,
    pub(crate) end_byte: usize,
    pub(crate) begin_char: usize,
    pub(crate) end_char: usize,
    pub(crate) word_id: u32,
    pub(crate) total_cost: i32,
}

impl Morpheme {
    #[inline(always)]
    pub fn begin_byte(&self) -> usize {
        self.begin_byte
    }

    #[inline(always)]
    pub fn end_byte(&self) -> usize {
        self.end_byte
    }

    #[inline(always)]
    pub fn begin_char(&self) -> usize {
        self.begin_char
    }

    #[inline(always)]
    pub fn end_char(&self) -> usize {
        self.end_char
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
