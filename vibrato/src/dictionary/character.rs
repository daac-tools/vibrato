use std::collections::BTreeMap;
use std::fmt;
use std::io::{prelude::*, BufReader, Read};

use bincode::{Decode, Encode};

use crate::errors::{Result, VibratoError};
use crate::utils::FromU32;

const CATE_IDSET_BITS: usize = 18;
const CATE_IDSET_MASK: u32 = (1 << CATE_IDSET_BITS) - 1;
const BASE_ID_BITS: usize = 8;
const BASE_ID_MASK: u32 = (1 << BASE_ID_BITS) - 1;
const LENGTH_BITS: usize = 4;

/// Information of a character defined in `char.def`.
///
/// The memory layout is
///   cate_idset = 18 bits
///      base_id =  8 bits
///       invoke =  1 bit
///        group =  1 bit
///       length =  4 bits
#[derive(Default, Clone, Copy, Decode, Encode)]
pub struct CharInfo(u32);

impl fmt::Debug for CharInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CharInfo")
            .field("cate_idset", &self.cate_idset())
            .field("base_id", &self.base_id())
            .field("invoke", &self.invoke())
            .field("group", &self.group())
            .field("length", &self.length())
            .finish()
    }
}

impl CharInfo {
    pub fn new(
        cate_idset: u32,
        base_id: u32,
        invoke: bool,
        group: bool,
        length: u16,
    ) -> Option<Self> {
        if cate_idset >> CATE_IDSET_BITS != 0 {
            return None;
        }
        if base_id >> BASE_ID_BITS != 0 {
            return None;
        }
        if length >> LENGTH_BITS != 0 {
            return None;
        }
        Some(Self(
            cate_idset
                | (base_id << CATE_IDSET_BITS)
                | (u32::from(invoke) << (CATE_IDSET_BITS + BASE_ID_BITS))
                | (u32::from(group) << (CATE_IDSET_BITS + BASE_ID_BITS + 1))
                | ((u32::from(length)) << (CATE_IDSET_BITS + BASE_ID_BITS + 2)),
        ))
    }

    #[inline(always)]
    pub fn reset_cate_idset(&mut self, cate_idset: u32) {
        self.0 &= !CATE_IDSET_MASK;
        self.0 |= cate_idset;
    }

    #[inline(always)]
    pub const fn cate_idset(&self) -> u32 {
        self.0 & CATE_IDSET_MASK
    }

    #[inline(always)]
    pub const fn base_id(&self) -> u32 {
        (self.0 >> CATE_IDSET_BITS) & BASE_ID_MASK
    }

    #[inline(always)]
    pub const fn invoke(&self) -> bool {
        (self.0 >> (CATE_IDSET_BITS + BASE_ID_BITS)) & 1 != 0
    }

    #[inline(always)]
    pub const fn group(&self) -> bool {
        (self.0 >> (CATE_IDSET_BITS + BASE_ID_BITS + 1)) & 1 != 0
    }

    #[inline(always)]
    pub const fn length(&self) -> u16 {
        (self.0 >> (CATE_IDSET_BITS + BASE_ID_BITS + 2)) as u16
    }
}

struct CharRange {
    start: usize,
    end: usize,
    categories: Vec<String>,
}

/// Mapping from characters to their information.
#[derive(Decode, Encode)]
pub struct CharProperty {
    chr2inf: Vec<CharInfo>,
    categories: Vec<String>, // indexed by category id
}

impl CharProperty {
    #[inline(always)]
    pub fn char_info(&self, c: char) -> CharInfo {
        self.chr2inf
            .get(usize::from_u32(u32::from(c)))
            .map_or_else(|| self.chr2inf[0], |cinfo| *cinfo)
    }

    #[inline(always)]
    pub fn cate_id(&self, category: &str) -> Option<u32> {
        self.categories
            .iter()
            .position(|cate| cate == category)
            .map(|id| u32::try_from(id).unwrap())
    }

