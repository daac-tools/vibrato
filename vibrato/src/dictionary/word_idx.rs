use crate::dictionary::LexType;

/// Identifier of a word.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct WordIdx {
    pub lex_type: LexType,
    pub word_id: u32,
}

impl Default for WordIdx {
    fn default() -> Self {
        Self::new(LexType::System, u32::MAX)
    }
}

impl WordIdx {
    /// Creates a new instance.
    #[inline(always)]
    pub const fn new(lex_type: LexType, word_id: u32) -> Self {
        Self { lex_type, word_id }
    }
}
