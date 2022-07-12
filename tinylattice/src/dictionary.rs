pub mod category;
pub mod connector;
pub mod lexicon;
pub mod oov;

pub use category::{CategoryMap, CategoryTypes};
pub use connector::Connector;
pub use lexicon::{Lexicon, WordParam};
pub use oov::SimpleOovGenerator;

pub struct Dictionary {
    lexicon: Lexicon,
    connector: Connector,
    category_map: CategoryMap,
    simple_oov: Option<SimpleOovGenerator>,
}

impl Dictionary {
    pub fn new(
        lexicon: Lexicon,
        connector: Connector,
        category_map: CategoryMap,
        simple_oov: Option<SimpleOovGenerator>,
    ) -> Self {
        Self {
            lexicon,
            connector,
            category_map,
            simple_oov,
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
    pub fn simple_oov_generator(&self) -> Option<&SimpleOovGenerator> {
        self.simple_oov.as_ref()
    }
}
