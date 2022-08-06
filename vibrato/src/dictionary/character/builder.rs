use std::collections::BTreeMap;
use std::io::{prelude::*, BufReader, Read};

use crate::errors::{Result, VibratoError};

use super::{CategorySet, CharInfo, CharProperty};

struct CharRange {
    start: usize,
    end: usize,
    // Need to use Vec, not CategorySet, to preserve the id order defined
    // in char.def and follow the original MeCab implementation.
    cate_ids: Vec<u32>,
}

impl CharProperty {
    /// Creates a new instance from `char.def`.
    ///
    /// # Arguments
    ///
    ///  - `rdr`: A reader of the file.
    pub fn from_reader<R>(rdr: R) -> Result<Self>
    where
        R: Read,
    {
        let mut cate2info = BTreeMap::new();
        let mut char_ranges = vec![];

        let reader = BufReader::new(rdr);
        for line in reader.lines() {
            let line = line?;
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if !line.starts_with("0x") {
                let (category, invoke, group, length) = Self::parse_char_category(line)?;
                assert_eq!(category.len(), 1);
                let cate_id = category.first_id().unwrap();
                cate2info.insert(
                    cate_id,
                    CharInfo::new(CategorySet::new(), cate_id, invoke, group, length).unwrap(),
                );
            } else {
                char_ranges.push(Self::parse_char_range(line)?);
            }
        }

        let init_cinfo =
            Self::encode_cate_info(&[CategorySet::DEFAULT.first_id().unwrap()], &cate2info);
        let mut chr2inf = vec![init_cinfo; 1 << 16];

        for r in &char_ranges {
            let cinfo = Self::encode_cate_info(&r.cate_ids, &cate2info);
            for e in chr2inf.iter_mut().take(r.end).skip(r.start) {
                *e = cinfo;
            }
        }

        Ok(Self { chr2inf })
    }

    fn encode_cate_info(target_ids: &[u32], cate2info: &BTreeMap<u32, CharInfo>) -> CharInfo {
        let mut base = *cate2info.get(target_ids.first().unwrap()).unwrap();
        let mut cate_ids = base.cate_ids();
        for target_id in target_ids {
            let cinfo = cate2info.get(target_id).unwrap();
            cate_ids |= CategorySet::from_id(cinfo.base_id());
        }
        base.set_cate_ids(cate_ids);
        base
    }

    fn parse_char_category(line: &str) -> Result<(CategorySet, bool, bool, u16)> {
        assert!(!line.is_empty());
        assert!(!line.starts_with("0x"));

        let cols: Vec<_> = line.split_whitespace().collect();
        if cols.len() < 4 {
            let msg = format!(
                "A character category must consists of four items separated by spaces, {}",
                line
            );
            return Err(VibratoError::invalid_argument("line", msg));
        }

        // TODO: Handle errors
        let category = cols[0].parse()?;
        let invoke = cols[1] == "1";
        let group = cols[2] == "1";
        let length = cols[3].parse()?;

        Ok((category, invoke, group, length))
    }

    fn parse_char_range(line: &str) -> Result<CharRange> {
        assert!(!line.is_empty());
        assert!(line.starts_with("0x"));

        let cols: Vec<_> = line.split_whitespace().collect();
        if cols.len() < 2 {
            let msg = format!("A character range must have two items at least, {}", line);
            return Err(VibratoError::invalid_argument("line", msg));
        }

        let r: Vec<_> = cols[0].split("..").collect();
        let start = usize::from_str_radix(String::from(r[0]).trim_start_matches("0x"), 16)?;
        let end = if r.len() > 1 {
            usize::from_str_radix(String::from(r[1]).trim_start_matches("0x"), 16)? + 1
        } else {
            start + 1
        };
        if start >= end {
            let msg = format!(
                "The start of a character range must be no more than the end, {}",
                line
            );
            return Err(VibratoError::invalid_argument("line", msg));
        }
        if start > 0xFFFF || end >= 0xFFFF {
            let msg = format!("A character range must be no more 0xFFFF, {}", line);
            return Err(VibratoError::invalid_argument("line", msg));
        }

        let mut cate_ids = vec![];
        for &cate in cols[1..].iter().take_while(|&&col| !col.starts_with('#')) {
            cate_ids.push(cate.parse::<CategorySet>()?.first_id().unwrap());
        }

        Ok(CharRange {
            start,
            end,
            cate_ids,
        })
    }
}
