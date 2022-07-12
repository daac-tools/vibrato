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
