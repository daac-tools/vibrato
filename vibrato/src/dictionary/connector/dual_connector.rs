use std::io::Read;

use bincode::{Decode, Encode};
use hashbrown::{HashMap, HashSet};

use crate::dictionary::connector::raw_connector::scorer::{
    Scorer, ScorerBuilder, U31x8, SIMD_SIZE,
};
use crate::dictionary::connector::raw_connector::{RawConnectorBuilder, INVALID_FEATURE_ID};
use crate::dictionary::connector::{Connector, ConnectorCost, MatrixConnector};
use crate::dictionary::mapper::ConnIdMapper;
use crate::errors::Result;
use crate::num::U31;

#[derive(Decode, Encode)]
pub struct DualConnector {
    matrix_connector: MatrixConnector,
    right_conn_id_map: Vec<u16>,
    left_conn_id_map: Vec<u16>,
    right_feat_ids: Vec<U31x8>,
    left_feat_ids: Vec<U31x8>,
    raw_scorer: Scorer,
}

impl DualConnector {
    /// Removes feature templates so that the matrix size is smaller using greedy search
    /// and returns a set of rest IDs.
    pub fn remove_feature_templates_greedy(
        raw_feat_template_size: usize,
        right_feat_ids_tmp: &[Vec<U31>],
        left_feat_ids_tmp: &[Vec<U31>],
        total_feat_template_size: usize,
    ) -> HashSet<usize> {
        let mut matrix_indices: HashSet<usize> = (0..total_feat_template_size).collect();
        eprintln!(
            "Initial matrix size: {}",
            left_feat_ids_tmp.len() * right_feat_ids_tmp.len()
        );
        for _ in 0..raw_feat_template_size {
            let mut candidate_idx = 0;
            let mut min_matrix_size = left_feat_ids_tmp.len() * right_feat_ids_tmp.len();
            for &trial_idx in &matrix_indices {
                let calculate_num_conn_ids = |feat_ids_tmp: &[Vec<U31>]| {
                    let mut map = HashMap::new();
                    for row in feat_ids_tmp {
                        let mut new_feats = vec![];
                        for &i in &matrix_indices {
                            if i != trial_idx {
                                if let Some(f) = row.get(i) {
                                    new_feats.push(f);
                                }
                            }
                        }
                        *map.entry(new_feats).or_insert(0) += 1;
                    }
                    map.len()
                };
                let right_num_conn_ids = calculate_num_conn_ids(right_feat_ids_tmp);
                let left_num_conn_ids = calculate_num_conn_ids(left_feat_ids_tmp);
                if right_num_conn_ids * left_num_conn_ids <= min_matrix_size {
                    min_matrix_size = right_num_conn_ids * left_num_conn_ids;
                    candidate_idx = trial_idx;
                }
            }
            eprintln!(
                "Removed feature template: #{}, matrix size: {}",
                candidate_idx, min_matrix_size
            );
            matrix_indices.remove(&candidate_idx);
        }
        matrix_indices
    }

    fn create_matrix_connector(
        right_feat_ids_tmp: &[Vec<U31>],
        left_feat_ids_tmp: &[Vec<U31>],
        matrix_indices: &[usize],
        feat_template_size: usize,
        scorer: &Scorer,
    ) -> (MatrixConnector, Vec<u16>, Vec<u16>) {
        let generate_feature_map = |feat_ids_tmp: &[Vec<U31>]| {
            let mut conn_id_map = vec![0];
            let mut feats_map = HashMap::new();
            feats_map.insert(vec![U31::default(); feat_template_size - SIMD_SIZE], 0);
            for row in feat_ids_tmp {
                let mut feat_ids = vec![];
                for &idx in matrix_indices {
                    feat_ids.push(*row.get(idx).unwrap_or(&INVALID_FEATURE_ID));
                }
                let new_conn_id = feats_map.len();
                let conn_id = *feats_map.entry(feat_ids).or_insert(new_conn_id);
                conn_id_map.push(u16::try_from(conn_id).unwrap());
            }
            (conn_id_map, feats_map)
        };
        let (right_conn_id_map, right_feats_map) = generate_feature_map(right_feat_ids_tmp);
        let (left_conn_id_map, left_feats_map) = generate_feature_map(left_feat_ids_tmp);
        let mut matrix = vec![0; right_feats_map.len() * left_feats_map.len()];
        for (right_feats, rid) in &right_feats_map {
            for (left_feats, lid) in &left_feats_map {
                let cost = scorer.accumulate_cost(
                    &U31x8::to_simd_vec(right_feats),
                    &U31x8::to_simd_vec(left_feats),
                );
                let index = *lid * right_feats_map.len() + *rid;
                matrix[index] = cost.min(i16::MAX as i32).max(i16::MIN as i32) as i16;
            }
        }
        let matrix_connector =
            MatrixConnector::new(matrix, right_feats_map.len(), left_feats_map.len());
        (matrix_connector, right_conn_id_map, left_conn_id_map)
    }

