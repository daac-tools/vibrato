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
//! let t0 = tokens.get(0);
//! assert_eq!(t0.surface().deref(), "京都");
//! assert_eq!(t0.range_char(), 0..2);
//! assert_eq!(t0.range_byte(), 0..6);
//! assert_eq!(t0.feature(), "京都,名詞,固有名詞,地名,一般,*,*,キョウト,京都,*,A,*,*,*,1/5");
//!
//! let t1 = tokens.get(1);
//! assert_eq!(t1.surface().deref(), "東京都");
//! assert_eq!(t1.range_char(), 2..5);
//! assert_eq!(t1.range_byte(), 6..15);
//! assert_eq!(t1.feature(), "東京都,名詞,固有名詞,地名,一般,*,*,トウキョウト,東京都,*,B,5/9,*,5/9,*");
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
