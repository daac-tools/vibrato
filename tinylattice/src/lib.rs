#[cfg(target_pointer_width = "16")]
compile_error!("`target_pointer_width` must be larger than or equal to 32");

pub mod dictionary;
pub mod morpheme;
pub mod tokenizer;

mod sentence;

#[cfg(test)]
pub mod tests;

pub use dictionary::Dictionary;
pub use tokenizer::Tokenizer;
