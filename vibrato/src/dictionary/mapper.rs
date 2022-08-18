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
    lid_to_rid_count: Vec<Vec<usize>>,
}

impl ConnIdCounter {
    /// Creates a new counter for the matrix of `num_left \times num_right`.
    pub fn new(num_left: usize, num_right: usize) -> Self {
        Self {
            // The initial value 1 is for avoiding zero frequency.
            lid_to_rid_count: vec![vec![1; num_right]; num_left],
        }
    }

    #[inline(always)]
    pub fn add(&mut self, left_id: u16, right_id: u16, num: usize) {
        self.lid_to_rid_count[usize::from(left_id)][usize::from(right_id)] += num;
    }

    /// Computes the trained probabilities of connection ids.
    pub fn compute_probs(&self) -> (ConnIdProbs, ConnIdProbs) {
        let lid_to_rid_count = &self.lid_to_rid_count;

        let num_left = lid_to_rid_count.len();
        let num_right = lid_to_rid_count[0].len();

        // Compute Left-id probs
        let mut lid_probs = Vec::with_capacity(num_left);
        let mut lid_to_rid_probs = Vec::with_capacity(num_left);

        for (lid, rid_count) in lid_to_rid_count.iter().enumerate() {
            assert_eq!(num_right, rid_count.len());

            let acc = rid_count.iter().sum::<usize>() as f64;
            let mut probs = vec![0.0; num_right];
            if acc != 0.0 {
                for (rid, &cnt) in rid_count.iter().enumerate() {
                    probs[rid] = cnt as f64 / acc;
                }
            }
            lid_probs.push((lid, acc)); // ittan acc wo push suru
            lid_to_rid_probs.push(probs);
        }

        let acc = lid_probs.iter().fold(0., |acc, &(_, cnt)| acc + cnt);
        for (_, lp) in lid_probs.iter_mut() {
            *lp /= acc;
        }

        // Compute Right-id probs
        let mut rid_probs = vec![(0, 0.0); num_right];
        for (i, (rid, rp)) in rid_probs.iter_mut().enumerate() {
            *rid = i;
            for lid in 0..num_left {
                assert_eq!(lid, lid_probs[lid].0);
                *rp += lid_probs[lid].1 * lid_to_rid_probs[lid][*rid];
            }
        }

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
