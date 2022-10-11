use crate::common::MAX_SENTENCE_LENGTH;
use crate::dictionary::character::{CharInfo, CharProperty};
use crate::errors::{Result, VibratoError};

#[derive(Default, Clone, Debug)]
pub struct Sentence {
    input: String,
    chars: Vec<char>,
    c2b: Vec<usize>,
    cinfos: Vec<CharInfo>,
    groupable: Vec<usize>,
}

impl Sentence {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.input.clear();
        self.chars.clear();
        self.c2b.clear();
        self.cinfos.clear();
        self.groupable.clear();
    }

    pub fn set_sentence<S>(&mut self, input: S)
    where
        S: AsRef<str>,
    {
        self.clear();
        self.input.push_str(input.as_ref());
    }

    pub fn compile(&mut self, char_prop: &CharProperty) -> Result<()> {
        self.compute_basic();
        self.compute_categories(char_prop);
        self.compute_groupable();
        Ok(())
    }

    fn compute_basic(&mut self) {
        for (bi, ch) in self.input.char_indices() {
            self.chars.push(ch);
            self.c2b.push(bi);
        }
        self.c2b.push(self.input.len());
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
        let mut rhs = self.cinfos.last().unwrap().cate_idset();

        for i in (1..self.chars.len()).rev() {
            let lhs = self.cinfos[i - 1].cate_idset();
            if (lhs & rhs) != 0 {
                self.groupable[i - 1] = self.groupable[i] + 1;
            }
            rhs = lhs;
        }
    }

    #[inline(always)]
    pub fn raw(&self) -> &str {
        &self.input
    }

    #[inline(always)]
    pub fn chars(&self) -> &[char] {
        &self.chars
    }

    #[inline(always)]
    pub fn len_char(&self) -> usize {
        self.chars.len()
    }

    #[inline(always)]
    pub fn byte_position(&self, pos_char: usize) -> usize {
        self.c2b[pos_char]
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
        sent.compute_basic();
        assert_eq!(sent.chars(), &['自', '然']);
        assert_eq!(sent.byte_position(0), 0);
        assert_eq!(sent.byte_position(1), 3);
        assert_eq!(sent.byte_position(2), 6);
    }
}
