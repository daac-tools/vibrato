//! Dictionary for tokenization.
pub(crate) mod character;
pub(crate) mod connector;
pub(crate) mod lexicon;
pub(crate) mod mapper;
pub(crate) mod unknown;

use std::io::{Read, Write};

use bincode::{Decode, Encode};

use crate::common;
use crate::errors::Result;

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

/// Inner data of [`Dictionary`].
#[derive(Decode, Encode)]
struct DictionaryInner {
    system_lexicon: Lexicon,
    user_lexicon: Option<Lexicon>,
    connector: Connector,
    mapper: Option<ConnIdMapper>,
    char_prop: CharProperty,
    unk_handler: UnkHandler,
}

/// Dictionary for tokenization.
pub struct Dictionary(DictionaryInner);

impl Dictionary {
    /// Creates a new instance.
    pub const fn new(
        system_lexicon: Lexicon,
        user_lexicon: Option<Lexicon>,
        connector: Connector,
        char_prop: CharProperty,
        unk_handler: UnkHandler,
    ) -> Self {
        Self(DictionaryInner {
            system_lexicon,
            user_lexicon,
            connector,
            mapper: None,
            char_prop,
            unk_handler,
        })
    }

    /// Gets the reference to the system lexicon.
    #[inline(always)]
    pub const fn system_lexicon(&self) -> &Lexicon {
        &self.0.system_lexicon
    }

    /// Gets the reference to the user lexicon.
    #[inline(always)]
    pub const fn user_lexicon(&self) -> Option<&Lexicon> {
        self.0.user_lexicon.as_ref()
    }

    /// Resets the user lexicon.
    #[inline(always)]
    pub fn reset_user_lexicon(&mut self, user_lexicon: Option<Lexicon>) {
        self.0.user_lexicon = user_lexicon;
        if let Some(user_lexicon) = self.0.user_lexicon.as_mut() {
            if let Some(mapper) = self.0.mapper.as_ref() {
                user_lexicon.do_mapping(mapper);
            }
        }
    }

    /// Gets the reference to the connection matrix.
    #[inline(always)]
    pub const fn connector(&self) -> &Connector {
        &self.0.connector
    }

    /// Gets the reference to the mapper for connection ids.
    #[inline(always)]
    pub const fn mapper(&self) -> Option<&ConnIdMapper> {
        self.0.mapper.as_ref()
    }

    /// Gets the reference to the character property.
    #[inline(always)]
    pub const fn char_prop(&self) -> &CharProperty {
        &self.0.char_prop
    }

    /// Gets the reference to the handler of unknown words.
    #[inline(always)]
    pub const fn unk_handler(&self) -> &UnkHandler {
        &self.0.unk_handler
    }

    /// Edits connection ids with the given mapping.
    pub fn do_mapping(&mut self, mapper: ConnIdMapper) {
        self.0.system_lexicon.do_mapping(&mapper);
        if let Some(user_lexicon) = self.0.user_lexicon.as_mut() {
            user_lexicon.do_mapping(&mapper);
        }
        self.0.connector.do_mapping(&mapper);
        self.0.unk_handler.do_mapping(&mapper);
        self.0.mapper = Some(mapper);
    }

    #[inline(always)]
    pub(crate) fn word_feature(&self, word_idx: WordIdx) -> &str {
        match word_idx.lex_type {
            LexType::System => self.system_lexicon().word_feature(word_idx),
            LexType::User => self.user_lexicon().unwrap().word_feature(word_idx),
            LexType::Unknown => self.unk_handler().word_feature(word_idx),
        }
    }

    /// Exports the dictionary data.
    ///
    /// # Errors
    ///
    /// When bincode generates an error, it will be returned as is.
    pub fn write<W>(&self, mut wtr: W) -> Result<usize>
    where
        W: Write,
    {
        let num_bytes =
            bincode::encode_into_std_write(&self.0, &mut wtr, common::bincode_config())?;
        Ok(num_bytes)
    }

    /// Creates a dictionary from a reader.
    ///
    /// # Errors
    ///
    /// When bincode generates an error, it will be returned as is.
    pub fn read<R>(mut rdr: R) -> Result<Self>
    where
        R: Read,
    {
        let data = bincode::decode_from_std_read(&mut rdr, common::bincode_config())?;
        Ok(Self(data))
    }
}
