pub mod parser;

pub struct CostMatrix {
    data: Vec<i16>,
    num_left: usize,
    num_right: usize,
}

impl CostMatrix {
    pub fn new(data: Vec<i16>, num_left: usize, num_right: usize) -> Self {
        Self {
            data,
            num_left,
            num_right,
        }
    }

    #[inline(always)]
    fn index(&self, left: usize, right: usize) -> usize {
        debug_assert!(left < self.num_left);
        debug_assert!(right < self.num_right);
        let index = right * self.num_left + left;
        debug_assert!(index < self.data.len());
        index
    }

    /// Gets the value of the connection matrix
    #[inline(always)]
    pub fn cost(&self, left: usize, right: usize) -> i16 {
        let index = self.index(left, right);
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
