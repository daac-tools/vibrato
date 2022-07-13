use anyhow::{anyhow, Result};

use super::{CategoryDef, UnkEntry, UnkHandler};

impl UnkHandler {
    pub fn from_lines<I, L>(char_def: I, unk_def: I) -> Result<Self>
    where
        I: IntoIterator<Item = L>,
        L: AsRef<str>,
    {
        let cate_defs = Self::parse_char_def(char_def)?;
        let unk_entries = Self::parse_unk_def(unk_def)?;
        Ok(Self::new(cate_defs, unk_entries))
    }

    fn parse_char_def<I, L>(lines: I) -> Result<Vec<CategoryDef>>
    where
        I: IntoIterator<Item = L>,
        L: AsRef<str>,
    {
        let mut cate_defs = vec![];
        for line in lines {
            let line = line.as_ref().trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with("0x") {
                continue;
            }

            let cols: Vec<_> = line.split_whitespace().collect();
            if cols.len() < 4 {
                return Err(anyhow!("Invalid format: {}", line));
            }

            // TODO: Handle errors
            let cate_type = cols[0].parse()?;
            let is_invoke = cols[1] == "1";
            let is_group = cols[2] == "1";
            let length = cols[3].parse()?;

            cate_defs.push(CategoryDef {
                cate_type,
                is_invoke,
                is_group,
                length,
            });
        }
        Ok(cate_defs)
    }

    fn parse_unk_def<I, L>(lines: I) -> Result<Vec<UnkEntry>>
    where
        I: IntoIterator<Item = L>,
        L: AsRef<str>,
    {
        let mut entries = vec![];
        for line in lines {
            let line = line.as_ref().trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let cols: Vec<_> = line.split(',').collect();
            if cols.len() < 4 {
                return Err(anyhow!("Invalid format: {}", line));
            }

            let cate_type = cols[0].parse()?;
            let left_id = cols[1].parse()?;
            let right_id = cols[2].parse()?;
            let word_cost = cols[3].parse()?;
            let feature = cols.get(4..).map_or("".to_string(), |x| x.join(","));

            entries.push(UnkEntry {
                cate_type,
                left_id,
                right_id,
                word_cost,
                feature,
            });
        }
        Ok(entries)
    }
}
