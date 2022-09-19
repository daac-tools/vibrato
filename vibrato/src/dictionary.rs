//! Dictionary for tokenization.
pub(crate) mod builder;
pub(crate) mod character;
pub(crate) mod connector;
pub(crate) mod lexicon;
pub(crate) mod mapper;
pub(crate) mod unknown;
pub(crate) mod word_idx;

use std::io::{Read, Write};

use bincode::{Decode, Encode};

use crate::common;
use crate::dictionary::character::CharProperty;
use crate::dictionary::connector::Connector;
use crate::dictionary::lexicon::Lexicon;
use crate::dictionary::mapper::ConnIdMapper;
use crate::dictionary::unknown::UnkHandler;
use crate::dictionary::word_idx::WordIdx;
use crate::errors::Result;

pub(crate) use crate::dictionary::lexicon::WordParam;

/// Type of a lexicon that contains the word.
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

/// Inner data of [`Dictionary`].
#[derive(Decode, Encode)]
pub(crate) struct DictionaryInner {
    system_lexicon: Lexicon,
    user_lexicon: Option<Lexicon>,
    connector: Connector,
    mapper: Option<ConnIdMapper>,
    char_prop: CharProperty,
    unk_handler: UnkHandler,
}

/// Dictionary for tokenization.
pub struct Dictionary {
    pub(crate) data: DictionaryInner,
    pub(crate) need_check: bool,
}

impl Dictionary {
    /// Gets the reference to the system lexicon.
    #[inline(always)]
    pub(crate) const fn system_lexicon(&self) -> &Lexicon {
        &self.data.system_lexicon
    }

    /// Gets the reference to the user lexicon.
    #[inline(always)]
    pub(crate) const fn user_lexicon(&self) -> Option<&Lexicon> {
        self.data.user_lexicon.as_ref()
    }

    /// Gets the reference to the connection matrix.
    #[inline(always)]
    pub(crate) const fn connector(&self) -> &Connector {
        &self.data.connector
    }

    /// Gets the reference to the mapper for connection ids.
    #[allow(dead_code)]
    #[inline(always)]
    pub(crate) const fn mapper(&self) -> Option<&ConnIdMapper> {
        self.data.mapper.as_ref()
    }

    /// Gets the reference to the character property.
    #[inline(always)]
    pub(crate) const fn char_prop(&self) -> &CharProperty {
        &self.data.char_prop
    }

    /// Gets the reference to the handler of unknown words.
    #[inline(always)]
    pub(crate) const fn unk_handler(&self) -> &UnkHandler {
        &self.data.unk_handler
    }

    /// Gets the word parameter.
    #[inline(always)]
    pub(crate) fn word_param(&self, word_idx: WordIdx) -> WordParam {
        match word_idx.lex_type {
            LexType::System => self.system_lexicon().word_param(word_idx),
            LexType::User => self.user_lexicon().unwrap().word_param(word_idx),
            LexType::Unknown => self.unk_handler().word_param(word_idx),
        }
    }

    /// Gets the reference to the feature string.
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
            bincode::encode_into_std_write(&self.data, &mut wtr, common::bincode_config())?;
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
        Ok(Self {
            data,
            need_check: true,
        })
    }

    /// Creates a dictionary from a reader.
    ///
    /// # Safety
    ///
    /// The given reader must be a correct file exported by
    /// [`Dictionary::write()`].
    ///
    /// # Errors
    ///
    /// When bincode generates an error, it will be returned as is.
    pub unsafe fn read_unchecked<R>(mut rdr: R) -> Result<Self>
    where
        R: Read,
    {
        let data = bincode::decode_from_std_read(&mut rdr, common::bincode_config())?;
        Ok(Self {
            data,
            need_check: false,
        })
    }
}
