use std::collections::BTreeSet;

use anyhow::{anyhow, Result};

use super::CategoryTypes;

#[derive(Debug, Clone, PartialEq, Eq)]
struct CategoryRange {
    begin: u32,
    end: u32,
    categories: CategoryTypes,
}

/// CategoryMap holds mapping from character to character category type
#[derive(Debug, Clone)]
pub struct CategoryMap {
    /// Split the whole domain of codepoints into ranges,
    /// limited by boundaries.
    ///
    /// Ranges are half-open: `[boundaries[i], boundaries[i + 1])`
    /// meaning that the right bound is not included.
    /// 0 and u32::MAX are not stored, they are included implicitly
    /// as if they would have indices of `-1` and `boundaries.len()`.
    boundaries: Vec<u32>,

    /// Stores the category for each range.
    /// `categories[i]` is for the range `[boundaries[i - 1], boundaries[i])`.
    /// Plays well with [`std::slice::binary_search`], see [`get_category_types()`].
    /// This should be always true: `boundaries.len() + 1 == categories.len()`.
    categories: Vec<CategoryTypes>,
}

impl Default for CategoryMap {
    fn default() -> Self {
        CategoryMap {
            boundaries: Vec::new(),
            categories: vec![CategoryTypes::DEFAULT],
        }
    }
}

impl CategoryMap {
    /// Creates a character category from file
    pub fn from_lines<I, L>(lines: I) -> Result<Self>
    where
        I: IntoIterator<Item = L>,
        L: AsRef<str>,
    {
        let ranges = Self::parse_character_definition(lines)?;
        Ok(Self::compile(&ranges))
    }

    /// Reads character type definition as a list of Ranges
    ///
    /// Definition file syntax:
    ///     Each line contains [TARGET_CHARACTER_CODE_POINT] [TYPES], where
    ///     TARGET_CHARACTER_CODE_POINT:
    ///         a code_point in hexadecimal format or two separated by ".."
    ///     TYPES:
    ///         one or more Category_types separated by white space
    ///     Loads only lines start with "0x" are loaded and ignore others
    ///
    /// Definition example:
    ///     "0x0030..0x0039 NUMERIC"
    ///     "0x3008         KANJI KANJINUMERIC"
    fn parse_character_definition<I, L>(lines: I) -> Result<Vec<CategoryRange>>
    where
        I: IntoIterator<Item = L>,
        L: AsRef<str>,
    {
        let mut ranges = vec![];

        for line in lines {
            let line = line.as_ref();
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') || !line.starts_with("0x") {
                continue;
            }

            let cols: Vec<_> = line.split_whitespace().collect();
            if cols.len() < 2 {
                return Err(anyhow!("InvalidFormat: {}", line));
            }

            let r: Vec<_> = cols[0].split("..").collect();
            let begin = u32::from_str_radix(String::from(r[0]).trim_start_matches("0x"), 16)?;
            let end = if r.len() > 1 {
                u32::from_str_radix(String::from(r[1]).trim_start_matches("0x"), 16)? + 1
            } else {
                begin + 1
            };
            if begin >= end {
                return Err(anyhow!("InvalidFormat: {}", line));
            }
            if char::from_u32(begin).is_none() {
                return Err(anyhow!("InvalidFormat: {}", line));
            }

            if char::from_u32(end).is_none() {
                return Err(anyhow!("InvalidFormat: {}", line));
            }

            let mut categories = CategoryTypes::empty();
            for elem in cols[1..]
                .iter()
                .take_while(|elem| elem.chars().next().unwrap() != '#')
            {
                categories.insert(match elem.parse() {
                    Ok(t) => t,
                    Err(_) => {
                        return Err(anyhow!("InvalidFormat: {}", line));
                    }
                });
            }

            ranges.push(CategoryRange {
                begin,
                end,
                categories,
            });
        }

