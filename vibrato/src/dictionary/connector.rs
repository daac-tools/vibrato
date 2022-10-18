mod matrix_connector;
mod raw_connector;

use bincode::{Decode, Encode};

pub use crate::dictionary::connector::matrix_connector::MatrixConnector;
pub use crate::dictionary::connector::raw_connector::RawConnector;
use crate::dictionary::mapper::ConnIdMapper;

pub trait Connector {
    /// Returns maximum number of left connection ID
    fn num_left(&self) -> usize;

    /// Returns maximum number of right connection ID
    fn num_right(&self) -> usize;

    /// Do NOT make this function public to maintain consistency in
    /// the connection-id mapping among members of `Dictionary`.
    /// The consistency is managed in `Dictionary`.
    fn map_connection_ids(&mut self, mapper: &ConnIdMapper);
}

pub trait ConnectorCost: Connector {
    /// Gets the value of the connection matrix
    fn cost(&self, right_id: u16, left_id: u16) -> i32;

    /// Gets the value of the connection matrix
    unsafe fn cost_unchecked(&self, right_id: u16, left_id: u16) -> i32;
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
    fn map_connection_ids(&mut self, mapper: &ConnIdMapper) {
        match self {
            Self::Matrix(c) => c.map_connection_ids(mapper),
            Self::Raw(c) => c.map_connection_ids(mapper),
        }
    }
}
