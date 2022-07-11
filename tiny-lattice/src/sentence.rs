#[derive(Default, Clone)]
pub struct Sentence {
    // Characters. Char-based indexing.
    chars: Vec<char>,
    // Char-to-byte mapping for the modified string. Char-based indexing.
    c2b: Vec<usize>,
    // Byte-to-char mapping for the modified string. Byte-based indexing.
    b2c: Vec<usize>,
}

impl Sentence {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_sentence(&mut self, input: &str) {
        self.chars.clear();
        self.c2b.clear();
        self.b2c.clear();
        self.b2c.resize(input.len() + 1, usize::MAX);

        for (ci, (bi, ch)) in input.char_indices().enumerate() {
            self.chars.push(ch);
            self.c2b.push(bi);
            self.b2c[bi] = ci;
        }
        self.c2b.push(input.len());
        self.b2c[input.len()] = self.chars.len();
    }

    pub fn chars(&self) -> &[char] {
        &self.chars
    }

    /// Returns byte offsets of current chars
    pub fn c2b_offsets(&self) -> &[usize] {
        &self.c2b
    }

    pub fn byte_offset(&self, char_offset: usize) -> usize {
        self.c2b[char_offset]
    }

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
        sent.set_sentence("自然");
        assert_eq!(sent.chars(), &['自', '然']);
        assert_eq!(sent.c2b_offsets(), &[0, 3, 6]);
        assert_eq!(sent.char_offset(0), 0);
        assert_eq!(sent.char_offset(3), 1);
        assert_eq!(sent.char_offset(6), 2);
    }
}
