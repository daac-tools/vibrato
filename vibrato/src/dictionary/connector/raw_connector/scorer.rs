use std::collections::BTreeMap;

#[cfg(target_feature = "avx2")]
use std::arch::x86_64::{self, __m256i};

use bincode::{
    de::Decoder,
    enc::Encoder,
    error::{DecodeError, EncodeError},
    Decode, Encode,
};

use crate::num::U31;
use crate::utils::FromU32;

const UNUSED_CHECK: u32 = u32::MAX;

pub const SIMD_SIZE: usize = 8;
#[cfg(not(target_feature = "avx2"))]
#[derive(Clone, Copy)]
pub struct U31x8([U31; SIMD_SIZE]);
#[cfg(target_feature = "avx2")]
#[derive(Clone, Copy)]
pub struct U31x8(__m256i);

impl U31x8 {
    pub fn to_simd_vec(data: &[U31]) -> Vec<Self> {
        let mut result = vec![];
        for xs in data.chunks(SIMD_SIZE) {
            let mut array = [U31::default(); SIMD_SIZE];
            array[..xs.len()].copy_from_slice(xs);

            #[cfg(not(target_feature = "avx2"))]
            result.push(Self(array));

            // Safety
            debug_assert_eq!(std::mem::size_of_val(array.as_slice()), 32);
            #[cfg(target_feature = "avx2")]
            unsafe {
                result.push(Self(x86_64::_mm256_loadu_si256(
                    array.as_ptr() as *const __m256i
                )));
            }
        }
        result
    }
}

impl Default for U31x8 {
    #[cfg(not(target_feature = "avx2"))]
    fn default() -> Self {
        Self([U31::default(); SIMD_SIZE])
    }

    #[cfg(target_feature = "avx2")]
    fn default() -> Self {
        unsafe { Self(x86_64::_mm256_set1_epi32(0)) }
    }
}

impl Decode for U31x8 {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        let (a, b, c, d, e, f, g, h): (U31, U31, U31, U31, U31, U31, U31, U31) =
            Decode::decode(decoder)?;
        let data = [a, b, c, d, e, f, g, h];

        // Safety
        debug_assert_eq!(std::mem::size_of_val(data.as_slice()), 32);
        #[cfg(target_feature = "avx2")]
        let data = unsafe { x86_64::_mm256_loadu_si256(data.as_ptr() as *const __m256i) };

        Ok(Self(data))
    }
}
bincode::impl_borrow_decode!(U31x8);

impl Encode for U31x8 {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        #[cfg(not(target_feature = "avx2"))]
        let data = (
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5], self.0[6], self.0[7],
        );

        #[cfg(target_feature = "avx2")]
        let data = unsafe {
            (
                x86_64::_mm256_extract_epi32(self.0, 0),
                x86_64::_mm256_extract_epi32(self.0, 1),
                x86_64::_mm256_extract_epi32(self.0, 2),
                x86_64::_mm256_extract_epi32(self.0, 3),
                x86_64::_mm256_extract_epi32(self.0, 4),
                x86_64::_mm256_extract_epi32(self.0, 5),
                x86_64::_mm256_extract_epi32(self.0, 6),
                x86_64::_mm256_extract_epi32(self.0, 7),
            )
        };

        Encode::encode(&data, encoder)?;
        Ok(())
    }
}

pub struct ScorerBuilder {
    // Two-level trie mapping a pair of two keys into a cost, where
    // the first level stores the first key, and the second level stores the second key.
    pub trie: Vec<BTreeMap<U31, i32>>,
}

impl ScorerBuilder {
    pub const fn new() -> Self {
        Self { trie: vec![] }
    }

    pub fn insert(&mut self, key1: U31, key2: U31, cost: i32) {
        let key1 = usize::from_u32(key1.get());
        if key1 >= self.trie.len() {
            self.trie.resize(key1 + 1, BTreeMap::new());
        }
        self.trie[key1].insert(key2, cost);
    }

    #[inline(always)]
    fn check_base(base: u32, second_map: &BTreeMap<U31, i32>, checks: &[u32]) -> bool {
        for &key2 in second_map.keys() {
            if let Some(check) = checks.get(usize::from_u32(base ^ key2.get())) {
                if *check != UNUSED_CHECK {
                    return false;
                }
            }
        }
        true
    }

