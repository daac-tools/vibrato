pub mod accessor;
pub mod builder;

#[derive(Default, Clone)]
pub struct Sentence {
    /// Original input data, output is done on this
    original: String,
    /// Normalized input data, analysis is done on this. Byte-based indexing.
    modified: String,
    /// Buffer for normalization, reusing allocations
    modified_2: String,
    /// Byte mapping from normalized data to originals.
    /// Only values lying on codepoint boundaries are correct. Byte-based indexing.
    m2o: Vec<usize>,
    /// Buffer for normalization.
    /// After building it is used as byte-to-char mapping for original data.
    m2o_2: Vec<usize>,
    /// Characters of the modified string. Char-based indexing.
    mod_chars: Vec<char>,
    /// Char-to-byte mapping for the modified string. Char-based indexing.
    mod_c2b: Vec<usize>,
    /// Byte-to-char mapping for the modified string. Byte-based indexing.
    mod_b2c: Vec<usize>,
    /// Markers whether the byte can start new word or not
    mod_bow: Vec<bool>,
}
