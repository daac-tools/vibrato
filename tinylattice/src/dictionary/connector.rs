pub mod parser;

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
    pub fn num_left(&self) -> usize {
        self.num_left
    }

    /// Returns maximum number of right connection ID
    #[inline(always)]
    pub fn num_right(&self) -> usize {
        self.num_right
    }
}
