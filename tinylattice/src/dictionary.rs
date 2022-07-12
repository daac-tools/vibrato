pub mod category;
pub mod connection;
pub mod lexicon;
pub mod oov;

pub use category::{CategoryMap, CategoryTypes};
pub use connection::ConnectionMatrix;
pub use lexicon::{Lexicon, WordParam};
pub use oov::SimpleOovGenerator;

pub struct Dictionary {
    lexicon: Lexicon,
    conn_matrix: ConnectionMatrix,
    cate_map: CategoryMap,
    simple_oov: Option<SimpleOovGenerator>,
}

impl Dictionary {
    pub fn new(
        lexicon: Lexicon,
        conn_matrix: ConnectionMatrix,
        cate_map: CategoryMap,
        simple_oov: Option<SimpleOovGenerator>,
    ) -> Self {
        Self {
            lexicon,
            conn_matrix,
            cate_map,
            simple_oov,
        }
    }

    pub fn lexicon(&self) -> &Lexicon {
        &self.lexicon
    }

    pub fn conn_matrix(&self) -> &ConnectionMatrix {
        &self.conn_matrix
    }

    pub fn category_map(&self) -> &CategoryMap {
        &self.cate_map
    }

    pub fn simple_oov_generator(&self) -> Option<&SimpleOovGenerator> {
        self.simple_oov.as_ref()
    }
}
