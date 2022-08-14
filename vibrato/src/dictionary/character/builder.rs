use std::collections::BTreeMap;
use std::io::{prelude::*, BufReader, Read};

use crate::dictionary::character::{CharInfo, CharProperty};
use crate::errors::{Result, VibratoError};
use crate::utils::FromU32;

struct CharRange {
    start: usize,
    end: usize,
    categories: Vec<String>,
}

impl CharProperty {
    /// Creates a new instance from `char.def`.
    ///
    /// Note that the reader is buffered automatically, so you should not
    /// wrap `rdr` in a buffered reader like `io::BufReader`.
    pub fn from_reader<R>(rdr: R) -> Result<Self>
    where
        R: Read,
    {
        let mut cate2info = BTreeMap::new();
        let mut cate_map = BTreeMap::new(); // Name -> Id
        let mut char_ranges = vec![];

        cate_map.insert("DEFAULT".to_string(), 0);

        let reader = BufReader::new(rdr);
        for line in reader.lines() {
            let line = line?;
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if !line.starts_with("0x") {
                let (category, invoke, group, length) = Self::parse_char_category(line)?;
                let new_cate_id = u32::try_from(cate_map.len()).unwrap();
                let cate_id = *cate_map.entry(category).or_insert(new_cate_id);
                cate2info.insert(
                    cate_id,
                    CharInfo::new(0, cate_id, invoke, group, length).unwrap(),
                );
            } else {
                char_ranges.push(Self::parse_char_range(line)?);
            }
        }

        let init_cinfo = Self::encode_cate_info(&["DEFAULT"], &cate2info, &cate_map);
        let mut chr2inf = vec![init_cinfo; 1 << 16];

        for r in &char_ranges {
            let cinfo = Self::encode_cate_info(&r.categories, &cate2info, &cate_map);
            for e in chr2inf.iter_mut().take(r.end).skip(r.start) {
                *e = cinfo;
            }
        }

        let mut categories = vec![String::new(); cate_map.len()];
        for (k, &v) in cate_map.iter() {
            categories[usize::from_u32(v)] = k.clone();
        }

        Ok(Self {
            chr2inf,
            categories,
        })
    }

    fn encode_cate_info<S>(
        targets: &[S],
        cate2info: &BTreeMap<u32, CharInfo>,
        cate_map: &BTreeMap<String, u32>,
    ) -> CharInfo
    where
        S: AsRef<str>,
    {
        let base_target_id = cate_map.get(targets[0].as_ref()).unwrap();
        let mut base_cinfo = *cate2info.get(base_target_id).unwrap();
        let mut cate_idset = base_cinfo.cate_idset();
        for target in targets {
            let target_id = cate_map.get(target.as_ref()).unwrap();
            let cinfo = cate2info.get(target_id).unwrap();
            cate_idset |= 1 << cinfo.base_id();
        }
        base_cinfo.reset_cate_idset(cate_idset);
        base_cinfo
    }

    fn parse_char_category(line: &str) -> Result<(String, bool, bool, u16)> {
        assert!(!line.is_empty());
        assert!(!line.starts_with("0x"));

        let cols: Vec<_> = line.split_whitespace().collect();
        if cols.len() < 4 {
            let msg = format!(
                "A character category must consists of four items separated by spaces, {}",
                line
            );
            return Err(VibratoError::invalid_format("char.def", msg));
        }

        let category = cols[0].to_string();
        let invoke = ["1", "0"]
            .contains(&cols[1])
            .then(|| cols[1] == "1")
            .ok_or_else(|| VibratoError::invalid_format("char.def", "INVOKE must be 1 or 0."))?;
        let group = ["1", "0"]
            .contains(&cols[2])
            .then(|| cols[2] == "1")
            .ok_or_else(|| VibratoError::invalid_format("char.def", "GROUP must be 1 or 0."))?;
        let length = cols[3].parse()?;

        Ok((category, invoke, group, length))
    }

    fn parse_char_range(line: &str) -> Result<CharRange> {
        assert!(!line.is_empty());
        assert!(line.starts_with("0x"));

        let cols: Vec<_> = line.split_whitespace().collect();
        if cols.len() < 2 {
            let msg = format!("A character range must have two items at least, {}", line);
            return Err(VibratoError::invalid_format("char.def", msg));
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
            return Err(VibratoError::invalid_format("char.def", msg));
        }
        if start > 0xFFFF || end > 0x10000 {
            let msg = format!("A character range must be no more 0xFFFF, {}", line);
            return Err(VibratoError::invalid_format("char.def", msg));
        }

        let mut categories = vec![];
        for &cate in cols[1..].iter().take_while(|&&col| !col.starts_with('#')) {
            categories.push(cate.to_string());
        }

        Ok(CharRange {
            start,
            end,
            categories,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let data = "DEFAULT 0 1 0\nSPACE 0 1 0\n0x0020 SPACE";
        let prop = CharProperty::from_reader(data.as_bytes()).unwrap();
        assert_eq!(prop.chr2inf[0x0020].cate_idset(), 0b10);
        assert_eq!(prop.chr2inf[0x0020].base_id(), 1);
        assert_eq!(prop.chr2inf[0x0020].invoke(), false);
        assert_eq!(prop.chr2inf[0x0020].group(), true);
        assert_eq!(prop.chr2inf[0x0020].length(), 0);
    }

    #[test]
    #[should_panic]
    fn test_invalid_cate() {
        let data = "INVALID 0 1 0";
        CharProperty::from_reader(data.as_bytes()).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_invalid_invoke() {
        let data = "DEFAULT 2 1 0";
        CharProperty::from_reader(data.as_bytes()).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_invalid_group() {
        let data = "DEFAULT 0 2 0";
        CharProperty::from_reader(data.as_bytes()).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_invalid_length() {
        let data = "DEFAULT 0 2 -1";
        CharProperty::from_reader(data.as_bytes()).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_few_cols() {
        let data = "DEFAULT 0 2";
        CharProperty::from_reader(data.as_bytes()).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_char_range_1() {
        let data = "DEFAULT 0 1 0\n0x10000 DEFAULT";
        CharProperty::from_reader(data.as_bytes()).unwrap();
    }

    #[test]
    fn test_char_range_2() {
        let data = "DEFAULT 0 1 0\n0x0..0xFFFF DEFAULT";
        CharProperty::from_reader(data.as_bytes()).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_char_range_3() {
        let data = "DEFAULT 0 1 0\n0x0..0x10000 DEFAULT";
        CharProperty::from_reader(data.as_bytes()).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_char_range_4() {
        let data = "DEFAULT 0 1 0\n0x0020..0x0019 DEFAULT";
        CharProperty::from_reader(data.as_bytes()).unwrap();
    }
}
