use std::ops::Range;

use super::Sentence;

// RO Accessors
impl Sentence {
    /// Borrow original data
    pub fn original(&self) -> &str {
        &self.original
    }

    /// Borrow modified data
    pub fn current(&self) -> &str {
        &self.modified
    }

    /// Borrow array of current characters
    pub fn current_chars(&self) -> &[char] {
        &self.mod_chars
    }

    /// Returns byte offsets of current chars
    pub fn curr_byte_offsets(&self) -> &[usize] {
        let len = self.mod_c2b.len();
        &self.mod_c2b[0..len - 1]
    }

    /// Get index of the current byte in original sentence
    /// Bytes not on character boundaries are not supported
    pub fn get_original_index(&self, index: usize) -> usize {
        debug_assert!(self.modified.is_char_boundary(index));
        self.m2o[index]
    }

    /// Mod Char Idx -> Orig Byte Idx
    pub fn to_orig_byte_idx(&self, index: usize) -> usize {
        let byte_idx = self.mod_c2b[index];
        self.m2o[byte_idx]
    }

    /// Mod Char Idx -> Orig Char Idx
    pub fn to_orig_char_idx(&self, index: usize) -> usize {
        let b_idx = self.to_orig_byte_idx(index);
        let res = self.m2o_2[b_idx];
        debug_assert_ne!(res, usize::MAX);
        res
    }

    /// Mod Char Idx -> Mod Byte Idx
    pub fn to_curr_byte_idx(&self, index: usize) -> usize {
        self.mod_c2b[index]
    }

    /// Input: Mod Char Idx
    pub fn curr_slice_c(&self, data: Range<usize>) -> &str {
        let start = self.mod_c2b[data.start];
        let end = self.mod_c2b[data.end];
        &self.modified[start..end]
    }

    /// Input: Mod Char Idx
    pub fn orig_slice_c(&self, data: Range<usize>) -> &str {
        let start = self.to_orig_byte_idx(data.start);
        let end = self.to_orig_byte_idx(data.end);
        &self.original[start..end]
    }

    pub fn ch_idx(&self, idx: usize) -> usize {
        self.mod_b2c[idx]
    }

    /// Return original data as owned, consuming itself    
    pub fn into_original(self) -> String {
        self.original
    }

    /// Whether the byte can start a new word.
    /// Supports bytes not on character boundaries.
    #[inline]
    pub fn can_bow(&self, offset: usize) -> bool {
        self.mod_bow[offset]
    }

    /// Returns char length to the next can_bow point
    ///
    /// Used by SimpleOOV plugin
    pub fn get_word_candidate_length(&self, char_idx: usize) -> usize {
        let char_len = self.mod_chars.len();
        for i in (char_idx + 1)..char_len {
            let byte_idx = self.mod_c2b[i];
            if self.can_bow(byte_idx) {
                return i - char_idx;
            }
        }
        char_len - char_idx
    }
}