    fn create_raw_connector(
        right_feat_ids_tmp: &[Vec<U31>],
        left_feat_ids_tmp: &[Vec<U31>],
        raw_indices: &[usize],
        scorer_builder: &mut ScorerBuilder,
    ) -> (Vec<U31>, Vec<U31>) {
        let mut right_feat_ids = vec![U31::default(); raw_indices.len()];
        let mut left_feat_ids = vec![U31::default(); raw_indices.len()];
        for row in right_feat_ids_tmp {
            for &idx in raw_indices {
                right_feat_ids.push(*row.get(idx).unwrap_or(&INVALID_FEATURE_ID));
            }
        }
        for row in left_feat_ids_tmp {
            for &idx in raw_indices {
                left_feat_ids.push(*row.get(idx).unwrap_or(&INVALID_FEATURE_ID));
            }
        }
        let right_used_feats: HashSet<_> = right_feat_ids.iter().cloned().collect();
        let left_used_feats: HashSet<_> = left_feat_ids.iter().cloned().collect();
        for (i, left_map) in scorer_builder.trie.iter_mut().enumerate() {
            if right_used_feats.contains(&U31::new(u32::try_from(i).unwrap()).unwrap()) {
                let left_feat_ids: Vec<_> = left_map.keys().cloned().collect();
                for feat_id in left_feat_ids {
                    if !left_used_feats.contains(&feat_id) {
                        left_map.remove(&feat_id);
                    }
                }
            } else {
                left_map.clear();
            }
        }
        (right_feat_ids, left_feat_ids)
    }

    /// Creates a new instance from `bigram.right`, `bigram.left`, and `bigram.cost`.
    pub fn from_readers<R, L, C>(right_rdr: R, left_rdr: L, cost_rdr: C) -> Result<Self>
    where
        R: Read,
        L: Read,
        C: Read,
    {
        let RawConnectorBuilder {
            right_feat_ids_tmp,
            left_feat_ids_tmp,
            feat_template_size,
            mut scorer_builder,
        } = RawConnectorBuilder::from_readers(right_rdr, left_rdr, cost_rdr)?;
        let scorer = scorer_builder.build();

        // Split features into RawConnector and MatrixConnector
        let matrix_ids_set = Self::remove_feature_templates_greedy(
            SIMD_SIZE,
            &right_feat_ids_tmp,
            &left_feat_ids_tmp,
            feat_template_size,
        );
        let mut matrix_indices = vec![];
        let mut raw_indices = vec![];
        for i in 0..feat_template_size {
            if matrix_ids_set.contains(&i) {
                matrix_indices.push(i);
            } else {
                raw_indices.push(i);
            }
        }

        let (matrix_connector, right_conn_id_map, left_conn_id_map) = Self::create_matrix_connector(
            &right_feat_ids_tmp,
            &left_feat_ids_tmp,
            &matrix_indices,
            feat_template_size,
            &scorer,
        );
        let (right_feat_ids, left_feat_ids) = Self::create_raw_connector(
            &right_feat_ids_tmp,
            &left_feat_ids_tmp,
            &raw_indices,
            &mut scorer_builder,
        );

        Ok(Self {
            matrix_connector,
            right_conn_id_map,
            left_conn_id_map,
            right_feat_ids: U31x8::to_simd_vec(&right_feat_ids),
            left_feat_ids: U31x8::to_simd_vec(&left_feat_ids),
            raw_scorer: scorer_builder.build(),
        })
    }
}

impl Connector for DualConnector {
    #[inline(always)]
    fn num_left(&self) -> usize {
        self.left_conn_id_map.len()
    }

    #[inline(always)]
    fn num_right(&self) -> usize {
        self.right_conn_id_map.len()
    }

    fn map_connection_ids(&mut self, mapper: &ConnIdMapper) {
        assert_eq!(mapper.num_left(), self.num_left());
        assert_eq!(mapper.num_right(), self.num_right());

        let mut new_right_feat_ids = vec![U31x8::default(); self.right_feat_ids.len()];
        let mut new_right_conn_id_map = vec![0; self.right_conn_id_map.len()];
        for right_id in 0..self.num_right() {
            let new_id = usize::from(mapper.right(u16::try_from(right_id).unwrap()));
            new_right_feat_ids[new_id] = self.right_feat_ids[right_id];
            new_right_conn_id_map[new_id] = self.right_conn_id_map[right_id];
        }
        self.right_feat_ids = new_right_feat_ids;
        self.right_conn_id_map = new_right_conn_id_map;

        let mut new_left_feat_ids = vec![U31x8::default(); self.left_feat_ids.len()];
        let mut new_left_conn_id_map = vec![0; self.left_conn_id_map.len()];
        for left_id in 0..self.num_left() {
            let new_id = usize::from(mapper.left(u16::try_from(left_id).unwrap()));
            new_left_feat_ids[new_id] = self.left_feat_ids[left_id];
            new_left_conn_id_map[new_id] = self.left_conn_id_map[left_id];
        }
        self.left_feat_ids = new_left_feat_ids;
        self.left_conn_id_map = new_left_conn_id_map;

        let mut matrix_mapper_left = vec![u16::MAX; self.matrix_connector.num_left()];
        let mut matrix_mapper_right = vec![u16::MAX; self.matrix_connector.num_right()];
        let mut left_id = 0;
        let mut right_id = 0;
        for i in &mut self.left_conn_id_map {
            let map = &mut matrix_mapper_left[usize::from(*i)];
            if *map != u16::MAX {
                *i = *map;
                continue;
            }
            *map = left_id;
            *i = left_id;
            left_id += 1;
        }
        for i in &mut self.right_conn_id_map {
            let map = &mut matrix_mapper_right[usize::from(*i)];
            if *map != u16::MAX {
                *i = *map;
                continue;
            }
            *map = right_id;
            *i = right_id;
            right_id += 1;
        }
        let matrix_mapper = ConnIdMapper::new(matrix_mapper_left, matrix_mapper_right);
        self.matrix_connector.map_connection_ids(&matrix_mapper);
    }
}

