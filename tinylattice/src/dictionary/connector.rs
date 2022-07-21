pub mod builder;

use super::mapper::ConnIdMapper;

pub struct Connector {
    data: Vec<i16>,
    num_right: usize,
    num_left: usize,
}

impl Connector {
    pub fn new(data: Vec<i16>, num_right: usize, num_left: usize) -> Self {
        Self {
            data,
            num_right,
            num_left,
        }
    }

    #[inline(always)]
    fn index(&self, right_id: usize, left_id: usize) -> usize {
        debug_assert!(right_id < self.num_right);
        debug_assert!(left_id < self.num_left);
        let index = left_id * self.num_right + right_id;
        debug_assert!(index < self.data.len());
        index
    }

    /// Gets the value of the connection matrix
    #[inline(always)]
    pub fn cost(&self, right_id: usize, left_id: usize) -> i16 {
        let index = self.index(right_id, left_id);
        *unsafe { self.data.get_unchecked(index) }
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

    pub fn map_ids(&mut self, mapper: &ConnIdMapper) {
        assert_eq!(mapper.num_left(), self.num_left);
        assert_eq!(mapper.num_right(), self.num_right);

        let mut mapped = vec![0; self.data.len()];
        for right_id in 0..self.num_right {
            let new_right_id = mapper.right(right_id as u16) as usize;
            for left_id in 0..self.num_left {
                let new_left_id = mapper.left(left_id as u16) as usize;
                let index = self.index(right_id, left_id);
                let new_index = self.index(new_right_id, new_left_id);
                mapped[new_index] = self.data[index];
            }
        }
        self.data = mapped;
    }
}
