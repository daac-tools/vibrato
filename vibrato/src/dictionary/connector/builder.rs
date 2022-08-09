use std::io::{prelude::*, BufReader, Read};

use crate::dictionary::Connector;
use crate::errors::{Result, VibratoError};

impl Connector {
    /// Creates a new instance from `matrix.def`.
    ///
    /// Note that the reader is buffered automatically, so you should not
    /// wrap `rdr` in a buffered reader like `io::BufReader`.
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
            let msg = format!(
                "The header must consists of two integers separated by spaces, {}",
                line
            );
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
                "A row other than the header must consists of three integers separated by spaces, {}",
                line
            );
            Err(VibratoError::invalid_format("matrix.def", msg))
        } else {
            Ok((cols[0].parse()?, cols[1].parse()?, cols[2].parse()?))
        }
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
        let conn = Connector::from_reader(data.as_bytes()).unwrap();
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
        let conn = Connector::from_reader(data.as_bytes()).unwrap();
        assert_eq!(conn.cost(0, 0), 0);
        assert_eq!(conn.cost(0, 1), 1);
        assert_eq!(conn.cost(0, 2), 2);
        assert_eq!(conn.cost(1, 0), -3);
        assert_eq!(conn.cost(1, 1), -4);
        assert_eq!(conn.cost(1, 2), -5);
    }

    #[test]
    #[should_panic]
    fn test_less_header() {
        let data = "2
0 0 0
0 1 1
1 0 -2
1 1 -3";
        Connector::from_reader(data.as_bytes()).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_more_header() {
        let data = "2 2 2
0 0 0
0 1 1
1 0 -2
1 1 -3";
        Connector::from_reader(data.as_bytes()).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_less_body() {
        let data = "2 2
0 0 0
0 1 1
1 -2
1 1 -3";
        Connector::from_reader(data.as_bytes()).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_more_body() {
        let data = "2 2
0 0 0
0 1 1
1 0 1 -2
1 1 -3";
        Connector::from_reader(data.as_bytes()).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_larger_matrix() {
        let data = "65536 65536";
        Connector::from_reader(data.as_bytes()).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_larger_left_id() {
        let data = "2 2
0 0 0
0 1 1
1 2 -2
1 1 -3";
        Connector::from_reader(data.as_bytes()).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_larger_right_id() {
        let data = "2 2
0 0 0
0 1 1
2 0 -2
1 1 -3";
        Connector::from_reader(data.as_bytes()).unwrap();
    }
}
