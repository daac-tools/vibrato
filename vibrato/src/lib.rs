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
//! use std::ops::Deref;
//!
//! use vibrato::{Dictionary, Tokenizer};
//!
//! let file = File::open("src/tests/resources/system.dic").unwrap();
//! let dict = Dictionary::read(BufReader::new(file)).unwrap();
//!
//! let mut tokenizer = vibrato::Tokenizer::new(&dict);
//! let tokens = tokenizer.tokenize("京都東京都").unwrap();
//!
//! assert_eq!(tokens.len(), 2);
//!
//! assert_eq!(tokens.get(0).surface().deref(), "京都");
//! assert_eq!(tokens.get(0).range_char(), 0..2);
//! assert_eq!(tokens.get(0).range_byte(), 0..6);
//!
//! assert_eq!(tokens.get(1).surface().deref(), "東京都");
//! assert_eq!(tokens.get(1).range_char(), 2..5);
//! assert_eq!(tokens.get(1).range_byte(), 6..15);
//! ```
#![deny(missing_docs)]

#[cfg(target_pointer_width = "16")]
compile_error!("`target_pointer_width` must be larger than or equal to 32");

mod common;
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
