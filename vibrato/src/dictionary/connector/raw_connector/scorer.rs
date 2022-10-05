use std::collections::BTreeMap;

#[cfg(target_feature = "avx2")]
use std::arch::x86_64::{self, __m256i};

use bincode::{Decode, Encode};

use crate::utils::FromU32;

const UNUSED_CHECK: u32 = u32::MAX;

pub const SIMD_SIZE: usize = 8;

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
                if *check != UNUSED_CHECK {
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
                        checks.resize(pos + 1, UNUSED_CHECK);
                        costs.resize(pos + 1, 0);
                    }
                    checks[pos] = u32::try_from(key1).unwrap();
                    costs[pos] = cost;
                }
                while checks.get(cand_first).copied().unwrap_or(UNUSED_CHECK) != UNUSED_CHECK {
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
    #[cfg(not(target_feature = "avx2"))]
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

    #[cfg(not(target_feature = "avx2"))]
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

    #[cfg(target_feature = "avx2")]
    #[inline(always)]
    pub fn accumulate_cost(&self, keys1: &[u32], keys2: &[u32]) -> i32 {
        assert_eq!(keys1.len() % SIMD_SIZE, 0);
        assert_eq!(keys2.len() % SIMD_SIZE, 0);
        assert_eq!(self.costs.len(), self.checks.len());
        unsafe {
            let bases_len = x86_64::_mm256_set1_epi32(i32::try_from(self.bases.len()).unwrap());
            let checks_len = x86_64::_mm256_set1_epi32(i32::try_from(self.checks.len()).unwrap());
            let unused_check = x86_64::_mm256_set1_epi32(UNUSED_CHECK as i32);
            let zeros = x86_64::_mm256_set1_epi32(0);
            let neg1 = x86_64::_mm256_set1_epi32(-1);
            let mut sums = x86_64::_mm256_set1_epi32(0);
            for (key1, key2) in keys1
                .chunks_exact(SIMD_SIZE)
                .zip(keys2.chunks_exact(SIMD_SIZE))
            {
                let key1 = x86_64::_mm256_loadu_si256(key1.as_ptr() as *const __m256i);
                let key2 = x86_64::_mm256_loadu_si256(key2.as_ptr() as *const __m256i);

                // 0 <= key1 < bases.len() ?
                let mask_valid_key1 = x86_64::_mm256_and_si256(
                    x86_64::_mm256_cmpgt_epi32(bases_len, key1),
                    x86_64::_mm256_cmpgt_epi32(key1, neg1),
                );
                // base = bases[key1]
                let base = x86_64::_mm256_mask_i32gather_epi32(
                    //unused_base,
                    zeros,
                    self.bases.as_ptr(),
                    key1,
                    mask_valid_key1,
                    4,
                );
                // pos = base + key2
                let pos = x86_64::_mm256_add_epi32(base, key2);
                // 0 <= pos < checks.len() && 0 <= key1 < bases.len() ?
                let mask_valid_pos = x86_64::_mm256_and_si256(
                    x86_64::_mm256_and_si256(
                        x86_64::_mm256_cmpgt_epi32(checks_len, pos),
                        x86_64::_mm256_cmpgt_epi32(pos, neg1),
                    ),
                    mask_valid_key1,
                );
                // check = checks[pos]
                let check = x86_64::_mm256_mask_i32gather_epi32(
                    unused_check,
                    self.checks.as_ptr() as *const i32,
                    pos,
                    mask_valid_pos,
                    4,
                );
                // check == key1 && 0 <= pos < checks.len() && 0 <= key1 < bases.len() ?
                let mask_checked = x86_64::_mm256_and_si256(
                    x86_64::_mm256_cmpeq_epi32(check, key1),
                    mask_valid_pos,
                );
                // returns costs[pos]
                let costs = x86_64::_mm256_mask_i32gather_epi32(
                    zeros,
                    self.costs.as_ptr(),
                    pos,
                    mask_checked,
                    4,
                );

                sums = x86_64::_mm256_add_epi32(sums, costs);
            }
            x86_64::_mm256_extract_epi32(sums, 0)
                + x86_64::_mm256_extract_epi32(sums, 1)
                + x86_64::_mm256_extract_epi32(sums, 2)
                + x86_64::_mm256_extract_epi32(sums, 3)
                + x86_64::_mm256_extract_epi32(sums, 4)
                + x86_64::_mm256_extract_epi32(sums, 5)
                + x86_64::_mm256_extract_epi32(sums, 6)
                + x86_64::_mm256_extract_epi32(sums, 7)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::dictionary::connector::raw_connector::INVALID_FEATURE_ID;

    #[cfg(not(target_feature = "avx2"))]
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
                &[
                    18,
                    17,
                    0,
                    INVALID_FEATURE_ID,
                    8,
                    12,
                    19,
                    INVALID_FEATURE_ID,
                    INVALID_FEATURE_ID,
                    9,
                    0,
                    7,
                    17,
                    13,
                    0,
                    INVALID_FEATURE_ID
                ],
                &[
                    17,
                    0,
                    0,
                    INVALID_FEATURE_ID,
                    6,
                    18,
                    5,
                    INVALID_FEATURE_ID,
                    INVALID_FEATURE_ID,
                    9,
                    19,
                    9,
                    4,
                    0,
                    18,
                    INVALID_FEATURE_ID
                ]
            ),
            100
        );
    }
}
