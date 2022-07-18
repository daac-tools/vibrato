use anyhow::{anyhow, Result};

use crate::dictionary::character::CategorySet;

use super::{UnkEntry, UnkHandler};

impl UnkHandler {
    pub fn from_lines<I, L>(lines: I) -> Result<Self>
    where
        I: IntoIterator<Item = L>,
        L: AsRef<str>,
    {
        let mut entries = vec![vec![]; CategorySet::NUM_CATEGORIES];

        for line in lines {
            let line = line.as_ref().trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let e = Self::parse_unk_entry(line)?;
            entries[e.cate_id as usize].push(e);
        }

        Ok(Self { entries })
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
