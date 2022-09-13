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

        let num_left = lid_count.len();
        let num_right = rid_count.len();

        // Compute Left-id probs
        let mut lid_probs = Vec::with_capacity(num_left);
        {
            let acc = lid_count.iter().sum::<usize>() as f64;
            for (lid, &cnt) in lid_count.iter().enumerate() {
                let cnt = cnt as f64;
                lid_probs.push((lid, cnt / acc));
            }
        }

        // Compute Right-id probs
        let mut rid_probs = Vec::with_capacity(num_right);
        {
            let acc = rid_count.iter().sum::<usize>() as f64;
            for (rid, &cnt) in rid_count.iter().enumerate() {
                let cnt = cnt as f64;
                rid_probs.push((rid, cnt / acc));
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
