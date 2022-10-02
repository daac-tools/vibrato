mod builder;
mod scorer;

use bincode::{Decode, Encode};

use crate::dictionary::mapper::ConnIdMapper;

use self::scorer::Scorer;

/// Matrix of connection costs.
#[derive(Decode, Encode)]
pub struct Connector {
    data: Vec<i16>,
    num_right: usize,
    num_left: usize,

    scorer: Scorer,
    right_ids: Vec<Vec<usize>>,
    left_ids: Vec<Vec<usize>>,
    zeros: Vec<usize>,
}

impl Connector {
    pub fn new(data: Vec<i16>, num_right: usize, num_left: usize) -> Self {
        Self {
            data,
            num_right,
            num_left,

            scorer: Scorer::default(),
            right_ids: vec![],
            left_ids: vec![],
            zeros: vec![],
        }
    }
    pub fn new_detailed(
        scorer: Scorer,
        right_ids: Vec<Vec<usize>>,
        left_ids: Vec<Vec<usize>>,
        zeros: Vec<usize>,
    ) -> Self {
        Self {
            data: vec![],
            num_right: 0,
            num_left: 0,

            scorer,
            right_ids,
            left_ids,
            zeros,
        }
    }

    #[inline(always)]
    fn index(&self, right_id: u16, left_id: u16) -> usize {
        debug_assert!(usize::from(right_id) < self.num_right);
        debug_assert!(usize::from(left_id) < self.num_left);
        let index = usize::from(left_id) * self.num_right + usize::from(right_id);
        debug_assert!(index < self.data.len());
        index
    }

    /// Gets the value of the connection matrix
    #[inline(always)]
    pub fn cost(&self, right_id: u16, left_id: u16) -> i32 {
        /*
        let index = self.index(right_id, left_id);
        self.data[index]
        */
        if right_id == 0 {
            self.scorer
                .calculate_score_simd(&self.zeros, &self.left_ids[usize::from(left_id - 1)])
        } else if left_id == 0 {
            self.scorer
                .calculate_score_simd(&self.right_ids[usize::from(right_id - 1)], &self.zeros)
        } else {
            self.scorer.calculate_score_simd(
                &self.right_ids[usize::from(right_id - 1)],
                &[usize::from(left_id - 1)],
            )
        }
    }

    /// Gets the value of the connection matrix
    #[inline(always)]
    pub unsafe fn cost_unchecked(&self, right_id: u16, left_id: u16) -> i16 {
        let index = self.index(right_id, left_id);
        // The tokenization time can be shortened by 5--10%.
        *self.data.get_unchecked(index)
    }

    /// Returns maximum number of left connection ID
    #[inline(always)]
    pub const fn num_left(&self) -> usize {
        self.num_left
    }

    /// Returns maximum number of right connection ID
    #[inline(always)]
    pub const fn num_right(&self) -> usize {
        self.num_right
    }

    /// Do NOT make this function public to maintain consistency in
    /// the connection-id mapping among members of `Dictionary`.
    /// The consistency is managed in `Dictionary`.
    pub fn do_mapping(&mut self, mapper: &ConnIdMapper) {
        assert_eq!(mapper.num_left(), self.num_left);
        assert_eq!(mapper.num_right(), self.num_right);

        let mut mapped = vec![0; self.data.len()];
        for right_id in 0..self.num_right {
            let right_id = right_id as u16;
            let new_right_id = mapper.right(right_id);
            for left_id in 0..self.num_left {
                let left_id = left_id as u16;
                let new_left_id = mapper.left(left_id);
                let index = self.index(right_id, left_id);
                let new_index = self.index(new_right_id, new_left_id);
                mapped[new_index] = self.data[index];
            }
        }
        self.data = mapped;
    }
}
