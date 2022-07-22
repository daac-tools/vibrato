use bincode::{Decode, Encode};

#[derive(Decode, Encode)]
pub struct Trie {
    units: Vec<u32>,
}

impl Trie {
    pub fn new(data: Vec<u8>) -> Self {
        assert_eq!(data.len() % 4, 0);
        let mut units = Vec::with_capacity(data.len() / 4);
        for i in (0..data.len()).step_by(4) {
            let unit = u32::from_le_bytes(data[i..i + 4].try_into().unwrap());
            units.push(unit);
        }
        Self { units }
    }

    #[inline(always)]
    pub fn common_prefix_iterator<'a>(&'a self, input: &'a [u8]) -> CommonPrefixIter<'a> {
        let unit: usize = self.get(0) as usize;
        CommonPrefixIter {
            trie: &self.units,
            node_pos: Self::offset(unit),
            data: input,
            offset: 0,
        }
    }

    #[inline(always)]
    fn get(&self, node_pos: usize) -> u32 {
        debug_assert!(node_pos < self.units.len());
        // UB if out of bounds
        // Should we panic in release builds here instead?
        // Safe version is not optimized away
        *unsafe { self.units.get_unchecked(node_pos) }
    }

    #[inline(always)]
    const fn has_leaf(unit: usize) -> bool {
        ((unit >> 8) & 1) == 1
    }

    #[inline(always)]
    const fn value(unit: u32) -> u32 {
        unit & ((1 << 31) - 1)
    }

    #[inline(always)]
    const fn label(unit: usize) -> usize {
        unit & ((1 << 31) | 0xFF)
    }

    #[inline(always)]
    const fn offset(unit: usize) -> usize {
        (unit >> 10) << ((unit & (1 << 9)) >> 6)
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct TrieMatch {
    pub value: u32,
    pub end_byte: u32,
}

impl TrieMatch {
    #[inline(always)]
    pub const fn new(value: u32, end_byte: u32) -> Self {
        Self { value, end_byte }
    }
}

pub struct CommonPrefixIter<'a> {
    trie: &'a [u32],
    node_pos: usize,
    data: &'a [u8],
    offset: usize,
}

impl<'a> CommonPrefixIter<'a> {
    #[inline(always)]
    fn get(&self, node_pos: usize) -> u32 {
        debug_assert!(node_pos < self.trie.len());
        // UB if out of bounds
        // Should we panic in release builds here instead?
        // Safe version is not optimized away
        *unsafe { self.trie.get_unchecked(node_pos) }
    }
}

impl<'a> Iterator for CommonPrefixIter<'a> {
    type Item = TrieMatch;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let mut node_pos = self.node_pos;
        let mut unit;

        for i in self.offset..self.data.len() {
            // Unwrap is safe: access is always in bounds
            // It is optimized away: https://rust.godbolt.org/z/va9K3az4n
            let k = self.data.get(i).unwrap();
            node_pos ^= *k as usize;
            unit = self.get(node_pos) as usize;
            if Trie::label(unit) != *k as usize {
                return None;
            }

            node_pos ^= Trie::offset(unit);
            if Trie::has_leaf(unit) {
                let r = TrieMatch::new(Trie::value(self.get(node_pos)), (i + 1) as u32);
                self.offset = r.end_byte as usize;
                self.node_pos = node_pos;
                return Some(r);
            }
        }
        None
    }
}
