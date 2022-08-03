use bincode::config::{self, Fixint, LittleEndian};

pub fn bincode_config() -> config::Configuration<LittleEndian, Fixint> {
    config::standard()
        .with_little_endian()
        .with_fixed_int_encoding()
        .write_fixed_array_length()
}
