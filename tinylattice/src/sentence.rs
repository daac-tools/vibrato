use crate::dictionary::{CategoryMap, CategoryTypes};

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

    #[inline(always)]
    pub fn clear(&mut self) {
        self.chars.clear();
        self.c2b.clear();
        self.b2c.clear();
        self.bow.clear();
    }

    pub fn set_sentence(&mut self, input: &str, cate_map: &CategoryMap) {
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

            let cate = cate_map.get_category_types(ch);
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
    pub fn c2b(&self) -> &[usize] {
        &self.c2b
    }

    #[inline(always)]
    pub fn byte_position(&self, char_pos: usize) -> usize {
        self.c2b[char_pos]
    }

    #[inline(always)]
    pub fn char_position(&self, byte_pos: usize) -> usize {
        self.b2c[byte_pos]
    }

    /// Whether the byte can start a new word.
    /// Supports bytes not on character boundaries.
    #[inline(always)]
    pub fn can_bow(&self, byte_pos: usize) -> bool {
        self.bow[byte_pos]
    }

    /// Returns char length to the next can_bow point
    ///
    /// Used by SimpleOOV plugin
    #[inline(always)]
    pub fn get_word_candidate_length(&self, char_pos: usize) -> usize {
        for i in (char_pos + 1)..self.chars.len() {
            let byte_pos = self.c2b[i];
            if self.can_bow(byte_pos) {
                return i - char_pos;
            }
        }
        self.chars.len() - char_pos
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sentence() {
        let mut sent = Sentence::new();
        sent.set_sentence("自然", &CategoryMap::default());
        assert_eq!(sent.chars(), &['自', '然']);
        assert_eq!(sent.c2b(), &[0, 3, 6]);
        assert_eq!(sent.char_position(0), 0);
        assert_eq!(sent.char_position(3), 1);
        assert_eq!(sent.char_position(6), 2);
    }
}
