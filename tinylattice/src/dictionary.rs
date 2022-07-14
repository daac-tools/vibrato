use anyhow::Result;

pub mod category;
pub mod connector;
pub mod lexicon;
pub mod unknown;

pub use category::{CategoryMap, CategoryTypes};
pub use connector::Connector;
pub use lexicon::{Lexicon, WordParam};
pub use unknown::simple::SimpleUnkHandler;

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
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

pub struct Dictionary {
    lexicon: Lexicon,
    connector: Connector,
    category_map: CategoryMap,
    unk_handler: SimpleUnkHandler,
}

impl Dictionary {
    pub fn from_lines<I, S>(lex_csv: I, matrix_def: I, char_def: I, unk_def: I) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let lexicon = Lexicon::from_lines(lex_csv, LexType::System)?;
        let connector = Connector::from_lines(matrix_def)?;
        let category_map = CategoryMap::from_lines(char_def)?;
        let unk_handler = SimpleUnkHandler::from_lines(unk_def)?;
        Ok(Self {
            lexicon,
            connector,
            category_map,
            unk_handler,
        })
    }

    pub fn new(
        lexicon: Lexicon,
        connector: Connector,
        category_map: CategoryMap,
        unk_handler: SimpleUnkHandler,
    ) -> Self {
        Self {
            lexicon,
            connector,
            category_map,
            unk_handler,
        }
    }

    #[inline(always)]
    pub fn lexicon(&self) -> &Lexicon {
        &self.lexicon
    }

    #[inline(always)]
    pub fn connector(&self) -> &Connector {
        &self.connector
    }

    #[inline(always)]
    pub fn category_map(&self) -> &CategoryMap {
        &self.category_map
    }

    #[inline(always)]
    pub fn unk_handler(&self) -> &SimpleUnkHandler {
        &self.unk_handler
    }
}
