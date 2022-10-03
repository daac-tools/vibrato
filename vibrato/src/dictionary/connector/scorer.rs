use std::collections::BTreeMap;

use bincode::{Decode, Encode};

use crate::utils::FromU32;

const UNUSED_POS: u32 = u32::MAX;

pub struct ScorerBuilder {
    trie: Vec<BTreeMap<u32, i32>>,
}

impl ScorerBuilder {
    pub const fn new() -> Self {
        Self { trie: vec![] }
    }

    pub fn insert(&mut self, key1: u32, key2: u32, weight: i32) {
        let key1 = usize::from_u32(key1);
        if key1 >= self.trie.len() {
            self.trie.resize(key1 + 1, BTreeMap::new());
        }
        self.trie[key1].insert(key2, weight);
    }

    #[inline(always)]
    fn check_base(base: i32, hm: &BTreeMap<u32, i32>, checks: &[u32]) -> bool {
        for &key2 in hm.keys() {
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
        let mut weights = vec![];
        let mut cand_first = 1;
        for (key1, hm) in self.trie.into_iter().enumerate() {
            if let Some(key2_head) = hm.keys().next() {
                let mut base = cand_first as i32 - *key2_head as i32;
                while !Self::check_base(base, &hm, &checks) {
                    base += 1;
                }
                bases[key1] = base as u32;
                for (key2, weight) in hm {
                    let pos = (base + key2 as i32) as u32;
                    let pos = usize::from_u32(pos);
                    if pos >= checks.len() {
                        checks.resize(pos + 1, UNUSED_POS);
                        weights.resize(pos + 1, 0);
                    }
                    checks[pos] = u32::try_from(key1).unwrap();
                    weights[pos] = weight;
                }
                while checks.get(cand_first).copied().unwrap_or(UNUSED_POS) != UNUSED_POS {
                    cand_first += 1;
                }
            }
        }
        Scorer {
            bases,
            checks,
            weights,
        }
    }
}

#[derive(Decode, Encode, Default)]
pub struct Scorer {
    bases: Vec<u32>,
    checks: Vec<u32>,
    weights: Vec<i32>,
}

impl Scorer {
    #[inline(always)]
    pub fn retrieve_weight(&self, key1: u32, key2: u32) -> Option<i32> {
        if let Some(base) = self.bases.get(usize::from_u32(key1)) {
            let pos = usize::from_u32(base.wrapping_add(key2));
            if let Some(check) = self.checks.get(pos) {
                if *check == key1 {
                    return Some(self.weights[pos]);
                }
            }
        }
        None
    }

    #[inline(always)]
    pub fn calculate_score(&self, keys1: &[u32], keys2: &[u32]) -> i32 {
        let mut score = 0;
        for (&key1, &key2) in keys1.iter().zip(keys2) {
            if let Some(w) = self.retrieve_weight(key1, key2) {
                score += w;
            }
        }
        score
    }
}
