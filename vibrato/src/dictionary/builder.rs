use std::io::Read;

use crate::dictionary::connector::{MatrixConnector, RawConnector};
use crate::dictionary::{
    CharProperty, ConnIdMapper, Connector, ConnectorWrapper, Dictionary, DictionaryInner, LexType,
    Lexicon, UnkHandler,
};
use crate::errors::{Result, VibratoError};

use super::lexicon::RawWordEntry;

impl Dictionary {
    pub(crate) fn new(
        system_word_entries: &[RawWordEntry],
        connector: ConnectorWrapper,
        char_prop: CharProperty,
        unk_handler: UnkHandler,
    ) -> Result<Self> {
        let system_lexicon = Lexicon::from_entries(system_word_entries, LexType::System)?;

        if !system_lexicon.verify(&connector) {
            return Err(VibratoError::invalid_argument(
                "system_lexicon_rdr",
                "system_lexicon_rdr includes invalid connection ids.",
            ));
        }
        if !unk_handler.verify(&connector) {
            return Err(VibratoError::invalid_argument(
                "unk_handler_rdr",
                "unk_handler_rdr includes invalid connection ids.",
            ));
        }

        Ok(Self {
            data: DictionaryInner {
                system_lexicon,
                user_lexicon: None,
                connector,
                mapper: None,
                char_prop,
                unk_handler,
            },
            need_check: false,
        })
    }

    /// Creates a new instance from readers in the MeCab format.
    ///
    /// # Arguments
    ///
    ///  - `system_lexicon_rdr`: A reader of a lexicon file `*.csv`.
    ///  - `connector_rdr`: A reader of matrix file `matrix.def`.
    ///  - `char_prop_rdr`: A reader of character definition file `char.def`.
    ///  - `unk_handler`: A reader of unknown definition file `unk.def`.
    ///
    /// # Errors
    ///
    /// [`VibratoError`] is returned when an input format is invalid.
    pub fn from_readers<S, C, P, U>(
        mut system_lexicon_rdr: S,
        connector_rdr: C,
        char_prop_rdr: P,
        unk_handler_rdr: U,
    ) -> Result<Self>
    where
        S: Read,
        C: Read,
        P: Read,
        U: Read,
    {
        let mut system_lexicon_buf = vec![];
        system_lexicon_rdr.read_to_end(&mut system_lexicon_buf)?;
        let system_word_entries = Lexicon::parse_csv(&system_lexicon_buf, "lex.csv")?;
        let connector = MatrixConnector::from_reader(connector_rdr)?;
        let char_prop = CharProperty::from_reader(char_prop_rdr)?;
        let unk_handler = UnkHandler::from_reader(unk_handler_rdr, &char_prop)?;

        Self::new(
            &system_word_entries,
            ConnectorWrapper::Matrix(connector),
            char_prop,
            unk_handler,
        )
    }

    /// Creates a new instance from readers with the detailed bi-gram information.
    ///
    /// # Arguments
    ///
    ///  - `system_lexicon_rdr`: A reader of a lexicon file `*.csv`.
    ///  - `bigram_right_rdr`: A reader of bi-gram info associated with right IDs `bigram.right`.
    ///  - `bigram_left_rdr`: A reader of bi-gram info associated with left IDs `bigram.left`.
    ///  - `bigram_cost_rdr`: A reader of a bi-gram cost file `bigram.cost`.
    ///  - `char_prop_rdr`: A reader of character definition file `char.def`.
    ///  - `unk_handler`: A reader of unknown definition file `unk.def`.
    ///
    /// # Errors
    ///
    /// [`VibratoError`] is returned when an input format is invalid.
    pub fn from_readers_with_bigram_info<S, R, L, C, P, U>(
        mut system_lexicon_rdr: S,
        bigram_right_rdr: R,
        bigram_left_rdr: L,
        bigram_cost_rdr: C,
        char_prop_rdr: P,
        unk_handler_rdr: U,
    ) -> Result<Self>
    where
        S: Read,
        R: Read,
        L: Read,
        C: Read,
        P: Read,
        U: Read,
    {
        let mut system_lexicon_buf = vec![];
        system_lexicon_rdr.read_to_end(&mut system_lexicon_buf)?;
        let system_word_entries = Lexicon::parse_csv(&system_lexicon_buf, "lex.csv")?;
        let connector =
            RawConnector::from_readers(bigram_right_rdr, bigram_left_rdr, bigram_cost_rdr)?;
        let char_prop = CharProperty::from_reader(char_prop_rdr)?;
        let unk_handler = UnkHandler::from_reader(unk_handler_rdr, &char_prop)?;

        Self::new(
            &system_word_entries,
            ConnectorWrapper::Raw(connector),
            char_prop,
            unk_handler,
        )
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
    pub fn user_lexicon_from_reader<R>(mut self, user_lexicon_rdr: Option<R>) -> Result<Self>
    where
        R: Read,
    {
        if let Some(user_lexicon_rdr) = user_lexicon_rdr {
            let mut user_lexicon = Lexicon::from_reader(user_lexicon_rdr, LexType::User)?;
            if let Some(mapper) = self.data.mapper.as_ref() {
                user_lexicon.do_mapping(mapper);
            }
            if !user_lexicon.verify(self.connector()) {
                return Err(VibratoError::invalid_argument(
                    "user_lexicon_rdr",
                    "user_lexicon_rdr includes invalid connection ids.",
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
    pub fn mapping_from_iter<L, R>(mut self, lmap: L, rmap: R) -> Result<Self>
    where
        L: IntoIterator<Item = u16>,
        R: IntoIterator<Item = u16>,
    {
        let mapper = ConnIdMapper::from_iter(lmap, rmap)?;
        self.data.system_lexicon.do_mapping(&mapper);
        if let Some(user_lexicon) = self.data.user_lexicon.as_mut() {
            user_lexicon.do_mapping(&mapper);
        }
        self.data.connector.do_mapping(&mapper);
        self.data.unk_handler.do_mapping(&mapper);
        self.data.mapper = Some(mapper);
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oor_lex() {
        let lexicon_csv = "自然,1,1,0";
        let matrix_def = "1 1\n0 0 0";
        let char_def = "DEFAULT 0 1 0";
        let unk_def = "DEFAULT,0,0,100,*";

        let result = Dictionary::from_readers(
            lexicon_csv.as_bytes(),
            matrix_def.as_bytes(),
            char_def.as_bytes(),
            unk_def.as_bytes(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_oor_unk() {
        let lexicon_csv = "自然,0,0,0";
        let matrix_def = "1 1\n0 0 0";
        let char_def = "DEFAULT 0 1 0";
        let unk_def = "DEFAULT,1,1,100,*";

        let result = Dictionary::from_readers(
            lexicon_csv.as_bytes(),
            matrix_def.as_bytes(),
            char_def.as_bytes(),
            unk_def.as_bytes(),
        );

        assert!(result.is_err());
    }
}