    pub fn build(&self) -> Scorer {
        let mut bases = vec![0; self.trie.len()];
        let mut checks = vec![];
        let mut costs = vec![];
        for (key1, second_map) in self.trie.iter().enumerate() {
            let mut base = 0;
            while !Self::check_base(base, second_map, &checks) {
                base += 1;
            }
            bases[key1] = base;
            for (key2, cost) in second_map {
                let pos = base ^ key2.get();
                let pos = usize::from_u32(pos);
                if pos >= checks.len() {
                    checks.resize(pos + 1, UNUSED_CHECK);
                    costs.resize(pos + 1, 0);
                }
                checks[pos] = u32::try_from(key1).unwrap();
                costs[pos] = *cost;
            }
        }

        #[cfg(target_feature = "avx2")]
        let bases_len = unsafe { x86_64::_mm256_set1_epi32(i32::try_from(bases.len()).unwrap()) };
        #[cfg(target_feature = "avx2")]
        let checks_len = unsafe { x86_64::_mm256_set1_epi32(i32::try_from(checks.len()).unwrap()) };
        Scorer {
            bases,
            checks,
            costs,

            #[cfg(target_feature = "avx2")]
            bases_len,
            #[cfg(target_feature = "avx2")]
            checks_len,
        }
    }
}

pub struct Scorer {
    bases: Vec<u32>,
    checks: Vec<u32>,
    costs: Vec<i32>,

    #[cfg(target_feature = "avx2")]
    bases_len: __m256i,
    #[cfg(target_feature = "avx2")]
    checks_len: __m256i,
}

#[allow(clippy::derivable_impls)]
impl Default for Scorer {
    fn default() -> Self {
        Self {
            bases: vec![],
            checks: vec![],
            costs: vec![],

            #[cfg(target_feature = "avx2")]
            bases_len: unsafe { x86_64::_mm256_set1_epi32(0) },
            #[cfg(target_feature = "avx2")]
            checks_len: unsafe { x86_64::_mm256_set1_epi32(0) },
        }
    }
}

impl Decode for Scorer {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        let bases: Vec<u32> = Decode::decode(decoder)?;
        let checks: Vec<u32> = Decode::decode(decoder)?;
        let costs: Vec<i32> = Decode::decode(decoder)?;

        if checks.len() != costs.len() {
            return Err(DecodeError::ArrayLengthMismatch {
                required: checks.len(),
                found: costs.len(),
            });
        }

        #[cfg(target_feature = "avx2")]
        let bases_len = unsafe { x86_64::_mm256_set1_epi32(i32::try_from(bases.len()).unwrap()) };
        #[cfg(target_feature = "avx2")]
        let checks_len = unsafe { x86_64::_mm256_set1_epi32(i32::try_from(checks.len()).unwrap()) };

        Ok(Self {
            bases,
            checks,
            costs,

            #[cfg(target_feature = "avx2")]
            bases_len,
            #[cfg(target_feature = "avx2")]
            checks_len,
        })
    }
}
bincode::impl_borrow_decode!(Scorer);

impl Encode for Scorer {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        Encode::encode(&self.bases, encoder)?;
        Encode::encode(&self.checks, encoder)?;
        Encode::encode(&self.costs, encoder)?;
        Ok(())
    }
}

impl Scorer {
    #[cfg(not(target_feature = "avx2"))]
    #[inline(always)]
    fn retrieve_cost(&self, key1: U31, key2: U31) -> Option<i32> {
        if let Some(base) = self.bases.get(usize::from_u32(key1.get())) {
            let pos = base ^ key2.get();
            let pos = usize::from_u32(pos);
            if let Some(check) = self.checks.get(pos) {
                if *check == key1.get() {
                    return Some(self.costs[pos]);
                }
            }
        }
        None
    }

    #[cfg(not(target_feature = "avx2"))]
    #[inline(always)]
    pub fn accumulate_cost(&self, keys1: &[U31x8], keys2: &[U31x8]) -> i32 {
        let mut score = 0;
        for (key1, key2) in keys1.iter().zip(keys2) {
            for (&key1, &key2) in key1.0.iter().zip(&key2.0) {
                if let Some(w) = self.retrieve_cost(key1, key2) {
                    score += w;
                }
            }
        }
        score
    }

