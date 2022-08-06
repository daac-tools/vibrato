//! Dictionary for tokenization.
pub(crate) mod character;
pub(crate) mod connector;
pub(crate) mod lexicon;
pub(crate) mod mapper;
pub(crate) mod unknown;

use bincode::{Decode, Encode};

pub use character::CharProperty;
pub use connector::Connector;
pub use lexicon::Lexicon;
pub use mapper::{ConnIdCounter, ConnIdMapper, ConnIdProbs};
pub use unknown::UnkHandler;

/// Type of lexicon that contains the word.
#[derive(Clone, Copy, Eq, PartialEq, Debug, Decode, Encode)]
#[repr(u8)]
pub enum LexType {
    /// System lexicon.
    System,
    /// User lexicon.
    User,
    /// Unknown words.
    Unknown,
}

impl Default for LexType {
    fn default() -> Self {
        Self::System
    }
}

/// Identifier of a word.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct WordIdx {
    pub(crate) lex_type: LexType,
    pub(crate) word_id: u32,
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

/// Dictionary for tokenization.
#[derive(Decode, Encode)]
pub struct Dictionary {
    system_lexicon: Lexicon,
    user_lexicon: Option<Lexicon>,
    connector: Connector,
    mapper: Option<ConnIdMapper>,
    char_prop: CharProperty,
    unk_handler: UnkHandler,
}

impl Dictionary {
    /// Creates a new instance.
    pub const fn new(
        system_lexicon: Lexicon,
        user_lexicon: Option<Lexicon>,
        connector: Connector,
        char_prop: CharProperty,
        unk_handler: UnkHandler,
    ) -> Self {
        Self {
            system_lexicon,
            user_lexicon,
            connector,
            mapper: None,
            char_prop,
            unk_handler,
        }
    }

    /// Gets the reference to the system lexicon.
    #[inline(always)]
    pub const fn system_lexicon(&self) -> &Lexicon {
        &self.system_lexicon
    }

    /// Gets the reference to the user lexicon.
    #[inline(always)]
    pub const fn user_lexicon(&self) -> Option<&Lexicon> {
        self.user_lexicon.as_ref()
    }

    /// Resets the user lexicon.
    #[inline(always)]
    pub fn reset_user_lexicon(&mut self, user_lexicon: Option<Lexicon>) {
        self.user_lexicon = user_lexicon;
        if let Some(user_lexicon) = self.user_lexicon.as_mut() {
            if let Some(mapper) = self.mapper.as_ref() {
                user_lexicon.do_mapping(mapper);
            }
        }
    }

    /// Gets the reference to the connection matrix.
    #[inline(always)]
    pub const fn connector(&self) -> &Connector {
        &self.connector
    }

    /// Gets the reference to the mapper for connection ids.
    #[inline(always)]
    pub const fn mapper(&self) -> Option<&ConnIdMapper> {
        self.mapper.as_ref()
    }

    /// Gets the reference to the character property.
    #[inline(always)]
    pub const fn char_prop(&self) -> &CharProperty {
        &self.char_prop
    }

    /// Gets the reference to the handler of unknown words.
    #[inline(always)]
    pub const fn unk_handler(&self) -> &UnkHandler {
        &self.unk_handler
    }

    /// Edits connection ids with the given mapping.
    pub fn do_mapping(&mut self, mapper: ConnIdMapper) {
        self.system_lexicon.do_mapping(&mapper);
        if let Some(user_lexicon) = self.user_lexicon.as_mut() {
            user_lexicon.do_mapping(&mapper);
        }
        self.connector.do_mapping(&mapper);
        self.unk_handler.do_mapping(&mapper);
        self.mapper = Some(mapper);
    }

    #[inline(always)]
    pub(crate) fn word_feature(&self, word_idx: WordIdx) -> &str {
        match word_idx.lex_type {
            LexType::System => self.system_lexicon().word_feature(word_idx),
            LexType::User => self.user_lexicon().unwrap().word_feature(word_idx),
            LexType::Unknown => self.unk_handler().word_feature(word_idx),
        }
    }
}
