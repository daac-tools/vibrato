use std::simd::{i32x4, usizex4, SimdPartialEq};

use std::collections::BTreeMap;

use bincode::{Decode, Encode};

pub struct ScorerBuilder {
    trie: Vec<BTreeMap<usize, i32>>,
}

impl ScorerBuilder {
    pub fn new() -> Self {
        Self { trie: vec![] }
    }

    pub fn insert(&mut self, key1: usize, key2: usize, weight: i32) {
        if key1 >= self.trie.len() {
            self.trie.resize(key1 + 1, BTreeMap::new());
        }
        self.trie[key1].insert(key2, weight);
    }

    #[inline(always)]
    fn check_base(base: isize, hm: &BTreeMap<usize, i32>, checks: &[usize]) -> bool {
        for &key2 in hm.keys() {
            if let Some(check) = checks.get((base + key2 as isize) as usize) {
                if *check != usize::MAX {
                    return false;
                }
            }
        }
        return true;
    }

    pub fn build(self) -> Scorer {
        let mut bases = vec![0; self.trie.len()];
        let mut checks = vec![];
        let mut weights = vec![];
        let mut cand_first = 1;
        for (key1, hm) in self.trie.into_iter().enumerate() {
            if let Some(key2_head) = hm.keys().next() {
                let mut base = cand_first as isize - *key2_head as isize;
                while !Self::check_base(base, &hm, &checks) {
                    base += 1;
                }
                bases[key1] = base as usize;
                for (key2, weight) in hm {
                    let pos = (base + key2 as isize) as usize;
                    if pos >= checks.len() {
                        checks.resize(pos + 1, usize::MAX);
                        weights.resize(pos + 1, 0);
                    }
                    checks[pos] = key1;
                    weights[pos] = weight;
                }
                while checks[cand_first] != 0 {
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
    bases: Vec<usize>,
    checks: Vec<usize>,
    weights: Vec<i32>,
}

impl Scorer {
    #[inline(always)]
    pub fn retrieve_weight(&self, key1: usize, key2: usize) -> Option<i32> {
        if let Some(base) = self.bases.get(key1) {
            let pos = base.wrapping_add(key2);
            if let Some(check) = self.checks.get(pos) {
                if *check == key1 {
                    Some(self.weights[pos])
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn calculate_score(&self, keys1: &[usize], keys2: &[usize]) -> i32 {
        let mut score = 0;
        for (&key1, &key2) in keys1.iter().zip(keys2) {
            if let Some(w) = self.retrieve_weight(key1, key2) {
                score += w;
            }
        }
        score
    }

    pub fn calculate_score_simd(&self, keys1: &[usize], keys2: &[usize]) -> i32 {
        let mut score = 0;
        let keys1_it = keys1.array_chunks::<4>();
        let keys2_it = keys2.array_chunks::<4>();
        let keys1_remainder = keys1_it.remainder();
        let keys2_remainder = keys2_it.remainder();
        for (keys1_slice, keys2_slice) in keys1_it.zip(keys2_it) {
            let keys1_simd = usizex4::from_slice(keys1_slice);
            let keys2_simd = usizex4::from_slice(keys2_slice);
            let bases_simd = usizex4::gather_or_default(&self.bases, keys1_simd);
            let pos_simd = bases_simd + keys2_simd;
            let checks_simd =
                usizex4::gather_or(&self.checks, pos_simd, usizex4::from([usize::MAX; 4]));
            let filter = checks_simd.simd_eq(keys1_simd);
            let weights_simd =
                i32x4::gather_select(&self.weights, filter, pos_simd, i32x4::from([0; 4]));
            score += weights_simd.to_array().iter().sum::<i32>();
        }
        let remainder_len = keys1_remainder.len();
        if remainder_len != 0 {
            let mut keys1_array = [usize::MAX; 4];
            let mut keys2_array = [usize::MAX; 4];
            keys1_array[..remainder_len].copy_from_slice(keys1_remainder);
            keys2_array[..remainder_len].copy_from_slice(keys2_remainder);
            let keys1_simd = usizex4::from(keys1_array);
            let keys2_simd = usizex4::from(keys2_array);
            let bases_simd = usizex4::gather_or_default(&self.bases, keys1_simd);
            let pos_simd = bases_simd + keys2_simd;
            let checks_simd =
                usizex4::gather_or(&self.checks, pos_simd, usizex4::from([usize::MAX; 4]));
            let filter = checks_simd.simd_eq(keys1_simd);
            let weights_simd =
                i32x4::gather_select(&self.weights, filter, pos_simd, i32x4::from([0; 4]));
            score += weights_simd.to_array().iter().sum::<i32>();
        }
        score
    }
}