    /// # Safety
    ///
    /// Arguments must satisfy the following constraints:
    ///
    /// * 0 <= key1
    /// * 0 <= key2
    /// * self.costs.len() == self.checks.len()
    #[cfg(target_feature = "avx2")]
    #[inline(always)]
    pub unsafe fn retrieve_cost(&self, key1: __m256i, key2: __m256i) -> __m256i {
        // key1 < bases.len() ?
        let mask_valid_key1 = x86_64::_mm256_cmpgt_epi32(self.bases_len, key1);
        // base = bases[key1]
        let base = x86_64::_mm256_mask_i32gather_epi32(
            x86_64::_mm256_set1_epi32(0),
            self.bases.as_ptr() as *const i32,
            key1,
            mask_valid_key1,
            4,
        );
        // pos = base ^ key2
        // (base >= 0 && key2 >= 0 ==> pos >= 0)
        let pos = x86_64::_mm256_xor_si256(base, key2);
        // pos < checks.len() && key1 < bases.len() ?
        let mask_valid_pos = x86_64::_mm256_and_si256(
            x86_64::_mm256_cmpgt_epi32(self.checks_len, pos),
            mask_valid_key1,
        );
        // check = checks[pos]
        let check = x86_64::_mm256_mask_i32gather_epi32(
            x86_64::_mm256_set1_epi32(UNUSED_CHECK as i32),
            self.checks.as_ptr() as *const i32,
            pos,
            mask_valid_pos,
            4,
        );
        // check == key1 && pos < checks.len() && key1 < bases.len() ?
        let mask_checked =
            x86_64::_mm256_and_si256(x86_64::_mm256_cmpeq_epi32(check, key1), mask_valid_pos);

        x86_64::_mm256_mask_i32gather_epi32(
            x86_64::_mm256_set1_epi32(0),
            self.costs.as_ptr(),
            pos,
            mask_checked,
            4,
        )
    }

