pub mod parser;

use crate::serializer::Serializable;

pub struct ConnectionMatrix {
    data: Vec<i16>,
    num_left: usize,
    num_right: usize,
}

impl ConnectionMatrix {
    pub fn new(data: Vec<i16>, num_left: usize, num_right: usize) -> Self {
        Self {
            data,
            num_left,
            num_right,
        }
    }

    #[inline(always)]
    fn index(&self, left: u16, right: u16) -> usize {
        let uleft = left as usize;
        let uright = right as usize;
        debug_assert!(uleft < self.num_left);
        debug_assert!(uright < self.num_right);
        let index = uright * self.num_left + uleft;
        debug_assert!(index < self.data.len());
        index
    }

    /// Gets the value of the connection matrix
    ///
    /// It is performance critical that this function
    /// 1. Has no branches
    /// 2. Is inlined to the caller
    ///
    /// This is UB if index is out of bounds, but that can't happen
    /// except in the case if the binary dictionary was tampered with.
    /// It is OK to make usage of tampered binary dictionaries UB.
    #[inline(always)]
    pub fn cost(&self, left: u16, right: u16) -> i16 {
        let index = self.index(left, right);
        *unsafe { self.data.get_unchecked(index) }
    }

    /// Returns maximum number of left connection ID
    pub fn num_left(&self) -> usize {
        self.num_left
    }

    /// Returns maximum number of right connection ID
    pub fn num_right(&self) -> usize {
        self.num_right
    }
}
