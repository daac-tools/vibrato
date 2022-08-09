use bitflags::bitflags;
use std::str::FromStr;

use crate::errors::{Result, VibratoError};

bitflags! {
    /// A set of categories for a character, inspired by sudachi.rs.
    #[repr(transparent)]
    pub struct CategorySet: u32 {
        const DEFAULT = (1 << 0);
        const SPACE = (1 << 1);
        const KANJI = (1 << 2);
        const SYMBOL = (1 << 3);
        const NUMERIC = (1 << 4);
        const ALPHA = (1 << 5);
        const HIRAGANA = (1 << 6);
        const KATAKANA = (1 << 7);
        const KANJINUMERIC = (1 << 8);
        const GREEK = (1 << 9);
        const CYRILLIC = (1 << 10);
    }
}

impl FromStr for CategorySet {
    type Err = VibratoError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "DEFAULT" => Ok(Self::DEFAULT),
            "SPACE" => Ok(Self::SPACE),
            "KANJI" => Ok(Self::KANJI),
            "SYMBOL" => Ok(Self::SYMBOL),
            "NUMERIC" => Ok(Self::NUMERIC),
            "ALPHA" => Ok(Self::ALPHA),
            "HIRAGANA" => Ok(Self::HIRAGANA),
            "KATAKANA" => Ok(Self::KATAKANA),
            "KANJINUMERIC" => Ok(Self::KANJINUMERIC),
            "GREEK" => Ok(Self::GREEK),
            "CYRILLIC" => Ok(Self::CYRILLIC),
            _ => Err(VibratoError::invalid_argument(
                "s",
                format!("Undefined category name, {}", s),
            )),
        }
    }
}

impl CategorySet {
    pub const NUM_CATEGORIES: usize = 11;

    #[inline(always)]
    pub const fn new() -> Self {
        Self { bits: 0 }
    }

    /// # Safety
    ///
    /// `bits >> Self::NUM_CATEGORIES == 0`
    #[inline(always)]
    pub unsafe fn from_raw_unchecked(bits: u32) -> Self {
        debug_assert_eq!(bits >> Self::NUM_CATEGORIES, 0);
        Self { bits }
    }

    #[inline(always)]
    pub fn from_id(id: u32) -> Self {
        debug_assert!((id as usize) < Self::NUM_CATEGORIES);
        Self { bits: 1 << id }
    }

    #[inline(always)]
    pub const fn first_id(&self) -> Option<u32> {
        if self.bits == 0 {
            None
        } else {
            Some(self.bits.trailing_zeros())
        }
    }

    #[inline(always)]
    pub const fn raw(&self) -> u32 {
        self.bits
    }

    #[inline(always)]
    pub const fn len(&self) -> u32 {
        self.bits.count_ones()
    }

    #[inline(always)]
    pub const fn id_iter(&self) -> CategoryIdIter {
        CategoryIdIter { bits: self.bits }
    }

    pub fn cate_str(cate_id: u32) -> Option<String> {
        let cate = match cate_id {
            0 => "DEFAULT",
            1 => "SPACE",
            2 => "KANJI",
            3 => "SYMBOL",
            4 => "NUMERIC",
            5 => "ALPHA",
            6 => "HIRAGANA",
            7 => "KATAKANA",
            8 => "KANJINUMERIC",
            9 => "GREEK",
            10 => "CYRILLIC",
            _ => return None,
        };
        Some(cate.to_string())
    }

    pub fn cate_strs(&self) -> Vec<String> {
        let mut cates = vec![];
        for id in self.id_iter() {
            cates.push(Self::cate_str(id).unwrap());
        }
        cates
    }
}

pub struct CategoryIdIter {
    bits: u32,
}

/// Iterating over individual bitfields (somehow is not automatically implemented)
/// by bitfields crate
impl Iterator for CategoryIdIter {
    type Item = u32;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.bits == 0 {
            return None;
        }
        let numtz = self.bits.trailing_zeros();
        let mask = 1u32 << numtz;
        self.bits ^= mask;
        Some(numtz)
    }
}
