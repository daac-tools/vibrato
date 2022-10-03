mod builder;
mod scorer;

use bincode::{Decode, Encode};

use crate::dictionary::connector::scorer::Scorer;
use crate::dictionary::mapper::ConnIdMapper;

pub trait Connector {
    /// Returns maximum number of left connection ID
    fn num_left(&self) -> usize;

    /// Returns maximum number of right connection ID
    fn num_right(&self) -> usize;

    /// Do NOT make this function public to maintain consistency in
    /// the connection-id mapping among members of `Dictionary`.
    /// The consistency is managed in `Dictionary`.
    fn do_mapping(&mut self, mapper: &ConnIdMapper);
}

pub trait ConnectorCost: Connector {
    /// Gets the value of the connection matrix
    fn cost(&self, right_id: u16, left_id: u16) -> i32;

    /// Gets the value of the connection matrix
    unsafe fn cost_unchecked(&self, right_id: u16, left_id: u16) -> i32;
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

impl ConnectorCost for MatrixConnector {
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
}

#[derive(Decode, Encode)]
pub struct RawConnector {
    right_ids: Vec<u32>,
    left_ids: Vec<u32>,
    col_size: usize,
    scorer: Scorer,
}

impl RawConnector {
    pub fn new(right_ids: Vec<u32>, left_ids: Vec<u32>, col_size: usize, scorer: Scorer) -> Self {
        Self {
            right_ids,
            left_ids,
            col_size,
            scorer,
        }
    }

    #[inline(always)]
    fn right_feature_ids(&self, right_id: u16) -> &[u32] {
        &self.right_ids
            [usize::from(right_id) * self.col_size..usize::from(right_id + 1) * self.col_size]
    }

    #[inline(always)]
    fn left_feature_ids(&self, left_id: u16) -> &[u32] {
        &self.left_ids
            [usize::from(left_id) * self.col_size..usize::from(left_id + 1) * self.col_size]
    }
}

impl Connector for RawConnector {
    #[inline(always)]
    fn num_left(&self) -> usize {
        self.left_ids.len() / self.col_size
    }

    #[inline(always)]
    fn num_right(&self) -> usize {
        self.right_ids.len() / self.col_size
    }

    fn do_mapping(&mut self, _mapper: &ConnIdMapper) {
        unimplemented!()
    }
}

impl ConnectorCost for RawConnector {
    #[inline(always)]
    fn cost(&self, right_id: u16, left_id: u16) -> i32 {
        self.scorer.calculate_score(
            self.right_feature_ids(right_id),
            self.left_feature_ids(left_id),
        )
    }

    #[inline(always)]
    unsafe fn cost_unchecked(&self, right_id: u16, left_id: u16) -> i32 {
        self.cost(right_id, left_id)
    }
}

#[derive(Decode, Encode)]
pub enum ConnectorWrapper {
    Matrix(MatrixConnector),
    Raw(RawConnector),
}

impl Connector for ConnectorWrapper {
    #[inline(always)]
    fn num_left(&self) -> usize {
        match self {
            Self::Matrix(c) => c.num_left(),
            Self::Raw(c) => c.num_left(),
        }
    }

    #[inline(always)]
    fn num_right(&self) -> usize {
        match self {
            Self::Matrix(c) => c.num_right(),
            Self::Raw(c) => c.num_right(),
        }
    }

    #[inline(always)]
    fn do_mapping(&mut self, mapper: &ConnIdMapper) {
        match self {
            Self::Matrix(c) => c.do_mapping(mapper),
            Self::Raw(c) => c.do_mapping(mapper),
        }
    }
}
