use std::io::{prelude::*, BufReader, Read};

use hashbrown::HashMap;
use regex::Regex;

use crate::dictionary::Connector;
use crate::errors::{Result, VibratoError};

impl Connector {
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

    /// Creates a new instance from `bigram.left`, `bigram.right`, and `bigram.weight`.
    pub fn from_origin<L, R, B>(left_rdr: L, right_rdr: R, bigram_weight_rdr: B) -> Result<Self>
    where
        L: Read,
        R: Read,
        B: Read,
    {
        let mut left_feature_ids = HashMap::new();
        let mut right_feature_ids = HashMap::new();
        let mut map = std::collections::HashMap::new();
        let weight_re = Regex::new(r"^(\S*)/(\S*)\t(\-?[0-9]+)$").unwrap();
        let bigram_weight_reader = BufReader::new(bigram_weight_rdr);
        for line in bigram_weight_reader.lines() {
            let line = line?;
            if let Some(cap) = weight_re.captures(&line) {
                let right_feature_str = cap.get(1).unwrap().as_str();
                let left_feature_str = cap.get(2).unwrap().as_str();
                let weight: i32 = cap.get(3).unwrap().as_str().parse()?;
                let new_right_id = u32::try_from(right_feature_ids.len()).unwrap() + 1;
                let new_left_id = u32::try_from(left_feature_ids.len()).unwrap() + 1;
                let right_id = *right_feature_ids.raw_entry_mut().from_key(right_feature_str).or_insert_with(|| (right_feature_str.to_string(), new_right_id)).1;
                let left_id = *left_feature_ids.raw_entry_mut().from_key(left_feature_str).or_insert_with(|| (left_feature_str.to_string(), new_left_id)).1;
                map.insert((right_id, left_id), weight);
            } else {
                return Err(VibratoError::invalid_format("bigram.weight", "invalid weight format"))
            }
        }

        let feature_line_re = Regex::new(r"^([0-9]+)( (.*))?$").unwrap();
        let feature_item_re = Regex::new(r"^([0-9]+):(.*)$").unwrap();

        let mut left_features = vec![];
        let left_reader = BufReader::new(left_rdr);
        for (i, line) in left_reader.lines().enumerate() {
            let line = line?;
            if let Some(cap) = feature_line_re.captures(&line) {
                let idx: usize = cap.get(1).unwrap().as_str().parse().unwrap();
                if idx != i + 1 {
                    return Err(VibratoError::invalid_format("bigram.left", "invalid weight format"))
                }
                let mut feat_list = vec![];
                if let Some(feats_str) = cap.get(3) {
                for feat in feats_str.as_str().split(' ') {
                    if let Some(cap) = feature_item_re.captures(feat) {
                        let idx: usize = cap.get(1).unwrap().as_str().parse().unwrap();
                        let feat_str = cap.get(2).unwrap().as_str();
                        if idx >= feat_list.len() {
                            feat_list.resize(idx + 1, 0);
                        }
                        feat_list[idx] = *left_feature_ids.get(feat_str).unwrap();
                    } else {
                        return Err(VibratoError::invalid_format("bigram.left", "invalid weight format"))
                    }
                }
                }
                left_features.push(feat_list);
            } else {
                return Err(VibratoError::invalid_format("bigram.left", "invalid weight format"))
            }
        }

        let mut right_features = vec![];
        let right_reader = BufReader::new(right_rdr);
        for (i, line) in right_reader.lines().enumerate() {
            let line = line?;
            if let Some(cap) = feature_line_re.captures(&line) {
                let idx: usize = cap.get(1).unwrap().as_str().parse().unwrap();
                if idx != i + 1 {
                    return Err(VibratoError::invalid_format("bigram.right", "invalid weight format"))
                }
                let mut feat_list = vec![];
                if let Some(feats_str) = cap.get(3) {
                for feat in feats_str.as_str().split(' ') {
                    if let Some(cap) = feature_item_re.captures(feat) {
                        let idx: usize = cap.get(1).unwrap().as_str().parse().unwrap();
                        let feat_str = cap.get(2).unwrap().as_str();
                        if idx >= feat_list.len() {
                            feat_list.resize(idx + 1, 0);
                        }
                        feat_list[idx] = *right_feature_ids.get(feat_str).unwrap();
                    } else {
                        return Err(VibratoError::invalid_format("bigram.right", "invalid weight format"))
                    }
                }
                }
                right_features.push(feat_list);
            } else {
                return Err(VibratoError::invalid_format("bigram.right", "invalid weight format"))
            }
        }

        Ok(Self::new_detailed(map, right_features, left_features))
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
    fn test_less_header() {
        let data = "2
0 0 0
0 1 1
1 0 -2
1 1 -3";
        let result = Connector::from_reader(data.as_bytes());

        assert!(result.is_err());
    }

    #[test]
    fn test_more_header() {
        let data = "2 2 2
0 0 0
0 1 1
1 0 -2
1 1 -3";
        let result = Connector::from_reader(data.as_bytes());

        assert!(result.is_err());
    }

    #[test]
    fn test_less_body() {
        let data = "2 2
0 0 0
0 1 1
1 -2
1 1 -3";
        let result = Connector::from_reader(data.as_bytes());

        assert!(result.is_err());
    }

    #[test]
    fn test_more_body() {
        let data = "2 2
0 0 0
0 1 1
1 0 1 -2
1 1 -3";
        let result = Connector::from_reader(data.as_bytes());

        assert!(result.is_err());
    }

    #[test]
    fn test_larger_matrix() {
        let data = "65536 65536";
        let result = Connector::from_reader(data.as_bytes());

        assert!(result.is_err());
    }

    #[test]
    fn test_larger_left_id() {
        let data = "2 2
0 0 0
0 1 1
1 2 -2
1 1 -3";
        let result = Connector::from_reader(data.as_bytes());

        assert!(result.is_err());
    }

    #[test]
    fn test_larger_right_id() {
        let data = "2 2
0 0 0
0 1 1
2 0 -2
1 1 -3";
        let result = Connector::from_reader(data.as_bytes());

        assert!(result.is_err());
    }
}
