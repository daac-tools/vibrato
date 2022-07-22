pub mod character;
pub mod connector;
pub mod lexicon;
pub mod mapper;
pub mod unknown;

use bincode::{Decode, Encode};

pub use character::CharProperty;
pub use connector::Connector;
pub use lexicon::{Lexicon, WordParam};
pub use mapper::ConnIdMapper;
pub use unknown::UnkHandler;

#[derive(Clone, Copy, Eq, PartialEq, Debug, Decode, Encode)]
pub enum LexType {
    System,
    Unknown,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct WordIdx {
    lex_type: LexType,
    word_id: u32,
}

impl Default for WordIdx {
    fn default() -> Self {
        Self::new(LexType::System, u32::MAX)
    }
}

impl WordIdx {
    #[inline(always)]
    pub const fn new(lex_type: LexType, word_id: u32) -> Self {
        Self { lex_type, word_id }
    }

    #[inline(always)]
    pub const fn lex_type(&self) -> LexType {
        self.lex_type
    }

    #[inline(always)]
    pub const fn word_id(&self) -> u32 {
        self.word_id
    }
}

#[derive(Decode, Encode)]
pub struct Dictionary {
    lexicon: Lexicon,
    connector: Connector,
    char_prop: CharProperty,
    unk_handler: UnkHandler,
}

impl Dictionary {
    pub const fn new(
        lexicon: Lexicon,
        connector: Connector,
        char_prop: CharProperty,
        unk_handler: UnkHandler,
    ) -> Self {
        Self {
            lexicon,
            connector,
            char_prop,
            unk_handler,
        }
    }

    #[inline(always)]
    pub const fn lexicon(&self) -> &Lexicon {
        &self.lexicon
    }

    #[inline(always)]
    pub const fn connector(&self) -> &Connector {
        &self.connector
    }

    #[inline(always)]
    pub const fn char_prop(&self) -> &CharProperty {
        &self.char_prop
    }

    #[inline(always)]
    pub const fn unk_handler(&self) -> &UnkHandler {
        &self.unk_handler
    }

    #[inline(always)]
    pub fn word_feature(&self, word_idx: WordIdx) -> &str {
        match word_idx.lex_type() {
            LexType::System => self.lexicon().word_feature(word_idx),
            LexType::Unknown => self.unk_handler().word_feature(word_idx),
        }
    }

    pub fn map_ids(&mut self, mapper: &ConnIdMapper) {
        self.lexicon.map_ids(mapper);
        self.connector.map_ids(mapper);
        self.unk_handler.map_ids(mapper);
    }
}
