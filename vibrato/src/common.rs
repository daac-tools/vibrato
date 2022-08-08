//! Common settings in Vibrato.
use bincode::config::{self, Fixint, LittleEndian};

/// Gets the common bincode configuration of serialization.
pub const fn bincode_config() -> config::Configuration<LittleEndian, Fixint> {
    config::standard()
        .with_little_endian()
        .with_fixed_int_encoding()
        .write_fixed_array_length()
}

/// The maximam length of an input sentence.
///
/// Note that the value must be represented with u16 so that
/// an (exclusive) end position can be represented in 16 bits.
pub const MAX_SENTENCE_LENGTH: u16 = 0xFFFF;

/// The fixed connection id of BOS/EOS.
pub const BOS_EOS_CONNECTION_ID: u16 = 0;
