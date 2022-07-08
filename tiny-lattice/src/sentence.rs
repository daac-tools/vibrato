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
    pub fn set_sentence(&mut self, input: &str) {
        self.chars.clear();
        self.c2b.clear();
        self.b2c.clear();
        self.b2c.resize(input.len(), usize::MAX);

        for (ci, (bi, ch)) in input.char_indices().enumerate() {
            self.chars.push(ch);
            self.c2b.push(bi);
            self.b2c[bi] = ci;
        }
    }

    pub fn chars(&self) -> &[char] {
        &self.chars
    }

    /// Returns byte offsets of current chars
    pub fn c2b_offsets(&self) -> &[usize] {
        &self.c2b
    }

    pub fn char_offset(&self, byte_offset: usize) -> usize {
        self.b2c[byte_offset]
    }
}
