mod scorer;

use std::io::{prelude::*, BufReader, Read};

use bincode::{Decode, Encode};
use hashbrown::HashMap;

use crate::dictionary::connector::raw_connector::scorer::{Scorer, ScorerBuilder};
use crate::dictionary::connector::{Connector, ConnectorCost};
use crate::dictionary::mapper::ConnIdMapper;
use crate::errors::{Result, VibratoError};
use crate::utils;

const INVALID_FEATURE_ID: u32 = u32::MAX;

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

    /// Creates a new instance from `bigram.right`, `bigram.left`, and `bigram.cost`.
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
            col_size = col_size.max(feature_ids.len());
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
            col_size = col_size.max(feature_ids.len());
            left_ids_tmp.push(feature_ids);
        }

        // Converts a vector of N vectors into a matrix of size (N+1)*M,
        // where M is the maximum length of a vector in the N vectors.
        //
        // All short vectors are padded with INVALID_FEATURE_IDs.
        let mut right_ids = vec![INVALID_FEATURE_ID; (right_ids_tmp.len() + 1) * col_size];
        let mut left_ids = vec![INVALID_FEATURE_ID; (left_ids_tmp.len() + 1) * col_size];

        // The first row reserved for BOS/EOS is always an empty row with zero values.
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

    /// Parses a line in file `bigram.right/left`, returning the entry id and a sequence of feature
    /// ids. If a feature is not stored in the given id map, `INVALID_FEATURE_ID` is used as the
    /// feature id.
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

    /// Parses a line in file `bigram.cost`, returning the ids of right and left features and the
    /// connection cost.
    ///
    /// If a feature is already stored in the given id map, the assigned id is returned;
    /// otherwise, the feature is inserted into the map, and a new id is returned.
    ///
    /// # Examples
    ///
    /// * Input
    ///   * `line = B3:名詞,普通名詞,一般/名詞,普通名詞,サ変可能\t-520`
    ///   * `right_id_map = {"B3:名詞,普通名詞,一般": 0, "B2:名詞,固有名詞": 1}`
    ///   * `left_id_map = {"名詞,普通名詞,一般": 0}`
    /// * Output
    ///   * `(right_id, left_id, cost) = (0, 1, -520)`
    ///   * `right_id_map = {"B3:名詞,普通名詞,一般": 0, "B2:名詞,固有名詞": 1}`
    ///   * `left_id_map = {"名詞,普通名詞,一般": 0, "名詞,普通名詞,サ変可能": 1}`
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

    fn do_mapping(&mut self, mapper: &ConnIdMapper) {
        assert_eq!(mapper.num_left(), self.num_left());
        assert_eq!(mapper.num_right(), self.num_right());

        let mut mapped = vec![0; self.right_ids.len()];
        for right_id in 0..self.num_right() {
            let new_right_id = usize::from(mapper.right(u16::try_from(right_id).unwrap()));
            mapped[new_right_id * self.col_size..(new_right_id + 1) * self.col_size]
                .copy_from_slice(
                    &self.right_ids[right_id * self.col_size..(right_id + 1) * self.col_size],
                );
        }
        self.right_ids = mapped;

        let mut mapped = vec![0; self.left_ids.len()];
        for left_id in 0..self.num_left() {
            let new_left_id = usize::from(mapper.right(u16::try_from(left_id).unwrap()));
            mapped[new_left_id * self.col_size..(new_left_id + 1) * self.col_size].copy_from_slice(
                &self.left_ids[left_id * self.col_size..(left_id + 1) * self.col_size],
            );
        }
        self.left_ids = mapped;
    }
}

impl ConnectorCost for RawConnector {
    #[inline(always)]
    fn cost(&self, right_id: u16, left_id: u16) -> i32 {
        self.scorer.accumulate_cost(
            self.right_feature_ids(right_id),
            self.left_feature_ids(left_id),
        )
    }

