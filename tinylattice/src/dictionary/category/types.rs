use bitflags::bitflags;
use std::str::FromStr;

use anyhow::{anyhow, Error, Result};

bitflags! {
    /// A set of categories for a character
    ///
    /// Implemented as a bitset with fixed size
    #[repr(transparent)]
    pub struct CategoryTypes: u32 {
        /** The fall back category. */
        const DEFAULT = (1 << 0);
        /** White spaces. */
        const SPACE = (1 << 1);
        /** CJKV ideographic characters. */
        const KANJI = (1 << 2);
        /** Symbols. */
        const SYMBOL = (1 << 3);
        /** Numerical characters. */
        const NUMERIC = (1 << 4);
        /** Latin alphabets. */
        const ALPHA = (1 << 5);
        /** Hiragana characters. */
        const HIRAGANA = (1 << 6);
        /** Katakana characters. */
        const KATAKANA = (1 << 7);
        /** Kanji numeric characters. */
        const KANJINUMERIC = (1 << 8);
        /** Greek alphabets. */
        const GREEK = (1 << 9);
        /** Cyrillic alphabets. */
        const CYRILLIC = (1 << 10);
        /** User defined category. */
        const USER1 = (1 << 11);
        /** User defined category. */
        const USER2 = (1 << 12);
        /** User defined category. */
        const USER3 = (1 << 13);
        /** User defined category. */
        const USER4 = (1 << 14);
        /** This character cannot be the beginning of an OOV word */
        const NOOOVBOW = (1 << 30);
        /** This and next characters cannot be the beginning of an OOV word */
        const NOOOVBOW2 = (1 << 31);

        /** All categories at once except NOOOVBOW/2 */
        const ALL = 0b00111111_11111111_11111111_11111111;
    }
}

struct CategoryTypesIter {
    values: u32,
}

/// Iterating over individual bitfields (somehow is not automatically implemented)
/// by bitfields crate
impl Iterator for CategoryTypesIter {
    type Item = CategoryTypes;

    fn next(&mut self) -> Option<Self::Item> {
        if self.values == 0 {
            return None;
        }

        let numtz = self.values.trailing_zeros();
        let mask = 1u32 << numtz;
        self.values ^= mask;
        // Safety: it is impossible to call this from user code
        // and we do not produce any new bits here
        Some(unsafe { CategoryTypes::from_bits_unchecked(mask) })
    }
}

impl CategoryTypes {
    pub fn iter(self) -> impl Iterator<Item = CategoryTypes> {
        CategoryTypesIter { values: self.bits }
    }

    pub fn count(self) -> u32 {
        self.bits.count_ones()
    }
}

impl Default for CategoryTypes {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl FromStr for CategoryTypes {
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
            "USER1" => Ok(Self::USER1),
            "USER2" => Ok(Self::USER2),
            "USER3" => Ok(Self::USER3),
            "USER4" => Ok(Self::USER4),
            "NOOOVBOW" => Ok(Self::NOOOVBOW),
            "NOOOVBOW2" => Ok(Self::NOOOVBOW2),
            "ALL" => Ok(Self::ALL),
            _ => Err(anyhow!(
                "InvalidCharacterCategoryTypes, {}",
                String::from(s)
            )),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn format() {
        assert_eq!("GREEK", format!("{:?}", CategoryTypes::GREEK));
        assert_eq!(
            "SPACE | GREEK",
            format!("{:?}", CategoryTypes::GREEK | CategoryTypes::SPACE)
        );
    }

    #[test]
    fn ct_size() {
        assert_eq!(4, std::mem::size_of::<CategoryTypes>())
    }

    #[test]
    fn count() {
        let c1 = CategoryTypes::GREEK | CategoryTypes::KANJI;
        assert_eq!(c1.count(), 2);
        let c2 = CategoryTypes::USER1 | CategoryTypes::USER2 | CategoryTypes::USER3;
        assert_eq!(c2.count(), 3);
    }

    #[test]
    fn iter_is_correct_3() {
        let ct = CategoryTypes::GREEK | CategoryTypes::KANJI | CategoryTypes::USER1;
        let mut iter = ct.iter();
        assert_eq!(iter.next(), Some(CategoryTypes::KANJI));
        assert_eq!(iter.next(), Some(CategoryTypes::GREEK));
        assert_eq!(iter.next(), Some(CategoryTypes::USER1));
        assert_eq!(iter.next(), None);
    }
}
