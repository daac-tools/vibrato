#[derive(Debug, Eq, PartialEq, Clone)]
pub struct TrieEntry {
    /// Value of Trie, this is not the pointer to WordId, but the offset in WordId table
    pub value: u32,
    /// Offset of word end
    pub end: usize,
}

impl TrieEntry {
    #[inline]
    pub fn new(value: u32, offset: usize) -> TrieEntry {
        TrieEntry { value, end: offset }
    }
}

pub struct Trie {
    units: Vec<u32>,
}

impl Trie {
    pub fn new(data: Vec<u8>) -> Self {
        assert_eq!(data.len() % 4, 0);
        let len = data.len() / 4;
        let mut units = Vec::with_capacity(len);
        for i in 0..len {
            let unit = u32::from_le_bytes(data[i * 4..i * 4 + 4].try_into().unwrap());
            units.push(unit);
        }
        Self { units }
    }

    #[inline]
    pub fn common_prefix_iterator<'a>(
        &'a self,
        input: &'a [u8],
        offset: usize,
    ) -> TrieEntryIter<'a> {
        let unit: usize = self.get(0) as usize;
        TrieEntryIter {
            trie: &self.units,
            node_pos: Trie::offset(unit),
            data: input,
            offset,
        }
    }

    #[inline(always)]
    fn get(&self, index: usize) -> u32 {
        debug_assert!(index < self.units.len());
        // UB if out of bounds
        // Should we panic in release builds here instead?
        // Safe version is not optimized away
        *unsafe { self.units.get_unchecked(index) }
    }

    #[inline(always)]
    fn has_leaf(unit: usize) -> bool {
        ((unit >> 8) & 1) == 1
    }

    #[inline(always)]
    fn value(unit: u32) -> u32 {
        unit & ((1 << 31) - 1)
    }

    #[inline(always)]
    fn label(unit: usize) -> usize {
        unit & ((1 << 31) | 0xFF)
    }

    #[inline(always)]
    fn offset(unit: usize) -> usize {
        (unit >> 10) << ((unit & (1 << 9)) >> 6)
    }
}

pub struct TrieEntryIter<'a> {
    trie: &'a [u32],
    node_pos: usize,
    data: &'a [u8],
    offset: usize,
}

impl<'a> TrieEntryIter<'a> {
    #[inline(always)]
    fn get(&self, index: usize) -> u32 {
        debug_assert!(index < self.trie.len());
        // UB if out of bounds
        // Should we panic in release builds here instead?
        // Safe version is not optimized away
        *unsafe { self.trie.get_unchecked(index) }
    }
}

impl<'a> Iterator for TrieEntryIter<'a> {
    type Item = TrieEntry;

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
                let r = TrieEntry::new(Trie::value(self.get(node_pos)), i + 1);
                self.offset = r.end;
                self.node_pos = node_pos;
                return Some(r);
            }
        }
        None
    }
}
