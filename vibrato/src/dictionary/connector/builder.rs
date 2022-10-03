use std::io::{prelude::*, BufReader, Read};

use hashbrown::HashMap;

use crate::dictionary::connector::scorer::ScorerBuilder;
use crate::dictionary::connector::{MatrixConnector, RawConnector};
use crate::errors::{Result, VibratoError};
use crate::utils;

const INVALID_FEATURE_ID: u32 = u32::MAX;

impl MatrixConnector {
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
}

impl RawConnector {
    /// Creates a new instance from `matrix.def`.
    pub fn from_readers<R, L, C>(right_rdr: R, left_rdr: L, cost_rdr: C) -> Result<Self>
    where
        R: Read,
        L: Read,
        C: Read,
    {
        let mut right_id_map = HashMap::new();
        let mut left_id_map = HashMap::new();
        right_id_map.insert(String::new(), 0);
        left_id_map.insert(String::new(), 0);
        let mut scorer_builder = ScorerBuilder::new();

        let cost_rdr = BufReader::new(cost_rdr);
        for line in cost_rdr.lines() {
            let line = line?;
            let (right_id, left_id, cost) =
                Self::parse_cost(&line, &mut right_id_map, &mut left_id_map)?;
            scorer_builder.insert(right_id, left_id, cost);
        }
        let scorer = scorer_builder.build();

        let mut col_size = 0;
        let mut right_ids_tmp = vec![];
        let right_rdr = BufReader::new(right_rdr);
        for (i, line) in right_rdr.lines().enumerate() {
            let line = line?;
            let (id, feature_ids) = Self::parse_features(&line, &right_id_map, "bigram.right")?;
            if id != i + 1 {
                return Err(VibratoError::invalid_format(
                    "bigram.right",
                    "must be ascending order",
                ));
            }
            if feature_ids.len() > col_size {
                col_size = feature_ids.len();
            }
            right_ids_tmp.push(feature_ids);
        }

        let mut left_ids_tmp = vec![];
        let left_rdr = BufReader::new(left_rdr);
        for (i, line) in left_rdr.lines().enumerate() {
            let line = line?;
            let (id, feature_ids) = Self::parse_features(&line, &left_id_map, "bigram.left")?;
            if id != i + 1 {
                return Err(VibratoError::invalid_format(
                    "bigram.left",
                    "must be ascending order",
                ));
            }
            if feature_ids.len() > col_size {
                col_size = feature_ids.len();
            }
            left_ids_tmp.push(feature_ids);
        }

        let mut right_ids = vec![INVALID_FEATURE_ID; (right_ids_tmp.len() + 1) * col_size];
        let mut left_ids = vec![INVALID_FEATURE_ID; (left_ids_tmp.len() + 1) * col_size];
        right_ids[..col_size].fill(0);
        left_ids[..col_size].fill(0);
        for (trg, src) in right_ids[col_size..]
            .chunks_mut(col_size)
            .zip(&right_ids_tmp)
        {
            trg[..src.len()].copy_from_slice(src);
        }
        for (trg, src) in left_ids[col_size..].chunks_mut(col_size).zip(&left_ids_tmp) {
            trg[..src.len()].copy_from_slice(src);
        }

        Ok(Self::new(right_ids, left_ids, col_size, scorer))
    }

    fn parse_features(
        line: &str,
        id_map: &HashMap<String, u32>,
        name: &'static str,
    ) -> Result<(usize, Vec<u32>)> {
        let mut spl = line.split('\t');
        let id_str = spl.next();
        let features_str = spl.next();
        let rest = spl.next();
        if let (Some(id_str), Some(features_str), None) = (id_str, features_str, rest) {
            let id: usize = id_str.parse()?;
            let features = utils::parse_csv_row(features_str);
            let mut result = vec![];
            for feature in features {
                result.push(*id_map.get(&feature).unwrap_or(&INVALID_FEATURE_ID));
            }
            return Ok((id, result));
        }
        let msg = format!("The format must be id<tab>csv_row, {line}");
        Err(VibratoError::invalid_format(name, msg))
    }

    fn parse_cost(
        line: &str,
        right_id_map: &mut HashMap<String, u32>,
        left_id_map: &mut HashMap<String, u32>,
    ) -> Result<(u32, u32, i32)> {
        let mut spl = line.split('\t');
        let feature_str = spl.next();
        let cost_str = spl.next();
        let rest = spl.next();
        if let (Some(feature_str), Some(cost_str), None) = (feature_str, cost_str, rest) {
            let cost: i32 = cost_str.parse()?;
            let mut spl = feature_str.split('/');
            let right_str = spl.next();
            let left_str = spl.next();
            let rest = spl.next();
            if let (Some(right_str), Some(left_str), None) = (right_str, left_str, rest) {
                let new_right_id = u32::try_from(right_id_map.len()).unwrap();
                let right_id = *right_id_map
                    .raw_entry_mut()
                    .from_key(right_str)
                    .or_insert_with(|| (right_str.to_string(), new_right_id))
                    .1;
                let new_left_id = u32::try_from(left_id_map.len()).unwrap();
                let left_id = *left_id_map
                    .raw_entry_mut()
                    .from_key(left_str)
                    .or_insert_with(|| (left_str.to_string(), new_left_id))
                    .1;
                return Ok((right_id, left_id, cost));
            }
        }
        let msg = format!("The format must be right/left<tab>cost, {line}");
        Err(VibratoError::invalid_format("bigram.cost", msg))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::dictionary::connector::ConnectorCost;

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
