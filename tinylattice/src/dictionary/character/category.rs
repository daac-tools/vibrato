use bitflags::bitflags;
use std::str::FromStr;

use anyhow::{anyhow, Error, Result};

bitflags! {
    /// A set of categories for a character
    ///
    /// Implemented as a bitset with fixed size
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

impl Default for CategorySet {
    fn default() -> Self {
        Self { bits: 0 }
    }
}

impl FromStr for CategorySet {
    type Err = Error;

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
            _ => Err(anyhow!("Undefined category: {}", s)),
        }
    }
}

impl CategorySet {
    pub const NUM_CATEGORIES: usize = 11;

    pub fn new() -> Self {
        Self { bits: 0 }
    }

    pub fn from_id(id: u32) -> Self {
        debug_assert!((id as usize) < Self::NUM_CATEGORIES);
        Self { bits: 1 << id }
    }

    pub fn first_id(&self) -> Option<u32> {
        if self.bits == 0 {
            None
        } else {
            Some(self.bits.trailing_zeros())
        }
    }

    pub fn len(self) -> u32 {
        self.bits.count_ones()
    }

    pub fn id_iter(self) -> CategoryIdIter {
        CategoryIdIter { bits: self.bits }
    }
}

pub struct CategoryIdIter {
    bits: u32,
}

/// Iterating over individual bitfields (somehow is not automatically implemented)
/// by bitfields crate
impl Iterator for CategoryIdIter {
    type Item = u32;

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