    #[inline(always)]
    pub fn cate_str(&self, cate_id: u32) -> Option<&str> {
        self.categories
            .get(usize::from_u32(cate_id))
            .map(|c| c.as_str())
    }

    #[inline(always)]
    pub fn num_categories(&self) -> usize {
        self.categories.len()
    }

    /// Creates a new instance from `char.def`.
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

        let init_cinfo = Self::encode_cate_info(&["DEFAULT"], &cate2info, &cate_map)?;
        let mut chr2inf = vec![init_cinfo; 1 << 16];

        for r in &char_ranges {
            let cinfo = Self::encode_cate_info(&r.categories, &cate2info, &cate_map)?;
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
    ) -> Result<CharInfo>
    where
        S: AsRef<str>,
    {
        let mut base_cinfo = *cate_map
            .get(targets[0].as_ref())
            .and_then(|base_target_id| cate2info.get(base_target_id))
            .ok_or_else(|| {
                let msg = format!("Undefined category: {}", targets[0].as_ref());
                VibratoError::invalid_format("char.def", msg)
            })?;
        let mut cate_idset = base_cinfo.cate_idset();
        for target in targets {
            let target_id = cate_map.get(target.as_ref()).unwrap();
            let cinfo = cate2info.get(target_id).unwrap();
            cate_idset |= 1 << cinfo.base_id();
        }
        base_cinfo.reset_cate_idset(cate_idset);
        Ok(base_cinfo)
    }

    fn parse_char_category(line: &str) -> Result<(String, bool, bool, u16)> {
        assert!(!line.is_empty());
        assert!(!line.starts_with("0x"));

        let cols: Vec<_> = line.split_whitespace().collect();
        if cols.len() < 4 {
            let msg = format!(
                "A character category must consists of four items separated by spaces, {line}",
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
            let msg = format!("A character range must have two items at least, {line}");
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
            let msg =
                format!("The start of a character range must be no more than the end, {line}");
            return Err(VibratoError::invalid_format("char.def", msg));
        }
        if start > 0xFFFF || end > 0x10000 {
            let msg = format!("A character range must be no more 0xFFFF, {line}");
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
    fn test_invalid_cate() {
        let data = "DEFAULT 0 1 0\n0x0..0xFFFF INVALID";
        let result = CharProperty::from_reader(data.as_bytes());
        assert!(result.is_err());
    }

    #[test]
    fn test_no_default_cate() {
        let data = "USER_DEFINED 0 1 0";
        let result = CharProperty::from_reader(data.as_bytes());
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_invoke() {
        let data = "DEFAULT 2 1 0";
        let result = CharProperty::from_reader(data.as_bytes());
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_group() {
        let data = "DEFAULT 0 2 0";
        let result = CharProperty::from_reader(data.as_bytes());
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_length() {
        let data = "DEFAULT 0 2 -1";
        let result = CharProperty::from_reader(data.as_bytes());
        assert!(result.is_err());
    }

    #[test]
    fn test_few_cols() {
        let data = "DEFAULT 0 2";
        let result = CharProperty::from_reader(data.as_bytes());
        assert!(result.is_err());
    }

    #[test]
    fn test_char_range_1() {
        let data = "DEFAULT 0 1 0\n0x10000 DEFAULT";
        let result = CharProperty::from_reader(data.as_bytes());
        assert!(result.is_err());
    }

    #[test]
    fn test_char_range_2() {
        let data = "DEFAULT 0 1 0\n0x0..0xFFFF DEFAULT";
        CharProperty::from_reader(data.as_bytes()).unwrap();
    }

    #[test]
    fn test_char_range_3() {
        let data = "DEFAULT 0 1 0\n0x0..0x10000 DEFAULT";
        let result = CharProperty::from_reader(data.as_bytes());
        assert!(result.is_err());
    }

    #[test]
    fn test_char_range_4() {
        let data = "DEFAULT 0 1 0\n0x0020..0x0019 DEFAULT";
        let result = CharProperty::from_reader(data.as_bytes());
        assert!(result.is_err());
    }
}
