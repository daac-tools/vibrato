use std::borrow::Cow;

use crate::dictionary::character::{CharInfo, CharProperty};
use crate::Morpheme;

#[derive(Default, Clone)]
pub struct Sentence<'a> {
    input: Cow<'a, str>,
    chars: Vec<char>,
    c2b: Vec<usize>,
    b2c: Vec<usize>,
    cinfos: Vec<CharInfo>,
    groupable: Vec<usize>,
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
        self.cinfos.clear();
        self.groupable.clear();
        self.morphs.clear();
    }

    pub fn set_sentence(&mut self, input: impl Into<Cow<'a, str>>) {
        self.clear();

        self.input = input.into();
        self.b2c.resize(self.input.len() + 1, usize::MAX);

        for (ci, (bi, ch)) in self.input.char_indices().enumerate() {
            self.chars.push(ch);
            self.c2b.push(bi);
            self.b2c[bi] = ci;
        }
        self.c2b.push(self.input.len());
        self.b2c[self.input.len()] = self.chars.len();
    }

    pub fn compile(&mut self, char_prop: &CharProperty) {
        self.compute_categories(char_prop);
        self.compute_groupable();
    }

    fn compute_categories(&mut self, char_prop: &CharProperty) {
        debug_assert!(!self.chars.is_empty());

        self.cinfos.reserve(self.chars.len());
        for &c in &self.chars {
            self.cinfos.push(char_prop.char_info(c));
        }
    }

    fn compute_groupable(&mut self) {
        debug_assert!(!self.chars.is_empty());
        debug_assert_eq!(self.chars.len(), self.cinfos.len());

        self.groupable.resize(self.chars.len(), 1);
        let mut rhs = self.cinfos.last().unwrap().cate_ids;

        for i in (1..self.chars.len()).rev() {
            let lhs = self.cinfos[i - 1].cate_ids;
            let and = lhs & rhs;
            if !and.is_empty() {
                self.groupable[i - 1] = self.groupable[i] + 1;
                rhs = and;
            } else {
                rhs = lhs;
            }
        }
    }

    #[inline(always)]
    pub fn raw(&self) -> &str {
        &self.input
    }

    #[inline(always)]
    pub fn bytes(&self) -> &[u8] {
        &self.input.as_bytes()
    }

    #[inline(always)]
    pub fn chars(&self) -> &[char] {
        &self.chars
    }

    #[inline(always)]
    pub fn surfaces(&self) -> Vec<&str> {
        self.morphs()
            .iter()
            .map(|m| &self.raw()[m.range_byte()])
            .collect()
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
    /// Including end position
    #[inline(always)]
    pub fn c2b(&self) -> &[usize] {
        &self.c2b
    }

    #[inline(always)]
    pub fn b2c(&self) -> &[usize] {
        &self.b2c
    }

    #[inline(always)]
    pub fn byte_position(&self, pos_char: usize) -> usize {
        self.c2b[pos_char]
    }

    #[inline(always)]
    pub fn char_position(&self, pos_byte: usize) -> usize {
        self.b2c[pos_byte]
    }

    #[inline(always)]
    pub fn char_info(&self, pos_char: usize) -> CharInfo {
        self.cinfos[pos_char]
    }

    #[inline(always)]
    pub fn groupable(&self, pos_char: usize) -> usize {
        self.groupable[pos_char]
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