    /// # Safety
    ///
    /// Arguments must satisfy the following constraints:
    ///
    /// * 0 <= key1
    /// * 0 <= key2
    /// * self.costs.len() == self.checks.len()
    #[cfg(target_feature = "avx2")]
    #[inline(always)]
    pub fn accumulate_cost(&self, keys1: &[U31x8], keys2: &[U31x8]) -> i32 {
        unsafe {
            let mut sums = x86_64::_mm256_set1_epi32(0);
            for (key1, key2) in keys1.iter().zip(keys2) {
                let key1 = key1.0;
                let key2 = key2.0;

                sums = x86_64::_mm256_add_epi32(sums, self.retrieve_cost(key1, key2));
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
        builder.insert(U31::new(18).unwrap(), U31::new(17).unwrap(), 1);
        builder.insert(U31::new(4).unwrap(), U31::new(9).unwrap(), 2);
        builder.insert(U31::new(17).unwrap(), U31::new(0).unwrap(), 3);
        builder.insert(U31::new(17).unwrap(), U31::new(12).unwrap(), 4);
        builder.insert(U31::new(8).unwrap(), U31::new(6).unwrap(), 5);
        builder.insert(U31::new(2).unwrap(), U31::new(5).unwrap(), 6);
        builder.insert(U31::new(12).unwrap(), U31::new(18).unwrap(), 7);
        builder.insert(U31::new(9).unwrap(), U31::new(1).unwrap(), 8);
        builder.insert(U31::new(19).unwrap(), U31::new(5).unwrap(), 9);
        builder.insert(U31::new(9).unwrap(), U31::new(4).unwrap(), 10);
        builder.insert(U31::new(0).unwrap(), U31::new(19).unwrap(), 11);
        builder.insert(U31::new(2).unwrap(), U31::new(19).unwrap(), 12);
        builder.insert(U31::new(7).unwrap(), U31::new(9).unwrap(), 13);
        builder.insert(U31::new(18).unwrap(), U31::new(9).unwrap(), 14);
        builder.insert(U31::new(17).unwrap(), U31::new(4).unwrap(), 15);
        builder.insert(U31::new(9).unwrap(), U31::new(6).unwrap(), 16);
        builder.insert(U31::new(13).unwrap(), U31::new(0).unwrap(), 17);
        builder.insert(U31::new(1).unwrap(), U31::new(4).unwrap(), 18);
        builder.insert(U31::new(0).unwrap(), U31::new(18).unwrap(), 19);
        builder.insert(U31::new(18).unwrap(), U31::new(11).unwrap(), 20);
        let scorer = builder.build();

        assert_eq!(
            scorer.retrieve_cost(U31::new(0).unwrap(), U31::new(18).unwrap()),
            Some(19)
        );
        assert_eq!(
            scorer.retrieve_cost(U31::new(0).unwrap(), U31::new(19).unwrap()),
            Some(11)
        );
        assert_eq!(
            scorer.retrieve_cost(U31::new(9).unwrap(), U31::new(4).unwrap()),
            Some(10)
        );
        assert_eq!(
            scorer.retrieve_cost(U31::new(9).unwrap(), U31::new(6).unwrap()),
            Some(16)
        );
        assert_eq!(
            scorer.retrieve_cost(U31::new(0).unwrap(), U31::new(0).unwrap()),
            None
        );
        assert_eq!(
            scorer.retrieve_cost(U31::new(9).unwrap(), U31::new(5).unwrap()),
            None
        );
    }

    #[test]
    fn accumulate_cost_test() {
        let mut builder = ScorerBuilder::new();
        builder.insert(U31::new(18).unwrap(), U31::new(17).unwrap(), 1);
        builder.insert(U31::new(4).unwrap(), U31::new(9).unwrap(), 2);
        builder.insert(U31::new(17).unwrap(), U31::new(0).unwrap(), 3);
        builder.insert(U31::new(17).unwrap(), U31::new(12).unwrap(), 4);
        builder.insert(U31::new(8).unwrap(), U31::new(6).unwrap(), 5);
        builder.insert(U31::new(2).unwrap(), U31::new(5).unwrap(), 6);
        builder.insert(U31::new(12).unwrap(), U31::new(18).unwrap(), 7);
        builder.insert(U31::new(9).unwrap(), U31::new(1).unwrap(), 8);
        builder.insert(U31::new(19).unwrap(), U31::new(5).unwrap(), 9);
        builder.insert(U31::new(9).unwrap(), U31::new(4).unwrap(), 10);
        builder.insert(U31::new(0).unwrap(), U31::new(19).unwrap(), 11);
        builder.insert(U31::new(2).unwrap(), U31::new(19).unwrap(), 12);
        builder.insert(U31::new(7).unwrap(), U31::new(9).unwrap(), 13);
        builder.insert(U31::new(18).unwrap(), U31::new(9).unwrap(), 14);
        builder.insert(U31::new(17).unwrap(), U31::new(4).unwrap(), 15);
        builder.insert(U31::new(9).unwrap(), U31::new(6).unwrap(), 16);
        builder.insert(U31::new(13).unwrap(), U31::new(0).unwrap(), 17);
        builder.insert(U31::new(1).unwrap(), U31::new(4).unwrap(), 18);
        builder.insert(U31::new(0).unwrap(), U31::new(18).unwrap(), 19);
        builder.insert(U31::new(18).unwrap(), U31::new(11).unwrap(), 20);
        let scorer = builder.build();

        assert_eq!(
            scorer.accumulate_cost(
                &U31x8::to_simd_vec(&[
                    U31::new(18).unwrap(),
                    U31::new(17).unwrap(),
                    U31::new(0).unwrap(),
                    INVALID_FEATURE_ID,
                    U31::new(8).unwrap(),
                    U31::new(12).unwrap(),
                    U31::new(19).unwrap(),
                    INVALID_FEATURE_ID,
                    INVALID_FEATURE_ID,
                    U31::new(9).unwrap(),
                    U31::new(0).unwrap(),
                    U31::new(7).unwrap(),
                    U31::new(17).unwrap(),
                    U31::new(13).unwrap(),
                    U31::new(0).unwrap(),
                    INVALID_FEATURE_ID
                ]),
                &U31x8::to_simd_vec(&[
                    U31::new(17).unwrap(),
                    U31::new(0).unwrap(),
                    U31::new(0).unwrap(),
                    INVALID_FEATURE_ID,
                    U31::new(6).unwrap(),
                    U31::new(18).unwrap(),
                    U31::new(5).unwrap(),
                    INVALID_FEATURE_ID,
                    INVALID_FEATURE_ID,
                    U31::new(9).unwrap(),
                    U31::new(19).unwrap(),
                    U31::new(9).unwrap(),
                    U31::new(4).unwrap(),
                    U31::new(0).unwrap(),
                    U31::new(18).unwrap(),
                    INVALID_FEATURE_ID
                ]),
            ),
            100,
        );
    }

    #[test]
    fn accumulate_cost_empty_test() {
        let builder = ScorerBuilder::new();
        let scorer = builder.build();

        assert_eq!(scorer.accumulate_cost(&[], &[]), 0);
    }
}
