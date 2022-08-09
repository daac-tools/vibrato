use std::io::Read;

use crate::dictionary::{
    CharProperty, ConnIdMapper, Connector, Dictionary, DictionaryInner, LexType, Lexicon,
    UnkHandler,
};
use crate::errors::{Result, VibratoError};

impl Dictionary {
    /// Creates a new instance from readers in the MeCab format.
    ///
    /// # Arguments
    ///
    ///  - `system_lexicon_rdr`: A reader of a lexicon file `*.csv`.
    ///  - `connector_rdr`: A reader of matrix file `matrix.def`.
    ///  - `char_prop_rdr`: A reader of character definition file `char.def`.
    ///  - `unk_handler`: A reader of unknown definition file `unk.def`.
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
    ///
    /// # Arguments
    ///
    ///  - `user_lexicon_rdr`: A reader of a lexicon file `*.csv` in the MeCab format.
    ///                        If `None`, clear the current user dictionary.
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
    ///
    /// # Format
    ///
    /// Mappings are written line by line.
    /// The `i`-th line (1-origin) indicates a mapping from id `i`.
    /// If a file is in the tsv format, only the first column is evaluated.
    /// Since id zero is fixed for BOS/EOS, a mapping to zero must not be incldeuded.
    ///
    /// # Examples
    ///
    /// The following text indicates the mapping
    /// `(1,2,3,4) -> (2,3,4,1)`.
    ///
    /// ```text
    /// 2
    /// 3
    /// 4
    /// 1
    /// ```
    ///
    /// # Arguments
    ///
    ///  - `l_rdr`: A reader of mappings of left ids.
    ///  - `r_rdr`: A reader of mappings of right ids.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_oor_lex() {
        let lexicon_csv = "自然,1,1,0";
        let matrix_def = "1 1\n0 0 0";
        let char_def = "DEFAULT 0 1 0";
        let unk_def = "DEFAULT,0,0,100,*";

        Dictionary::from_reader(
            lexicon_csv.as_bytes(),
            matrix_def.as_bytes(),
            char_def.as_bytes(),
            unk_def.as_bytes(),
        )
        .unwrap();
    }

    #[test]
    #[should_panic]
    fn test_oor_unk() {
        let lexicon_csv = "自然,0,0,0";
        let matrix_def = "1 1\n0 0 0";
        let char_def = "DEFAULT 0 1 0";
        let unk_def = "DEFAULT,1,1,100,*";

        Dictionary::from_reader(
            lexicon_csv.as_bytes(),
            matrix_def.as_bytes(),
            char_def.as_bytes(),
            unk_def.as_bytes(),
        )
        .unwrap();
    }
}
