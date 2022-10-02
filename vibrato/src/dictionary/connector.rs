mod builder;

use bincode::{Decode, Encode};

use crate::dictionary::mapper::ConnIdMapper;

pub trait Connector {
    /// Gets the value of the connection matrix
    fn cost(&self, right_id: u16, left_id: u16) -> i32;

    /// Gets the value of the connection matrix
    unsafe fn cost_unchecked(&self, right_id: u16, left_id: u16) -> i32;

    /// Returns maximum number of left connection ID
    fn num_left(&self) -> usize;

    /// Returns maximum number of right connection ID
    fn num_right(&self) -> usize;

    /// Do NOT make this function public to maintain consistency in
    /// the connection-id mapping among members of `Dictionary`.
    /// The consistency is managed in `Dictionary`.
    fn do_mapping(&mut self, mapper: &ConnIdMapper);
}

/// Matrix of connection costs.
#[derive(Decode, Encode)]
pub struct MatrixConnector {
    data: Vec<i16>,
    num_right: usize,
    num_left: usize,
}

impl MatrixConnector {
    pub fn new(data: Vec<i16>, num_right: usize, num_left: usize) -> Self {
        Self {
            data,
            num_right,
            num_left,
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
}

impl Connector for MatrixConnector {
    #[inline(always)]
    fn cost(&self, right_id: u16, left_id: u16) -> i32 {
        let index = self.index(right_id, left_id);
        i32::from(self.data[index])
    }

    #[inline(always)]
    unsafe fn cost_unchecked(&self, right_id: u16, left_id: u16) -> i32 {
        let index = self.index(right_id, left_id);
        // The tokenization time can be shortened by 5--10%.
        i32::from(*self.data.get_unchecked(index))
    }

    #[inline(always)]
    fn num_left(&self) -> usize {
        self.num_left
    }

    #[inline(always)]
    fn num_right(&self) -> usize {
        self.num_right
    }

    fn do_mapping(&mut self, mapper: &ConnIdMapper) {
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
