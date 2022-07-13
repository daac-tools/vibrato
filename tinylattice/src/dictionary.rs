pub mod category;
pub mod connector;
pub mod lexicon;
pub mod oov;
pub mod unknown;

pub use category::{CategoryMap, CategoryTypes};
pub use connector::Connector;
pub use lexicon::{Lexicon, WordParam};
pub use oov::SimpleOovProvider;

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct WordIdx {
    lex_id: u32,
    word_id: u32,
}

impl Default for WordIdx {
    fn default() -> Self {
        Self::new(u32::MAX, u32::MAX)
    }
}

impl WordIdx {
    #[inline(always)]
    pub const fn new(lex_id: u32, word_id: u32) -> Self {
        Self { lex_id, word_id }
    }

    #[inline(always)]
    pub const fn lex_id(&self) -> u32 {
        self.lex_id
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
    simple_oov_provider: Option<SimpleOovProvider>,
}

impl Dictionary {
    pub fn new(
        lexicon: Lexicon,
        connector: Connector,
        category_map: CategoryMap,
        simple_oov_provider: Option<SimpleOovProvider>,
    ) -> Self {
        Self {
            lexicon,
            connector,
            category_map,
            simple_oov_provider,
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
    pub fn simple_oov_provider(&self) -> Option<&SimpleOovProvider> {
        self.simple_oov_provider.as_ref()
    }
}
