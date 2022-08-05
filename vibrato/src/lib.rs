//!
//!
#![deny(missing_docs)]

#[cfg(target_pointer_width = "16")]
compile_error!("`target_pointer_width` must be larger than or equal to 32");

pub mod common;
pub mod dictionary;
mod sentence;
pub mod token;
pub mod tokenizer;
mod utils;

#[cfg(test)]
mod tests;

pub use dictionary::Dictionary;
pub use tokenizer::Tokenizer;
