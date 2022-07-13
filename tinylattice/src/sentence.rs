use crate::dictionary::{CategoryMap, CategoryTypes};
use crate::Morpheme;

#[derive(Default, Clone)]
pub struct Sentence<'a> {
    bytes: &'a [u8],
    chars: Vec<char>,
    c2b: Vec<usize>,
    b2c: Vec<usize>,
    bow: Vec<bool>,
    categories: Vec<CategoryTypes>,
    concatable: Vec<usize>,
    morphs: Vec<Morpheme>,
}

impl<'a> Sentence<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.chars.clear();
        self.c2b.clear();
        self.b2c.clear();
        self.bow.clear();
        self.categories.clear();
        self.concatable.clear();
        self.morphs.clear();
    }

    pub fn set_sentence(&mut self, input: &'a str) {
        self.clear();

        self.bytes = input.as_bytes();
        self.b2c.resize(input.len() + 1, usize::MAX);

        for (ci, (bi, ch)) in input.char_indices().enumerate() {
            self.chars.push(ch);
            self.c2b.push(bi);
            self.b2c[bi] = ci;
        }
        self.c2b.push(input.len());
        self.b2c[input.len()] = self.chars.len();
    }

    pub fn compute_bow(&mut self, cate_map: &CategoryMap) {
        debug_assert!(self.bow.is_empty());

        self.bow.resize(self.bytes.len(), false);

        let non_starting = CategoryTypes::ALPHA | CategoryTypes::GREEK | CategoryTypes::CYRILLIC;
        let mut prev_cate = CategoryTypes::empty();
        let mut next_bow = true;

        for (&ch, &bi) in self.chars.iter().zip(self.c2b.iter()) {
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
    }

    pub fn compute_concatable(&mut self) {
        debug_assert!(!self.chars.is_empty());
        debug_assert_eq!(self.chars.len(), self.categories.len());

        self.concatable.resize(self.chars.len(), 1);
        let mut rhs = self.categories.last().cloned().unwrap();

        for i in (1..self.chars.len()).rev() {
            let lhs = self.categories[i - 1];
            let and = lhs & rhs;
            if !and.is_empty() {
                self.concatable[i - 1] = self.concatable[i] + 1;
                rhs = and;
            } else {
                rhs = lhs;
            }
        }
    }

    #[inline(always)]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[inline(always)]
    pub fn chars(&self) -> &[char] {
        &self.chars
    }

    #[inline(always)]
    pub fn morphs(&self) -> &[Morpheme] {
        &self.morphs
    }

    #[inline(always)]
    pub fn take_morphs(&mut self) -> Vec<Morpheme> {
        std::mem::take(&mut self.morphs)
    }

    #[inline(always)]
    pub fn set_morphs(&mut self, morphs: Vec<Morpheme>) {
        self.morphs = morphs;
    }

    /// Returns byte offsets of current chars
    #[inline(always)]
    pub fn c2b(&self) -> &[usize] {
        &self.c2b
    }

    #[inline(always)]
    pub fn b2c(&self) -> &[usize] {
        &self.b2c
    }

    #[inline(always)]
    pub fn byte_position(&self, char_pos: usize) -> usize {
        self.c2b[char_pos]
    }

    #[inline(always)]
    pub fn char_position(&self, byte_pos: usize) -> usize {
        self.b2c[byte_pos]
    }

    #[inline(always)]
    pub fn category(&self, char_pos: usize) -> CategoryTypes {
        self.categories[char_pos]
    }

    #[inline(always)]
    pub fn concatable(&self, char_pos: usize) -> usize {
        self.concatable[char_pos]
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
        sent.set_sentence("自然");
        assert_eq!(sent.chars(), &['自', '然']);
        assert_eq!(sent.c2b(), &[0, 3, 6]);
        assert_eq!(sent.char_position(0), 0);
        assert_eq!(sent.char_position(3), 1);
        assert_eq!(sent.char_position(6), 2);
    }
}
