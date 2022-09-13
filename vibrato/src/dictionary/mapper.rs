mod builder;

use bincode::{Decode, Encode};

/// Mapper for connection ids.
#[derive(Decode, Encode)]
pub struct ConnIdMapper {
    left: Vec<u16>,
    right: Vec<u16>,
}

impl ConnIdMapper {
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
}

/// Trained occurrence probabilities of connection ids.
pub type ConnIdProbs = Vec<(usize, f64)>;

/// Counter to train mappings of connection ids.
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

    /// Computes the trained probabilities of connection ids.
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
}
