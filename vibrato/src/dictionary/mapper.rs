use bincode::{Decode, Encode};

use crate::errors::{Result, VibratoError};

use crate::common::BOS_EOS_CONNECTION_ID;

/// Mapper for connection ids.
#[derive(Decode, Encode)]
pub struct ConnIdMapper {
    left: Vec<u16>,
    right: Vec<u16>,
}

impl ConnIdMapper {
    pub const fn new(left: Vec<u16>, right: Vec<u16>) -> Self {
        Self { left, right }
    }

    #[inline(always)]
    pub fn num_left(&self) -> usize {
        self.left.len()
    }

    #[inline(always)]
    pub fn num_right(&self) -> usize {
        self.right.len()
    }

    #[inline(always)]
    pub fn left(&self, id: u16) -> u16 {
        self.left[usize::from(id)]
    }

    #[inline(always)]
    pub fn right(&self, id: u16) -> u16 {
        self.right[usize::from(id)]
    }

    pub fn from_iter<L, R>(lmap: L, rmap: R) -> Result<Self>
    where
        L: IntoIterator<Item = u16>,
        R: IntoIterator<Item = u16>,
    {
        let left = Self::parse(lmap)?;
        let right = Self::parse(rmap)?;
        Ok(Self::new(left, right))
    }

    fn parse<I>(map: I) -> Result<Vec<u16>>
    where
        I: IntoIterator<Item = u16>,
    {
        let mut old_ids = vec![BOS_EOS_CONNECTION_ID];
        for old_id in map {
            if old_id == BOS_EOS_CONNECTION_ID {
                let msg = format!("Id {BOS_EOS_CONNECTION_ID} is reserved.");
                return Err(VibratoError::invalid_argument("map", msg));
            }
            old_ids.push(old_id);
        }

        let mut new_ids = vec![u16::MAX; old_ids.len()];
        new_ids[usize::from(BOS_EOS_CONNECTION_ID)] = BOS_EOS_CONNECTION_ID;

        for (new_id, &old_id) in old_ids.iter().enumerate().skip(1) {
            debug_assert_ne!(old_id, BOS_EOS_CONNECTION_ID);
            if let Some(e) = new_ids.get_mut(usize::from(old_id)) {
                if *e != u16::MAX {
                    return Err(VibratoError::invalid_argument("map", "ids are duplicate."));
                }
                *e = u16::try_from(new_id)?;
            } else {
                return Err(VibratoError::invalid_argument(
                    "map",
                    "ids are out of range.",
                ));
            }
        }
        Ok(new_ids)
    }
}

/// Trained occurrence probabilities of connection ids.
pub type ConnIdProbs = Vec<(usize, f64)>;

/// Counter to reorder mappings of connection ids.
pub struct ConnIdCounter {
    lid_count: Vec<usize>,
    rid_count: Vec<usize>,
}

impl ConnIdCounter {
    /// Creates a new counter for the matrix of `num_left \times num_right`.
    pub fn new(num_left: usize, num_right: usize) -> Self {
        Self {
            lid_count: vec![0; num_left],
            rid_count: vec![0; num_right],
        }
    }

    #[inline(always)]
    pub fn add(&mut self, left_id: u16, right_id: u16, num: usize) {
        self.lid_count[usize::from(left_id)] += num;
        self.rid_count[usize::from(right_id)] += num;
    }

    /// Computes the probabilities of connection ids.
    pub fn compute_probs(&self) -> (ConnIdProbs, ConnIdProbs) {
        let lid_count = &self.lid_count;
        let rid_count = &self.rid_count;

        // Compute Left-id probs
        let lid_sum = lid_count.iter().sum::<usize>() as f64;
        let mut lid_probs: Vec<_> = lid_count
            .iter()
            .enumerate()
            .map(|(lid, &cnt)| (lid, cnt as f64 / lid_sum))
            .collect();

        // Compute Right-id probs
        let rid_sum = rid_count.iter().sum::<usize>() as f64;
        let mut rid_probs: Vec<_> = rid_count
            .iter()
            .enumerate()
            .map(|(rid, &cnt)| (rid, cnt as f64 / rid_sum))
            .collect();

        // Pop Id = 0
        assert_eq!(crate::common::BOS_EOS_CONNECTION_ID, 0);
        lid_probs.drain(..1);
        rid_probs.drain(..1);

        // Sort
        lid_probs.sort_unstable_by(|(i1, p1), (i2, p2)| {
            p2.partial_cmp(p1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| i1.cmp(i2))
        });
        rid_probs.sort_unstable_by(|(i1, p1), (i2, p2)| {
            p2.partial_cmp(p1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| i1.cmp(i2))
        });

        (lid_probs, rid_probs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_probs() {
        let mut counter = ConnIdCounter::new(3, 3);
        counter.add(0, 2, 1);
        counter.add(1, 0, 3);
        counter.add(2, 2, 4);
        counter.add(1, 2, 2);

        let (lprobs, rprobs) = counter.compute_probs();
        assert_eq!(lprobs, vec![(1, 5f64 / 10f64), (2, 4f64 / 10f64)]);
        assert_eq!(rprobs, vec![(2, 7f64 / 10f64), (1, 0f64 / 10f64)]);
    }

    #[test]
    fn test_parse_basic() {
        let map = vec![2, 3, 4, 1];
        let mapping = ConnIdMapper::parse(map.into_iter()).unwrap();
        assert_eq!(mapping, vec![0, 4, 1, 2, 3]);
    }

    #[test]
    fn test_parse_zero() {
        let map = vec![2, 3, 0, 1];
        let result = ConnIdMapper::parse(map.into_iter());
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_oor() {
        let map = vec![2, 3, 5, 1];
        let result = ConnIdMapper::parse(map.into_iter());
        assert!(result.is_err());
    }
}
