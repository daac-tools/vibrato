use crate::category::{CategoryTable, CategoryTypes};

#[derive(Default, Clone)]
pub struct Sentence {
    // Characters. Char-based indexing.
    chars: Vec<char>,
    // Char-to-byte mapping for the modified string. Char-based indexing.
    c2b: Vec<usize>,
    // Byte-to-char mapping for the modified string. Byte-based indexing.
    b2c: Vec<usize>,
    // Markers whether the byte can start new word or not
    bow: Vec<bool>,
}

impl Sentence {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.chars.clear();
        self.c2b.clear();
        self.b2c.clear();
        self.bow.clear();
    }

    pub fn set_sentence(&mut self, input: &str, cate_table: &CategoryTable) {
        self.clear();

        self.b2c.resize(input.len() + 1, usize::MAX);
        self.bow.resize(input.len(), false);

        // Special cases for BOW logic
        let non_starting = CategoryTypes::ALPHA | CategoryTypes::GREEK | CategoryTypes::CYRILLIC;
        let mut prev_cate = CategoryTypes::empty();
        let mut next_bow = true;

        for (ci, (bi, ch)) in input.char_indices().enumerate() {
            self.chars.push(ch);
            self.c2b.push(bi);
            self.b2c[bi] = ci;

            let cate = cate_table.get_category_types(ch);
            let can_bow = if !next_bow {
                // this char was forbidden by the previous one
                next_bow = true;
                false
            } else if cate.intersects(CategoryTypes::NOOOVBOW2) {
                // this rule is stronger than the next one and must come before
                // this and next are forbidden
                next_bow = false;
                false
            } else if cate.intersects(CategoryTypes::NOOOVBOW) {
                // this char is forbidden
                false
            } else if cate.intersects(non_starting) {
                // the previous char is compatible
                !cate.intersects(prev_cate)
            } else {
                true
            };
            self.bow[bi] = can_bow;
            prev_cate = cate;
        }

        self.c2b.push(input.len());
        self.b2c[input.len()] = self.chars.len();
    }

    #[inline(always)]
    pub fn chars(&self) -> &[char] {
        &self.chars
    }

    /// Returns byte offsets of current chars
    #[inline(always)]
    pub fn c2b_offsets(&self) -> &[usize] {
        &self.c2b
    }

    #[inline(always)]
    pub fn byte_offset(&self, char_offset: usize) -> usize {
        self.c2b[char_offset]
    }

    #[inline(always)]
    pub fn char_offset(&self, byte_offset: usize) -> usize {
        self.b2c[byte_offset]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sentence() {
        let mut sent = Sentence::new();
        sent.set_sentence("自然", &CategoryTable::default());
        assert_eq!(sent.chars(), &['自', '然']);
        assert_eq!(sent.c2b_offsets(), &[0, 3, 6]);
        assert_eq!(sent.char_offset(0), 0);
        assert_eq!(sent.char_offset(3), 1);
        assert_eq!(sent.char_offset(6), 2);
    }
}
