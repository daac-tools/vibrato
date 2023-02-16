//! # Vibrato
//!
//! Vibrato is a fast implementation of tokenization (or morphological analysis)
//! based on the viterbi algorithm.
//!
//! ## Examples
//!
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use std::fs::File;
//! use std::io::{BufRead, BufReader};
//!
//! use vibrato::{SystemDictionaryBuilder, Tokenizer};
//!
//! // Loads a set of raw dictionary files
//! let dict = SystemDictionaryBuilder::from_readers(
//!     File::open("src/tests/resources/lex.csv")?,
//!     File::open("src/tests/resources/matrix.def")?,
//!     File::open("src/tests/resources/char.def")?,
//!     File::open("src/tests/resources/unk.def")?,
//! )?;
//! // or loads a compiled dictionary
//! // let reader = File::open("path/to/system.dic")?;
//! // let dict = Dictionary::read(reader)?;
//!
//! let tokenizer = vibrato::Tokenizer::new(dict);
//! let mut worker = tokenizer.new_worker();
//!
//! worker.reset_sentence("京都東京都");
//! worker.tokenize();
//! assert_eq!(worker.num_tokens(), 2);
//!
//! let t0 = worker.token(0);
//! assert_eq!(t0.surface(), "京都");
//! assert_eq!(t0.range_char(), 0..2);
//! assert_eq!(t0.range_byte(), 0..6);
//! assert_eq!(t0.feature(), "京都,名詞,固有名詞,地名,一般,*,*,キョウト,京都,*,A,*,*,*,1/5");
//!
//! let t1 = worker.token(1);
//! assert_eq!(t1.surface(), "東京都");
//! assert_eq!(t1.range_char(), 2..5);
//! assert_eq!(t1.range_byte(), 6..15);
//! assert_eq!(t1.feature(), "東京都,名詞,固有名詞,地名,一般,*,*,トウキョウト,東京都,*,B,5/9,*,5/9,*");
//! # Ok(())
//! # }
//! ```
#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(not(any(target_pointer_width = "32", target_pointer_width = "64")))]
compile_error!("`target_pointer_width` must be 32 or 64");

pub mod common;
pub mod dictionary;
pub mod errors;
mod num;
mod sentence;
pub mod token;
pub mod tokenizer;
mod utils;

#[cfg(feature = "train")]
#[cfg_attr(docsrs, doc(cfg(feature = "train")))]
pub mod mecab;

#[cfg(feature = "train")]
#[cfg_attr(docsrs, doc(cfg(feature = "train")))]
pub mod trainer;

#[cfg(all(test, feature = "train"))]
mod test_utils;
#[cfg(test)]
mod tests;

pub use dictionary::{Dictionary, SystemDictionaryBuilder};
pub use tokenizer::Tokenizer;

/// Version number of this library.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
