use std::collections::BTreeMap;

use anyhow::{anyhow, Result};

use super::{CategorySet, CharInfo, CharProperty};

struct CharRange {
    start: usize,
    end: usize,
    categories: CategorySet,
}

impl CharProperty {
    pub fn from_lines<I, L>(lines: I) -> Result<Self>
    where
        I: IntoIterator<Item = L>,
        L: AsRef<str>,
    {
        let mut cate2info = BTreeMap::new();
        let mut char_ranges = vec![];

        for line in lines {
            let line = line.as_ref().trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if !line.starts_with("0x") {
                let (category, invoke, group, length) = Self::parse_char_category(line)?;
                assert_eq!(category.len(), 1);
                let cate_id = category.first_id().unwrap();
                cate2info.insert(
                    cate_id,
                    CharInfo {
                        base_id: cate_id,
                        cate_ids: CategorySet::new(),
                        invoke,
                        group,
                        length: length as u16,
                    },
                );
            } else {
                char_ranges.push(Self::parse_char_range(line)?);
            }
        }

        let init_cinfo = Self::encode_cate_info("DEFAULT".parse().unwrap(), &cate2info);
        let mut chr2inf = vec![init_cinfo; 1 << 16];

        for r in &char_ranges {
            let inf = Self::encode_cate_info(r.categories, &cate2info);
            for c in r.start..r.end {
                chr2inf[c] = inf.clone();
            }
        }

        Ok(Self { chr2inf })
    }

    fn encode_cate_info(categories: CategorySet, cate2info: &BTreeMap<u32, CharInfo>) -> CharInfo {
        let mut base = cate2info
            .get(&categories.first_id().unwrap())
            .unwrap()
            .clone();
        for cate_id in categories.id_iter() {
            let info = cate2info.get(&cate_id).unwrap();
            base.cate_ids |= CategorySet::from_id(info.base_id);
        }
        base
    }

    fn parse_char_category(line: &str) -> Result<(CategorySet, bool, bool, usize)> {
        assert!(!line.is_empty());
        assert!(!line.starts_with("0x"));

        let cols: Vec<_> = line.split_whitespace().collect();
        if cols.len() < 4 {
            return Err(anyhow!("Invalid format: {}", line));
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
            return Err(anyhow!("InvalidFormat: {}", line));
        }

        let r: Vec<_> = cols[0].split("..").collect();
        let start = usize::from_str_radix(String::from(r[0]).trim_start_matches("0x"), 16)?;
        let end = if r.len() > 1 {
            usize::from_str_radix(String::from(r[1]).trim_start_matches("0x"), 16)? + 1
        } else {
            start + 1
        };
        if start >= end {
            return Err(anyhow!("InvalidFormat: {}", line));
        }
        // out of range
        if start > 0xFFFF || end >= 0xFFFF {
            return Err(anyhow!("InvalidFormat: {}", line));
        }

        let mut categories = CategorySet::new();
        for cate in cols[1..]
            .iter()
            .take_while(|c| c.chars().next().unwrap() != '#')
        {
            categories |= cate.parse()?;
        }

        Ok(CharRange {
            start,
            end,
            categories,
        })
    }
}
