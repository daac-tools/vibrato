use std::io::{prelude::*, BufReader, Read};

use crate::dictionary::context_id::ContextIds;
use crate::errors::{Result, VibratoError};

impl ContextIds {
    /// Builds a new instance from `left-id.def` and `right-id.def`.
    ///
    /// Note that the reader is buffered automatically, so you should not
    /// wrap `rdr` in a buffered reader like `io::BufReader`.
    pub fn from_readers<L, R>(left_id_rdr: L, right_id_rdr: R) -> Result<Self>
    where
        L: Read,
        R: Read,
    {
        let eos_left_id = Self::parse_reader(left_id_rdr)?;
        let bos_right_id = Self::parse_reader(right_id_rdr)?;
        Ok(Self {
            bos_right_id,
            eos_left_id,
        })
    }

    /// Gets the id of BOS/EOS.
    fn parse_reader<R>(rdr: R) -> Result<u16>
    where
        R: Read,
    {
        let reader = BufReader::new(rdr);
        for line in reader.lines() {
            let line = line?;
            let i = line.bytes().position(|c| c == b' ').unwrap();
            let id_str = &line[..i];
            let feature = &line[i + 1..];
            if feature.starts_with("BOS/EOS") {
                return Ok(id_str.parse()?);
            }
        }
        Err(VibratoError::invalid_format(
            "left/right-id.def",
            "BOS/EOS is not defined.",
        ))
    }
}
