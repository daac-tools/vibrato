//! Dictionary for tokenization.
pub mod builder;
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
use crate::dictionary::connector::{Connector, ConnectorWrapper};
use crate::dictionary::lexicon::Lexicon;
use crate::dictionary::mapper::ConnIdMapper;
use crate::dictionary::unknown::UnkHandler;
use crate::errors::{Result, VibratoError};

pub use crate::dictionary::builder::SystemDictionaryBuilder;
pub use crate::dictionary::word_idx::WordIdx;

pub(crate) use crate::dictionary::lexicon::WordParam;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const MAGIC_LEN: usize = 20;

/// Returns the magic number for model.
fn model_magic() -> [u8; MAGIC_LEN] {
    let magic_str = format!("Vibrato {VERSION}");
    let magic_bytes = magic_str.as_bytes();
    debug_assert!(magic_bytes.len() <= MAGIC_LEN);
    let mut magic_number = [0u8; MAGIC_LEN];
    magic_number[..magic_bytes.len()].copy_from_slice(magic_bytes);
    magic_number
}

/// Type of a lexicon that contains the word.
#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash, Decode, Encode)]
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
    connector: ConnectorWrapper,
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
    pub(crate) const fn connector(&self) -> &ConnectorWrapper {
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
    pub fn word_feature(&self, word_idx: WordIdx) -> &str {
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
        wtr.write_all(&model_magic())?;
        let config = common::bincode_config();
        let num_bytes = bincode::encode_into_std_write(&self.data, &mut wtr, config)?;
        Ok(MAGIC_LEN + num_bytes)
    }

    /// Creates a dictionary from a reader.
    ///
    /// # Errors
    ///
    /// When bincode generates an error, it will be returned as is.
    pub fn read<R>(rdr: R) -> Result<Self>
    where
        R: Read,
    {
        Ok(Self {
            data: Self::read_data(rdr)?,
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
    pub unsafe fn read_unchecked<R>(rdr: R) -> Result<Self>
    where
        R: Read,
    {
        Ok(Self {
            data: Self::read_data(rdr)?,
            need_check: false,
        })
    }

    fn read_data<R>(mut rdr: R) -> Result<DictionaryInner>
    where
        R: Read,
    {
        let mut magic = [0u8; MAGIC_LEN];
        rdr.read_exact(&mut magic)?;
        if magic != model_magic() {
            return Err(VibratoError::invalid_argument(
                "rdr",
                "The magic number of the input model mismatches.",
            ));
        }
        let config = common::bincode_config();
        let data = bincode::decode_from_std_read(&mut rdr, config)?;
        Ok(data)
    }

    /// Resets the user dictionary from a reader.
    ///
    /// # Arguments
    ///
    ///  - `user_lexicon_rdr`: A reader of a lexicon file `*.csv` in the MeCab format.
    ///                        If `None`, clear the current user dictionary.
    ///
    /// # Errors
    ///
    /// [`VibratoError`] is returned when an input format is invalid.
    pub fn reset_user_lexicon_from_reader<R>(mut self, user_lexicon_rdr: Option<R>) -> Result<Self>
    where
        R: Read,
    {
        if let Some(user_lexicon_rdr) = user_lexicon_rdr {
            let mut user_lexicon = Lexicon::from_reader(user_lexicon_rdr, LexType::User)?;
            if let Some(mapper) = self.data.mapper.as_ref() {
                user_lexicon.map_connection_ids(mapper);
            }
            if !user_lexicon.verify(self.connector()) {
                return Err(VibratoError::invalid_argument(
                    "user_lexicon_rdr",
                    "includes invalid connection ids.",
                ));
            }
            self.data.user_lexicon = Some(user_lexicon);
        } else {
            self.data.user_lexicon = None;
        }
        Ok(self)
    }

    /// Edits connection ids with the given mappings.
    ///
    /// # Arguments
    ///
    ///  - `lmap/rmap`: An iterator of mappings of left/right ids, where
    ///                 the `i`-th item (1-origin) indicates a new id mapped from id `i`.
    ///
    /// # Errors
    ///
    /// [`VibratoError`] is returned when
    ///  - a new id of [`BOS_EOS_CONNECTION_ID`](crate::common::BOS_EOS_CONNECTION_ID)
    ///    is included,
    ///  - new ids are duplicated, or
    ///  - the set of new ids are not same as that of old ids.
    pub fn map_connection_ids_from_iter<L, R>(mut self, lmap: L, rmap: R) -> Result<Self>
    where
        L: IntoIterator<Item = u16>,
        R: IntoIterator<Item = u16>,
    {
        let mapper = ConnIdMapper::from_iter(lmap, rmap)?;
        self.data.system_lexicon.map_connection_ids(&mapper);
        if let Some(user_lexicon) = self.data.user_lexicon.as_mut() {
            user_lexicon.map_connection_ids(&mapper);
        }
        self.data.connector.map_connection_ids(&mapper);
        self.data.unk_handler.map_connection_ids(&mapper);
        self.data.mapper = Some(mapper);
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_magic() {
        // Checks if it does not panic.
        model_magic();
    }
}