impl ConnectorCost for DualConnector {
    #[inline(always)]
    fn cost(&self, right_id: u16, left_id: u16) -> i32 {
        let right_conn_id = self.right_conn_id_map[usize::from(right_id)];
        let left_conn_id = self.left_conn_id_map[usize::from(left_id)];
        let matrix_cost = self.matrix_connector.cost(right_conn_id, left_conn_id);
        let raw_cost = self.raw_scorer.accumulate_cost(
            &[self.right_feat_ids[usize::from(right_id)]],
            &[self.left_feat_ids[usize::from(left_id)]],
        );
        matrix_cost + raw_cost
    }

    #[inline(always)]
    unsafe fn cost_unchecked(&self, right_id: u16, left_id: u16) -> i32 {
        let right_conn_id = *self.right_conn_id_map.get_unchecked(usize::from(right_id));
        let left_conn_id = *self.left_conn_id_map.get_unchecked(usize::from(left_id));
        let matrix_cost = self
            .matrix_connector
            .cost_unchecked(right_conn_id, left_conn_id);
        let raw_cost = self.raw_scorer.accumulate_cost(
            &[*self.right_feat_ids.get_unchecked(usize::from(right_id))],
            &[*self.left_feat_ids.get_unchecked(usize::from(left_id))],
        );
        matrix_cost + raw_cost
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_readers_test() {
        let right_rdr = "\
1\tAB,*,CD,*,EF,*,GH,*,IJ,*,KL,*,MN,*,OP,*,QR,*,ST
2\tUV,*,WX,*,YZ,*,12,*,34,*,56,*,78,*,90,*,*,*,*"
            .as_bytes();
        let left_rdr = "\
1\tuv,*,wx,*,yz,*,12,*,34,*,56,*,78,*,90,*,*,*,*
2\tab,*,cd,*,ef,*,gh,*,ij,*,kl,*,mn,*,op,*,qr,*,st"
            .as_bytes();
        let cost_rdr = "\
AB/ab\t-10
CD/cd\t20
EF/ef\t-30
GH/gh\t40
IJ/ij\t-50
KL/kl\t60
MN/mn\t-70
OP/op\t80
QR/qr\t-90
ST/st\t100
UV/uv\t-110
WX/wx\t120
YZ/yz\t-130
12/12\t140
34/34\t-150
56/56\t160
78/78\t-170
90/90\t180"
            .as_bytes();

        let conn = DualConnector::from_readers(right_rdr, left_rdr, cost_rdr).unwrap();

        assert_eq!(conn.cost(1, 2), 50);
        assert_eq!(conn.cost(2, 1), 40);
    }

    #[test]
    fn mapping_test() {
        let right_rdr = "\
1\tAB,*,CD,*,EF,*,GH,*,IJ,*,KL,*,MN,*,OP,*,QR,*,ST
2\tUV,*,WX,*,YZ,*,12,*,34,*,56,*,78,*,90,*,*,*,*"
            .as_bytes();
        let left_rdr = "\
1\tuv,*,wx,*,yz,*,12,*,34,*,56,*,78,*,90,*,*,*,*
2\tab,*,cd,*,ef,*,gh,*,ij,*,kl,*,mn,*,op,*,qr,*,st"
            .as_bytes();
        let cost_rdr = "\
AB/ab\t-10
CD/cd\t20
EF/ef\t-30
GH/gh\t40
IJ/ij\t-50
KL/kl\t60
MN/mn\t-70
OP/op\t80
QR/qr\t-90
ST/st\t100
UV/uv\t-110
WX/wx\t120
YZ/yz\t-130
12/12\t140
34/34\t-150
56/56\t160
78/78\t-170
90/90\t180"
            .as_bytes();

        let mut conn = DualConnector::from_readers(right_rdr, left_rdr, cost_rdr).unwrap();

        let mapper = ConnIdMapper::new(vec![1, 2, 0], vec![2, 0, 1]);
        conn.map_connection_ids(&mapper);

        assert_eq!(conn.cost(0, 0), 50);
        assert_eq!(conn.cost(1, 2), 40);
    }
}