        Ok(ranges)
    }

    /// Creates a character category from given range_list
    ///
    /// Transforms given range_list to non overlapped range list
    /// to apply binary search in get_category_types
    fn compile(ranges: &[CategoryRange]) -> Self {
        if ranges.is_empty() {
            return Self::default();
        }

        let boundaries = Self::collect_boundaries(ranges);
        let mut categories = vec![CategoryTypes::empty(); boundaries.len()];

        for range in ranges {
            let start_idx = match boundaries.binary_search(&range.begin) {
                Ok(i) => i + 1,
                Err(_) => panic!("there can not be not found boundaries"),
            };
            // apply category to all splits which are included in the current range
            for i in start_idx..boundaries.len() {
                if boundaries[i] > range.end {
                    break;
                }
                categories[i] |= range.categories;
            }
        }

        // first category is always default (it is impossible to get it assigned above)
        debug_assert_eq!(categories[0], CategoryTypes::empty());
        categories[0] = CategoryTypes::DEFAULT;
        // merge successive ranges of the same category
        let mut final_boundaries = Vec::with_capacity(boundaries.len());
        let mut final_categories = Vec::with_capacity(categories.len());

        let mut last_category = categories[0];
        let mut last_boundary = boundaries[0];
        for i in 1..categories.len() {
            if categories[i] == last_category {
                last_boundary = boundaries[i];
                continue;
            }
            final_boundaries.push(last_boundary);
            final_categories.push(last_category);
            last_category = categories[i];
            last_boundary = boundaries[i];
        }

        final_categories.push(last_category);
        final_boundaries.push(last_boundary);

        // replace empty categories with default
        for cat in final_categories.iter_mut() {
            if cat.is_empty() {
                *cat = CategoryTypes::DEFAULT;
            }
        }

        // and add the category after the last boundary
        final_categories.push(CategoryTypes::DEFAULT);

        final_boundaries.shrink_to_fit();
        final_categories.shrink_to_fit();

        Self {
            boundaries: final_boundaries,
            categories: final_categories,
        }
    }

    /// Find sorted list of all boundaries
    fn collect_boundaries(data: &[CategoryRange]) -> Vec<u32> {
        let mut boundaries = BTreeSet::new();
        for i in data {
            boundaries.insert(i.begin);
            boundaries.insert(i.end);
        }
        boundaries.into_iter().collect()
    }

    /// Returns a set of category types which given char has
    pub fn get_category_types(&self, c: char) -> CategoryTypes {
        if self.boundaries.is_empty() {
            return CategoryTypes::DEFAULT;
        }
        let cint = c as u32;
        match self.boundaries.binary_search(&cint) {
            //Ok means the index in boundaries, so the next category
            Ok(idx) => self.categories[idx + 1],
            //Err means the insertion index, so the current category
            Err(idx) => self.categories[idx],
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use claim::assert_matches;
//     use std::path::PathBuf;

//     const TEST_RESOURCE_DIR: &str = "./tests/resources/";
//     const TEST_CHAR_DEF_FILE: &str = "char.def";

//     #[test]
//     fn get_category_types() {
//         let path = PathBuf::from(TEST_RESOURCE_DIR).join(TEST_CHAR_DEF_FILE);
//         let cat = CategoryMap::from_file(&path).expect("failed to load char.def for test");
//         let cats = cat.get_category_types('熙');
//         assert_eq!(1, cats.count());
//         assert!(cats.contains(CategoryType::KANJI));
//     }

//     fn read_categories(data: &str) -> CategoryMap {
//         let ranges = CategoryMap::read_character_definition(data.as_bytes())
//             .expect("error when parsing character categories");
//         CategoryMap::compile(&ranges)
//     }

//     type CT = CategoryType;

//     #[test]
//     fn read_cdef_1() {
//         let cat = read_categories(
//             "
//             0x0030..0x0039 NUMERIC
//             0x0032         KANJI",
//         );
//         assert_eq!(cat.get_category_types('\u{0030}'), CT::NUMERIC);
//         assert_eq!(cat.get_category_types('\u{0031}'), CT::NUMERIC);
//         assert_eq!(cat.get_category_types('\u{0032}'), CT::NUMERIC | CT::KANJI);
//         assert_eq!(cat.get_category_types('\u{0033}'), CT::NUMERIC);
//         assert_eq!(cat.get_category_types('\u{0039}'), CT::NUMERIC);
//     }

//     #[test]
//     fn read_cdef_2() {
//         let cat = read_categories(
//             "
//             0x0030..0x0039 NUMERIC
//             0x0070..0x0079 ALPHA
//             0x3007         KANJI
//             0x0030         KANJI",
//         );
//         assert_eq!(cat.get_category_types('\u{0030}'), CT::NUMERIC | CT::KANJI);
//         assert_eq!(cat.get_category_types('\u{0039}'), CT::NUMERIC);
//         assert_eq!(cat.get_category_types('\u{3007}'), CT::KANJI);
//         assert_eq!(cat.get_category_types('\u{0069}'), CT::DEFAULT);
//         assert_eq!(cat.get_category_types('\u{0070}'), CT::ALPHA);
//         assert_eq!(cat.get_category_types('\u{0080}'), CT::DEFAULT);
//     }

//     #[test]
//     fn read_cdef_3() {
//         let cat = read_categories(
//             "
//             0x0030..0x0039 KATAKANA
//             0x3007         KANJI KANJINUMERIC
//             0x3008         KANJI KANJINUMERIC
//             0x3009         KANJI KANJINUMERIC
//             0x0039..0x0040 ALPHA
//             0x0030..0x0039 NUMERIC
//             0x0030         KANJI",
//         );
//         assert_eq!(cat.get_category_types('\u{0029}'), CT::DEFAULT);
//         assert_eq!(
//             cat.get_category_types('\u{0030}'),
//             CT::NUMERIC | CT::KATAKANA | CT::KANJI
//         );
//         assert_eq!(
//             cat.get_category_types('\u{0039}'),
//             CT::NUMERIC | CT::ALPHA | CT::KATAKANA
//         );
//         assert_eq!(cat.get_category_types('\u{0040}'), CT::ALPHA);
//         assert_eq!(cat.get_category_types('\u{0041}'), CT::DEFAULT);
//         assert_eq!(
//             cat.get_category_types('\u{3007}'),
//             CT::KANJI | CT::KANJINUMERIC
//         );
//         assert_eq!(cat.get_category_types('\u{4007}'), CT::DEFAULT);
//     }

//     #[test]
//     fn read_cdef_4() {
//         let cat = read_categories(
//             "
//             0x4E00..0x9FFF KANJI
//             0x4E8C         KANJI KANJINUMERIC",
//         );
//         assert_eq!(cat.get_category_types('男'), CT::KANJI);
//         assert_eq!(cat.get_category_types('\u{4E8B}'), CT::KANJI);
//         assert_eq!(
//             cat.get_category_types('\u{4E8C}'),
//             CT::KANJI | CT::KANJINUMERIC
//         );
//         assert_eq!(cat.get_category_types('\u{4E8D}'), CT::KANJI);
//     }

//     #[test]
//     fn read_cdef_holes_1() {
//         let cat = read_categories(
//             "
//             0x0030 USER1
//             0x0032 USER2",
//         );
//         assert_eq!(cat.get_category_types('\u{0029}'), CT::DEFAULT);
//         assert_eq!(cat.get_category_types('\u{0030}'), CT::USER1);
//         assert_eq!(cat.get_category_types('\u{0031}'), CT::DEFAULT);
//         assert_eq!(cat.get_category_types('\u{0032}'), CT::USER2);
//         assert_eq!(cat.get_category_types('\u{0033}'), CT::DEFAULT);
//     }

//     #[test]
//     fn read_cdef_merge_1() {
//         let cat = read_categories(
//             "
//             0x0030 USER1
//             0x0031 USER1",
//         );
//         assert_eq!(cat.boundaries.len(), 2);
//         assert_eq!(cat.categories.len(), 3);
//         assert_eq!(cat.get_category_types('\u{0029}'), CT::DEFAULT);
//         assert_eq!(cat.get_category_types('\u{0030}'), CT::USER1);
//         assert_eq!(cat.get_category_types('\u{0031}'), CT::USER1);
//         assert_eq!(cat.get_category_types('\u{0032}'), CT::DEFAULT);
//     }

//     #[test]
//     fn read_cdef_merge_2() {
//         let cat = read_categories(
//             "
//             0x0030          USER1
//             0x0031..0x0032  USER1",
//         );
//         assert_eq!(cat.boundaries.len(), 2);
//         assert_eq!(cat.categories.len(), 3);
//         assert_eq!(cat.get_category_types('\u{0029}'), CT::DEFAULT);
//         assert_eq!(cat.get_category_types('\u{0030}'), CT::USER1);
//         assert_eq!(cat.get_category_types('\u{0031}'), CT::USER1);
//         assert_eq!(cat.get_category_types('\u{0032}'), CT::USER1);
//         assert_eq!(cat.get_category_types('\u{0033}'), CT::DEFAULT);
//     }

//     #[test]
//     fn read_cdef_merge_3() {
//         let cat = read_categories(
//             "
//             0x0030..0x0031  USER1
//             0x0032..0x0033  USER1",
//         );
//         assert_eq!(cat.boundaries.len(), 2);
//         assert_eq!(cat.categories.len(), 3);
//         assert_eq!(cat.get_category_types('\u{0029}'), CT::DEFAULT);
//         assert_eq!(cat.get_category_types('\u{0030}'), CT::USER1);
//         assert_eq!(cat.get_category_types('\u{0031}'), CT::USER1);
//         assert_eq!(cat.get_category_types('\u{0032}'), CT::USER1);
//         assert_eq!(cat.get_category_types('\u{0033}'), CT::USER1);
//         assert_eq!(cat.get_category_types('\u{0034}'), CT::DEFAULT);
//     }

//     #[test]
//     fn read_character_definition_with_invalid_format() {
//         let data = "0x0030..0x0039";
//         let result = CategoryMap::read_character_definition(data.as_bytes());
//         assert_matches!(
//             result,
//             Err(SudachiError::InvalidCategoryMap(
//                 Error::InvalidFormat(0)
//             ))
//         );
//     }

//     #[test]
//     fn read_character_definition_with_invalid_range() {
//         let data = "0x0030..0x0029 NUMERIC";
//         let result = CategoryMap::read_character_definition(data.as_bytes());
//         assert_matches!(
//             result,
//             Err(SudachiError::InvalidCategoryMap(
//                 Error::InvalidFormat(0)
//             ))
//         );
//     }

//     #[test]
//     fn read_character_definition_with_invalid_type() {
//         let data = "0x0030..0x0039 FOO";
//         let result = CategoryMap::read_character_definition(data.as_bytes());
//         assert_matches!(result, Err(SudachiError::InvalidCategoryMap(Error::InvalidCategoryType(0, s))) if s == "FOO");
//     }

//     #[test]
//     fn check_test_cdef() {
//         let data: &[u8] = include_bytes!("../../tests/resources/char.def");
//         let c = CategoryMap::from_reader(data).expect("failed to read chars");
//         assert_eq!(c.get_category_types('â'), CT::ALPHA);
//         assert_eq!(c.get_category_types('ｂ'), CT::ALPHA);
//         assert_eq!(c.get_category_types('C'), CT::ALPHA);
//         assert_eq!(c.get_category_types('漢'), CT::KANJI);
//         assert_eq!(c.get_category_types('𡈽'), CT::DEFAULT);
//         assert_eq!(c.get_category_types('ア'), CT::KATAKANA);
//         assert_eq!(c.get_category_types('ｺ'), CT::KATAKANA);
//         assert_eq!(c.get_category_types('ﾞ'), CT::KATAKANA);
//     }

//     #[test]
//     fn iter_cdef_holes_1() {
//         let cat = read_categories(
//             "
//             0x0030 USER1
//             0x0032 USER2",
//         );
//         let mut iter = cat.iter();
//         assert_matches!(
//             iter.next(),
//             Some((
//                 Range {
//                     start: '\x00',
//                     end: '\x30'
//                 },
//                 CT::DEFAULT
//             ))
//         );
//         assert_matches!(
//             iter.next(),
//             Some((
//                 Range {
//                     start: '\x30',
//                     end: '\x31'
//                 },
//                 CT::USER1
//             ))
//         );
//         assert_matches!(
//             iter.next(),
//             Some((
//                 Range {
//                     start: '\x31',
//                     end: '\x32'
//                 },
//                 CT::DEFAULT
//             ))
//         );
//         assert_matches!(
//             iter.next(),
//             Some((
//                 Range {
//                     start: '\x32',
//                     end: '\x33'
//                 },
//                 CT::USER2
//             ))
//         );
//         assert_matches!(
//             iter.next(),
//             Some((
//                 Range {
//                     start: '\x33',
//                     end: char::MAX
//                 },
//                 CT::DEFAULT
//             ))
//         );
//         assert_eq!(iter.next(), None);
//     }
// }
