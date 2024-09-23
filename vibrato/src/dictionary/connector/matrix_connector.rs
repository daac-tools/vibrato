use std::io::{prelude::*, BufReader, Read};

use bincode::{Decode, Encode};

use crate::dictionary::connector::{Connector, ConnectorCost};
use crate::dictionary::mapper::ConnIdMapper;
use crate::errors::{Result, VibratoError};

/// Matrix of connection costs.
#[derive(Decode, Encode)]
pub struct MatrixConnector {
    data: Vec<i16>,
    num_right: usize,
    num_left: usize,
}

impl MatrixConnector {
    pub const fn new(data: Vec<i16>, num_right: usize, num_left: usize) -> Self {
        Self {
            data,
            num_right,
            num_left,
        }
    }

    /// Creates a new instance from `matrix.def`.
    pub fn from_reader<R>(rdr: R) -> Result<Self>
    where
        R: Read,
    {
        let reader = BufReader::new(rdr);
        let mut lines = reader.lines();

        let (num_right, num_left) = Self::parse_header(&lines.next().unwrap()?)?;
        let mut data = vec![0; num_right * num_left];

        for line in lines {
            let line = line?;
            if !line.is_empty() {
                let (right_id, left_id, conn_cost) = Self::parse_body(&line)?;
                if num_right <= right_id || num_left <= left_id {
                    return Err(VibratoError::invalid_format(
                        "matrix.def",
                        "left/right_id must be within num_left/right.",
                    ));
                }
                data[left_id * num_right + right_id] = conn_cost;
            }
        }
        Ok(Self::new(data, num_right, num_left))
    }

    fn parse_header(line: &str) -> Result<(usize, usize)> {
        let cols: Vec<_> = line.split(' ').collect();
        if cols.len() != 2 {
            let msg =
                format!("The header must consists of two integers separated by spaces, {line}");
            Err(VibratoError::invalid_format("matrix.def", msg))
        } else {
            let num_right: u16 = cols[0].parse()?;
            let num_left: u16 = cols[1].parse()?;
            Ok((usize::from(num_right), usize::from(num_left)))
        }
    }

    fn parse_body(line: &str) -> Result<(usize, usize, i16)> {
        let cols: Vec<_> = line.split(' ').collect();
        if cols.len() != 3 {
            let msg = format!(
                "A row other than the header must consists of three integers separated by spaces, {line}"
            );
            Err(VibratoError::invalid_format("matrix.def", msg))
        } else {
            Ok((cols[0].parse()?, cols[1].parse()?, cols[2].parse()?))
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

    fn map_connection_ids(&mut self, mapper: &ConnIdMapper) {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_2x2() {
        let data = "2 2
0 0 0
0 1 1
1 0 -2
1 1 -3";
        let conn = MatrixConnector::from_reader(data.as_bytes()).unwrap();
        assert_eq!(conn.cost(0, 0), 0);
        assert_eq!(conn.cost(0, 1), 1);
        assert_eq!(conn.cost(1, 0), -2);
        assert_eq!(conn.cost(1, 1), -3);
    }

    #[test]
    fn test_2x3() {
        let data = "2 3
0 0 0
0 1 1
0 2 2
1 0 -3
1 1 -4
1 2 -5";
        let conn = MatrixConnector::from_reader(data.as_bytes()).unwrap();
        assert_eq!(conn.cost(0, 0), 0);
        assert_eq!(conn.cost(0, 1), 1);
        assert_eq!(conn.cost(0, 2), 2);
        assert_eq!(conn.cost(1, 0), -3);
        assert_eq!(conn.cost(1, 1), -4);
        assert_eq!(conn.cost(1, 2), -5);
    }

    #[test]
    fn test_mapping() {
        let data = "2 3
0 0 0
0 1 1
0 2 2
1 0 -3
1 1 -4
1 2 -5";
        let mut conn = MatrixConnector::from_reader(data.as_bytes()).unwrap();

        let mapper = ConnIdMapper::new(vec![2, 0, 1], vec![1, 0]);
        conn.map_connection_ids(&mapper);

        assert_eq!(conn.cost(0, 0), -4);
        assert_eq!(conn.cost(0, 1), -5);
        assert_eq!(conn.cost(0, 2), -3);
        assert_eq!(conn.cost(1, 0), 1);
        assert_eq!(conn.cost(1, 1), 2);
        assert_eq!(conn.cost(1, 2), 0);
    }

    #[test]
    fn test_less_header() {
        let data = "2
0 0 0
0 1 1
1 0 -2
1 1 -3";
        let result = MatrixConnector::from_reader(data.as_bytes());

        assert!(result.is_err());
    }

    #[test]
    fn test_more_header() {
        let data = "2 2 2
0 0 0
0 1 1
1 0 -2
1 1 -3";
        let result = MatrixConnector::from_reader(data.as_bytes());

        assert!(result.is_err());
    }

    #[test]
    fn test_less_body() {
        let data = "2 2
0 0 0
0 1 1
1 -2
1 1 -3";
        let result = MatrixConnector::from_reader(data.as_bytes());

        assert!(result.is_err());
    }

    #[test]
    fn test_more_body() {
        let data = "2 2
0 0 0
0 1 1
1 0 1 -2
1 1 -3";
        let result = MatrixConnector::from_reader(data.as_bytes());

        assert!(result.is_err());
    }

    #[test]
    fn test_larger_matrix() {
        let data = "65536 65536";
        let result = MatrixConnector::from_reader(data.as_bytes());

        assert!(result.is_err());
    }

    #[test]
    fn test_larger_left_id() {
        let data = "2 2
0 0 0
0 1 1
1 2 -2
1 1 -3";
        let result = MatrixConnector::from_reader(data.as_bytes());

        assert!(result.is_err());
    }

    #[test]
    fn test_larger_right_id() {
        let data = "2 2
0 0 0
0 1 1
2 0 -2
1 1 -3";
        let result = MatrixConnector::from_reader(data.as_bytes());

        assert!(result.is_err());
    }
}
