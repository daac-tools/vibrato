//! # Vibrato
//!
//! Vibrato is a fast implementation of tokenization (or morphological analysis)
//! based on the viterbi algorithm.
//!
//! ## Examples
//!
//! ```
//! use std::fs::File;
//! use std::io::{BufRead, BufReader};
//!
//! let mut reader = BufReader::new(File::open("src/tests/resources/system.dic").unwrap());
//! let dict = bincode::decode_from_std_read(&mut reader, vibrato::common::bincode_config()).unwrap();
//!
//! let mut tokenizer = vibrato::Tokenizer::new(&dict);
//! let tokens = tokenizer.tokenize("京都東京都").unwrap();
//!
//! assert_eq!(tokens.len(), 2);
//! ```
#![deny(missing_docs)]

#[cfg(target_pointer_width = "16")]
compile_error!("`target_pointer_width` must be larger than or equal to 32");

pub mod common;
pub mod dictionary;
pub mod errors;
mod sentence;
pub mod token;
pub mod tokenizer;
mod utils;

#[cfg(test)]
mod tests;

pub use dictionary::Dictionary;
pub use tokenizer::Tokenizer;
