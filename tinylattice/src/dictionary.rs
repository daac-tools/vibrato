pub mod category;
pub mod lexicon;
pub mod matrix;
pub mod oov;

pub use category::CategoryTable;
pub use lexicon::Lexicon;
pub use matrix::CostMatrix;
pub use oov::SimpleOovGenerator;

pub use lexicon::WordParam;

pub struct Dictionary {
    pub lexicon: Lexicon,
    pub matrix: CostMatrix,
    pub cate_table: CategoryTable,
    pub oov_generator: SimpleOovGenerator,
}
