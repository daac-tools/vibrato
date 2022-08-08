use std::io::Read;

use super::{
    CharProperty, ConnIdMapper, Connector, Dictionary, DictionaryInner, LexType, Lexicon,
    UnkHandler,
};
use crate::errors::{Result, VibratoError};

impl Dictionary {
    /// Creates a new instance from readers.
    ///
    /// # Arguments
    ///
    ///  - `system_lexicon_rdr`: A reader of file `lex.csv`.
    ///  - `connector_rdr`: A reader of file `matrix.def`.
    ///  - `char_prop_rdr`: A reader of file `char.def`.
    ///  - `unk_handler`: A reader of file `unk.def`.
    pub fn from_reader<S, C, P, U>(
        system_lexicon_rdr: S,
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
        let system_lexicon = Lexicon::from_reader(system_lexicon_rdr, LexType::System)?;
        let connector = Connector::from_reader(connector_rdr)?;
        let char_prop = CharProperty::from_reader(char_prop_rdr)?;
        let unk_handler = UnkHandler::from_reader(unk_handler_rdr)?;

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

        Ok(Self(DictionaryInner {
            system_lexicon,
            user_lexicon: None,
            connector,
            mapper: None,
            char_prop,
            unk_handler,
        }))
    }

    /// Resets the user dictionary from a reader.
    pub fn user_lexicon_from_reader<R>(mut self, user_lexicon_rdr: Option<R>) -> Result<Self>
    where
        R: Read,
    {
        if let Some(user_lexicon_rdr) = user_lexicon_rdr {
            let mut user_lexicon = Lexicon::from_reader(user_lexicon_rdr, LexType::User)?;
            if let Some(mapper) = self.0.mapper.as_ref() {
                user_lexicon.do_mapping(mapper);
            }
            if !user_lexicon.verify(self.connector()) {
                return Err(VibratoError::invalid_argument(
                    "user_lexicon_rdr",
                    "user_lexicon_rdr includes invalid connection ids.",
                ));
            }
            self.0.user_lexicon = Some(user_lexicon);
        } else {
            self.0.user_lexicon = None;
        }
        Ok(self)
    }

    /// Edits connection ids with the given mappings.
    pub fn mapping_from_reader<L, R>(mut self, l_rdr: L, r_rdr: R) -> Result<Self>
    where
        L: Read,
        R: Read,
    {
        let mapper = ConnIdMapper::from_reader(l_rdr, r_rdr)?;
        self.0.system_lexicon.do_mapping(&mapper);
        if let Some(user_lexicon) = self.0.user_lexicon.as_mut() {
            user_lexicon.do_mapping(&mapper);
        }
        self.0.connector.do_mapping(&mapper);
        self.0.unk_handler.do_mapping(&mapper);
        self.0.mapper = Some(mapper);
        Ok(self)
    }
}
