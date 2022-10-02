mod builder;

use bincode::{Decode, Encode};

use crate::dictionary::mapper::ConnIdMapper;

/// Matrix of connection costs.
#[derive(Decode, Encode)]
pub struct Connector {
    data: Vec<i16>,
    num_right: usize,
    num_left: usize,

    map: std::collections::HashMap<(u32, u32), i32>,
    right_ids: Vec<Vec<u32>>,
    left_ids: Vec<Vec<u32>>,
}

impl Connector {
    pub fn new(data: Vec<i16>, num_right: usize, num_left: usize) -> Self {
        Self {
            data,
            num_right,
            num_left,

            map: std::collections::HashMap::new(),
            right_ids: vec![],
            left_ids: vec![],
        }
    }
    pub fn new_detailed(map: std::collections::HashMap<(u32, u32), i32>, right_ids: Vec<Vec<u32>>, left_ids: Vec<Vec<u32>>) -> Self {
        Self {
            data: vec![],
            num_right: 0,
            num_left: 0,

            map,
            right_ids,
            left_ids,
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
        let mut weight = 0;
        if right_id == 0 {
            for &left_id in &self.left_ids[usize::from(left_id - 1)] {
                if let Some(w) = self.map.get(&(0, left_id)) {
                    weight += w;
                }
            }
        } else if left_id == 0 {
            for &right_id in &self.right_ids[usize::from(right_id - 1)] {
                if let Some(w) = self.map.get(&(right_id, 0)) {
                    weight += w;
                }
            }
        } else {
            for (&right_id, &left_id) in self.right_ids[usize::from(right_id - 1)].iter().zip(&self.left_ids[usize::from(left_id - 1)]) {
                if let Some(w) = self.map.get(&(right_id, left_id)) {
                    weight += w;
                }
            }
        }
        weight
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
