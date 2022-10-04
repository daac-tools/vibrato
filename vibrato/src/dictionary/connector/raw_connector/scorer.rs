use std::collections::BTreeMap;

use bincode::{Decode, Encode};

use crate::utils::FromU32;

const UNUSED_POS: u32 = u32::MAX;

pub struct ScorerBuilder {
    // Two-level trie mapping a pair of two keys into a cost, where
    // the first level stores the first key, and the second level stores the second key.
    trie: Vec<BTreeMap<u32, i32>>,
}

impl ScorerBuilder {
    pub const fn new() -> Self {
        Self { trie: vec![] }
    }

    pub fn insert(&mut self, key1: u32, key2: u32, cost: i32) {
        let key1 = usize::from_u32(key1);
        if key1 >= self.trie.len() {
            self.trie.resize(key1 + 1, BTreeMap::new());
        }
        self.trie[key1].insert(key2, cost);
    }

    #[inline(always)]
    fn check_base(base: i32, second_map: &BTreeMap<u32, i32>, checks: &[u32]) -> bool {
        for &key2 in second_map.keys() {
            if let Some(check) = checks.get((base + key2 as i32) as usize) {
                if *check != UNUSED_POS {
                    return false;
                }
            }
        }
        true
    }

    pub fn build(self) -> Scorer {
        let mut bases = vec![0; self.trie.len()];
        let mut checks = vec![];
        let mut costs = vec![];
        let mut cand_first = 1;
        for (key1, second_map) in self.trie.into_iter().enumerate() {
            if let Some(key2_head) = second_map.keys().next() {
                let mut base = cand_first as i32 - *key2_head as i32;
                while !Self::check_base(base, &second_map, &checks) {
                    base += 1;
                }
                bases[key1] = base;
                for (key2, cost) in second_map {
                    let pos = (base + key2 as i32) as u32;
                    let pos = usize::from_u32(pos);
                    if pos >= checks.len() {
                        checks.resize(pos + 1, UNUSED_POS);
                        costs.resize(pos + 1, 0);
                    }
                    checks[pos] = u32::try_from(key1).unwrap();
                    costs[pos] = cost;
                }
                while checks.get(cand_first).copied().unwrap_or(UNUSED_POS) != UNUSED_POS {
                    cand_first += 1;
                }
            }
        }
        Scorer {
            bases,
            checks,
            costs,
        }
    }
}

#[derive(Decode, Encode, Default)]
pub struct Scorer {
    bases: Vec<i32>,
    checks: Vec<u32>,
    costs: Vec<i32>,
}

impl Scorer {
    #[inline(always)]
    fn retrieve_cost(&self, key1: u32, key2: u32) -> Option<i32> {
        if let Some(base) = self.bases.get(usize::from_u32(key1)) {
            let pos = (base + key2 as i32) as u32;
            let pos = usize::from_u32(pos);
            if let Some(check) = self.checks.get(pos) {
                if *check == key1 {
                    return Some(self.costs[pos]);
                }
            }
        }
        None
    }

    #[inline(always)]
    pub fn accumulate_cost(&self, keys1: &[u32], keys2: &[u32]) -> i32 {
        let mut score = 0;
        for (&key1, &key2) in keys1.iter().zip(keys2) {
            if let Some(w) = self.retrieve_cost(key1, key2) {
                score += w;
            }
        }
        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retrieve_cost_test() {
        let mut builder = ScorerBuilder::new();
        builder.insert(18, 17, 1);
        builder.insert(4, 9, 2);
        builder.insert(17, 0, 3);
        builder.insert(17, 12, 4);
        builder.insert(8, 6, 5);
        builder.insert(2, 5, 6);
        builder.insert(12, 18, 7);
        builder.insert(9, 1, 8);
        builder.insert(19, 5, 9);
        builder.insert(9, 4, 10);
        builder.insert(0, 19, 11);
        builder.insert(2, 19, 12);
        builder.insert(7, 9, 13);
        builder.insert(18, 9, 14);
        builder.insert(17, 4, 15);
        builder.insert(9, 6, 16);
        builder.insert(13, 0, 17);
        builder.insert(1, 4, 18);
        builder.insert(0, 18, 19);
        builder.insert(18, 11, 20);
        let scorer = builder.build();

        assert_eq!(scorer.retrieve_cost(0, 18), Some(19));
        assert_eq!(scorer.retrieve_cost(0, 19), Some(11));
        assert_eq!(scorer.retrieve_cost(9, 4), Some(10));
        assert_eq!(scorer.retrieve_cost(9, 6), Some(16));
        assert_eq!(scorer.retrieve_cost(0, 0), None);
        assert_eq!(scorer.retrieve_cost(9, 5), None);
    }

    #[test]
    fn accumulate_cost_test() {
        let mut builder = ScorerBuilder::new();
        builder.insert(18, 17, 1);
        builder.insert(4, 9, 2);
        builder.insert(17, 0, 3);
        builder.insert(17, 12, 4);
        builder.insert(8, 6, 5);
        builder.insert(2, 5, 6);
        builder.insert(12, 18, 7);
        builder.insert(9, 1, 8);
        builder.insert(19, 5, 9);
        builder.insert(9, 4, 10);
        builder.insert(0, 19, 11);
        builder.insert(2, 19, 12);
        builder.insert(7, 9, 13);
        builder.insert(18, 9, 14);
        builder.insert(17, 4, 15);
        builder.insert(9, 6, 16);
        builder.insert(13, 0, 17);
        builder.insert(1, 4, 18);
        builder.insert(0, 18, 19);
        builder.insert(18, 11, 20);
        let scorer = builder.build();

        assert_eq!(
            scorer.accumulate_cost(
                &[18, 17, 0, 8, 12, 19, 9, 0, 7, 17, 13, 0],
                &[17, 0, 0, 6, 18, 5, 9, 19, 9, 4, 0, 18]
            ),
            100
        );
    }
}
