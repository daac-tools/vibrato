use std::io::{prelude::*, BufReader, Read};

use super::ConnIdMapper;
use crate::errors::{Result, VibratoError};

use crate::common::BOS_EOS_CONNECTION_ID;

impl ConnIdMapper {
    /// Creates a new instance from mappings.
    ///
    /// # Arguments
    ///
    ///  - `l_ranks`: A list of connection left-ids sorted by rank.
    ///  - `r_ranks`: A list of connection right-ids sorted by rank.
    pub fn from_ranks<L, R>(l_ranks: L, r_ranks: R) -> Result<Self>
    where
        L: IntoIterator<Item = u16>,
        R: IntoIterator<Item = u16>,
    {
        let left = Self::compile(l_ranks)?;
        let right = Self::compile(r_ranks)?;
        Ok(Self { left, right })
    }

    fn compile<I>(ranks: I) -> Result<Vec<u16>>
    where
        I: IntoIterator<Item = u16>,
    {
        let mut old_ids = vec![BOS_EOS_CONNECTION_ID];
        for old_id in ranks {
            if old_id == BOS_EOS_CONNECTION_ID {
                return Err(VibratoError::invalid_argument(
                    "ranks",
                    format!("Id {} is reserved", BOS_EOS_CONNECTION_ID),
                ));
            }
            old_ids.push(old_id);
        }
        let mut new_ids = vec![0; old_ids.len()];
        for (new_id, &old_id) in old_ids.iter().enumerate().skip(1) {
            debug_assert_ne!(old_id, BOS_EOS_CONNECTION_ID);
            new_ids[usize::from(old_id)] = u16::try_from(new_id)?;
        }
        Ok(new_ids)
    }

    /// Creates a new instance from tsv files in which the first column indicates
    /// connection ids sorted by rank.
    ///
    /// # Arguments
    ///
    ///  - `l_rdr`: A reader of the file for left-ids.
    ///  - `r_rdr`: A reader of the file for right-ids.
    pub fn from_reader<L, R>(l_rdr: L, r_rdr: R) -> Result<Self>
    where
        L: Read,
        R: Read,
    {
        let left = Self::read(l_rdr)?;
        let right = Self::read(r_rdr)?;
        Ok(Self { left, right })
    }

    fn read<R>(rdr: R) -> Result<Vec<u16>>
    where
        R: Read,
    {
        let reader = BufReader::new(rdr);
        let lines = reader.lines();

        let mut old_ids = vec![BOS_EOS_CONNECTION_ID];
        for line in lines {
            let line = line?;
            let cols: Vec<_> = line.split('\t').collect();
            if cols.is_empty() {
                return Err(VibratoError::invalid_argument(
                    "rdr",
                    "A line must not be empty.",
                ));
            }
            let old_id = cols[0].parse()?;
            if old_id == BOS_EOS_CONNECTION_ID {
                let msg = format!("Id {} is reserved, {}", BOS_EOS_CONNECTION_ID, line);
                return Err(VibratoError::invalid_argument("rdr", msg));
            }
            old_ids.push(old_id);
        }
        let mut new_ids = vec![0; old_ids.len()];
        for (new_id, &old_id) in old_ids.iter().enumerate().skip(1) {
            debug_assert_ne!(old_id, BOS_EOS_CONNECTION_ID);
            new_ids[usize::from(old_id)] = u16::try_from(new_id)?;
        }
        Ok(new_ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read() {
        let data = "2\n3\n4\n1\n";
        let mapping = ConnIdMapper::read(data.as_bytes()).unwrap();
        assert_eq!(mapping, vec![0, 4, 1, 2, 3]);
    }
}
