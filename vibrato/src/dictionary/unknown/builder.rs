use std::io::{prelude::*, BufReader, Read};

use anyhow::{anyhow, Result};

use crate::dictionary::character::CategorySet;

use super::{UnkEntry, UnkHandler};

impl UnkHandler {
    /// Creates a new instance from `unk.def`.
    ///
    /// # Arguments
    ///
    ///  - `rdr`: A reader of the file.
    pub fn from_reader<R>(rdr: R) -> Result<Self>
    where
        R: Read,
    {
        let mut map = vec![vec![]; CategorySet::NUM_CATEGORIES];

        let reader = BufReader::new(rdr);
        for line in reader.lines() {
            let line = line?;
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let e = Self::parse_unk_entry(line)?;
            map[e.cate_id as usize].push(e);
        }

        let mut offsets = vec![];
        let mut entries = vec![];
        for mut v in map {
            offsets.push(entries.len());
            entries.append(&mut v);
        }
        offsets.push(entries.len());
        Ok(Self { offsets, entries })
    }

    fn parse_unk_entry(line: &str) -> Result<UnkEntry> {
        let cols: Vec<_> = line.split(',').collect();
        if cols.len() < 4 {
            return Err(anyhow!("Invalid format: {}", line));
        }

        let category: CategorySet = cols[0].parse()?;
        let left_id = cols[1].parse()?;
        let right_id = cols[2].parse()?;
        let word_cost = cols[3].parse()?;
        let feature = cols.get(4..).map_or("".to_string(), |x| x.join(","));

        let cate_id = category.first_id().unwrap() as u16;

        Ok(UnkEntry {
            cate_id,
            left_id,
            right_id,
            word_cost,
            feature,
        })
    }
}