    /// TODO: Implement unchecked optimization.
    #[inline(always)]
    unsafe fn cost_unchecked(&self, right_id: u16, left_id: u16) -> i32 {
        self.cost(right_id, left_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::utils::hashmap;

    #[test]
    fn parse_cost_test() {
        let mut right_id_map = HashMap::new();
        let mut left_id_map = HashMap::new();

        assert_eq!(
            RawConnector::parse_cost(
                "SURF-SURF:これ/は\t-100",
                &mut right_id_map,
                &mut left_id_map
            )
            .unwrap(),
            (0, 0, -100),
        );
        assert_eq!(
            RawConnector::parse_cost(
                "SURF-POS:これ/助詞\t200",
                &mut right_id_map,
                &mut left_id_map
            )
            .unwrap(),
            (1, 1, 200),
        );
        assert_eq!(
            RawConnector::parse_cost(
                "POS-SURF:代名詞/は\t-300",
                &mut right_id_map,
                &mut left_id_map
            )
            .unwrap(),
            (2, 0, -300),
        );

        assert_eq!(
            hashmap![
                "SURF-SURF:これ".to_string() => 0,
                "SURF-POS:これ".to_string() => 1,
                "POS-SURF:代名詞".to_string() => 2,
            ],
            right_id_map,
        );
        assert_eq!(
            hashmap![
                "は".to_string() => 0,
                "助詞".to_string() => 1,
            ],
            left_id_map,
        );
    }

    #[test]
    fn parse_cost_invalid_feature_test() {
        let mut right_id_map = HashMap::new();
        let mut left_id_map = HashMap::new();

        assert!(RawConnector::parse_cost(
            "SURF-SURF:これは\t100",
            &mut right_id_map,
            &mut left_id_map
        )
        .is_err());
    }

    #[test]
    fn parse_cost_invalid_tab_test() {
        let mut right_id_map = HashMap::new();
        let mut left_id_map = HashMap::new();

        assert!(RawConnector::parse_cost(
            "SURF-SURF:これ/は100",
            &mut right_id_map,
            &mut left_id_map
        )
        .is_err());
    }

    #[test]
    fn parse_cost_invalid_cost_test() {
        let mut right_id_map = HashMap::new();
        let mut left_id_map = HashMap::new();

        assert!(RawConnector::parse_cost(
            "SURF-SURF:これ/は\tabc",
            &mut right_id_map,
            &mut left_id_map
        )
        .is_err());
    }

    #[test]
    fn parse_feature_test() {
        let id_map = hashmap![
            "これ".to_string() => 0,
            "助詞".to_string() => 1,
            "コレ".to_string() => 2,
            "これ,助詞".to_string() => 3,
            "これ,コレ".to_string() => 4,
        ];

        assert_eq!(
            RawConnector::parse_features("2\tこれ,*,コレ,\"これ,助詞\",*", &id_map, "bigram.left",)
                .unwrap(),
            (2, vec![0, INVALID_FEATURE_ID, 2, 3, INVALID_FEATURE_ID]),
        );
    }

    #[test]
    fn parse_feature_invalid_id_test() {
        let id_map = hashmap![
            "これ".to_string() => 0,
            "助詞".to_string() => 1,
            "コレ".to_string() => 2,
            "これ,助詞".to_string() => 3,
            "これ,コレ".to_string() => 4,
        ];

        assert!(RawConnector::parse_features(
            "これ,*,コレ,\"これ,助詞\",*",
            &id_map,
            "bigram.left",
        )
        .is_err());
    }

    #[test]
    fn from_readers_test() {
        let right_rdr = "\
1\tSURF-SURF:これ,*,SURF-POS:これ,POS-SURF:代名詞,*
2\tSURF-SURF:テスト,*,SURF-POS:テスト,POS-SURF:名詞,*"
            .as_bytes();
        let left_rdr = "\
1\tです,*,助動詞,です,*
2\tは,*,助詞,は,*"
            .as_bytes();
        let cost_rdr = "\
SURF-SURF:これ/は\t-100
SURF-POS:これ/助詞\t200
POS-SURF:代名詞/は\t-300"
            .as_bytes();

        let conn = RawConnector::from_readers(right_rdr, left_rdr, cost_rdr).unwrap();

        assert_eq!(conn.cost(1, 2), -200);
    }
}
