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
    matrix_left_id_map: Vec<u16>,
    matrix_right_id_map: Vec<u16>,
    raw_right_ids: Vec<U31x8>,
    raw_left_ids: Vec<U31x8>,
    raw_scorer: Scorer,
}

impl DualConnector {
    /// Removes feature templates so that the matrix size is smaller using greedy search
    /// and returns a set of rest IDs.
    pub fn remove_feature_templates_greedy(
        raw_id_size: usize,
        left_ids: &[Vec<U31>],
        right_ids: &[Vec<U31>],
        col_size: usize,
    ) -> HashSet<usize> {
        let mut matrix_ids: HashSet<usize> = (0..col_size).collect();
        eprintln!("Initial matrix size: {}", left_ids.len() * right_ids.len());
        for _ in 0..raw_id_size {
            let mut candidate_idx = 0;
            let mut min_matrix_size = left_ids.len() * right_ids.len();
            for &trial_idx in &matrix_ids {
                let mut right_map = HashMap::new();
                let mut left_map = HashMap::new();
                for right_features in right_ids {
                    let mut new_right_features = vec![];
                    for &i in &matrix_ids {
                        if i != trial_idx {
                            if let Some(f) = right_features.get(i) {
                                new_right_features.push(f);
                            }
                        }
                    }
                    *right_map.entry(new_right_features).or_insert(0) += 1;
                }
                for left_features in left_ids {
                    let mut new_left_features = vec![];
                    for &i in &matrix_ids {
                        if i != trial_idx {
                            if let Some(f) = left_features.get(i) {
                                new_left_features.push(f);
                            }
                        }
                    }
                    *left_map.entry(new_left_features).or_insert(0) += 1;
                }
                if right_map.len() * left_map.len() < min_matrix_size {
                    min_matrix_size = right_map.len() * left_map.len();
                    candidate_idx = trial_idx;
                }
            }
            eprintln!(
                "Removed feature template: #{}, matrix size: {}",
                candidate_idx, min_matrix_size
            );
            matrix_ids.remove(&candidate_idx);
        }
        matrix_ids
    }

    fn create_matrix_connector(
        right_ids_tmp: &[Vec<U31>],
        left_ids_tmp: &[Vec<U31>],
        matrix_indices: &[usize],
        col_size: usize,
        scorer: &Scorer,
    ) -> (MatrixConnector, Vec<u16>, Vec<u16>) {
        let mut right_id_map = vec![0];
        let mut left_id_map = vec![0];
        let mut right_features_map = HashMap::new();
        let mut left_features_map = HashMap::new();
        right_features_map.insert(vec![U31::default(); col_size - SIMD_SIZE], 0);
        left_features_map.insert(vec![U31::default(); col_size - SIMD_SIZE], 0);
        for right_features in right_ids_tmp {
            let mut right_feature_ids = vec![];
            for &idx in matrix_indices {
                right_feature_ids.push(*right_features.get(idx).unwrap_or(&INVALID_FEATURE_ID));
            }
            let right_new_id = right_features_map.len();
            let right_id = *right_features_map
                .entry(right_feature_ids)
                .or_insert(right_new_id);
            right_id_map.push(u16::try_from(right_id).unwrap());
        }
        for left_features in left_ids_tmp {
            let mut left_feature_ids = vec![];
            for &idx in matrix_indices {
                left_feature_ids.push(*left_features.get(idx).unwrap_or(&INVALID_FEATURE_ID));
            }
            let left_new_id = left_features_map.len();
            let left_id = *left_features_map
                .entry(left_feature_ids)
                .or_insert(left_new_id);
            left_id_map.push(u16::try_from(left_id).unwrap());
        }
        let mut matrix = vec![0; right_features_map.len() * left_features_map.len()];
        for (right_features, rid) in &right_features_map {
            for (left_features, lid) in &left_features_map {
                let cost = scorer.accumulate_cost(
                    &U31x8::to_simd_vec(right_features),
                    &U31x8::to_simd_vec(left_features),
                );
                let index = *lid * right_features_map.len() + *rid;
                matrix[index] = cost.min(i16::MAX as i32).max(i16::MIN as i32) as i16;
            }
        }
        let matrix_connector =
            MatrixConnector::new(matrix, right_features_map.len(), left_features_map.len());

        (matrix_connector, right_id_map, left_id_map)
    }

    fn create_raw_connector(
        right_ids_tmp: &[Vec<U31>],
        left_ids_tmp: &[Vec<U31>],
        raw_indices: &[usize],
        scorer_builder: &mut ScorerBuilder,
    ) -> (Vec<U31>, Vec<U31>) {
        let mut right_ids = vec![U31::default(); SIMD_SIZE];
        let mut left_ids = vec![U31::default(); SIMD_SIZE];
        for right_features in right_ids_tmp {
            for &idx in raw_indices {
                right_ids.push(*right_features.get(idx).unwrap_or(&INVALID_FEATURE_ID));
            }
        }
        for left_features in left_ids_tmp {
            for &idx in raw_indices {
                left_ids.push(*left_features.get(idx).unwrap_or(&INVALID_FEATURE_ID));
            }
        }
        let right_used_features: HashSet<_> = right_ids.iter().cloned().collect();
        let left_used_features: HashSet<_> = left_ids.iter().cloned().collect();
        for (i, left_map) in scorer_builder.trie.iter_mut().enumerate() {
            if right_used_features.contains(&U31::new(u32::try_from(i).unwrap()).unwrap()) {
                let left_indices: Vec<_> = left_map.keys().cloned().collect();
                for id in left_indices {
                    if !left_used_features.contains(&id) {
                        left_map.remove(&id);
                    }
                }
            } else {
                left_map.clear();
            }
        }

        (right_ids, left_ids)
    }

    /// Creates a new instance from `bigram.right`, `bigram.left`, and `bigram.cost`.
    pub fn from_readers<R, L, C>(right_rdr: R, left_rdr: L, cost_rdr: C) -> Result<Self>
    where
        R: Read,
        L: Read,
        C: Read,
    {
        let RawConnectorBuilder {
            right_ids_tmp,
            left_ids_tmp,
            col_size,
            mut scorer_builder,
        } = RawConnectorBuilder::from_readers(right_rdr, left_rdr, cost_rdr)?;
        let scorer = scorer_builder.build();

        // Split features into RawConnector and MatrixConnector
        let matrix_ids_set = Self::remove_feature_templates_greedy(
            SIMD_SIZE,
            &left_ids_tmp,
            &right_ids_tmp,
            col_size,
        );
        let mut matrix_indices = vec![];
        let mut raw_indices = vec![];
        for i in 0..col_size {
            if matrix_ids_set.contains(&i) {
                matrix_indices.push(i);
            } else {
                raw_indices.push(i);
            }
        }

        let (matrix_connector, matrix_right_id_map, matrix_left_id_map) =
            Self::create_matrix_connector(
                &right_ids_tmp,
                &left_ids_tmp,
                &matrix_indices,
                col_size,
                &scorer,
            );
        let (raw_right_ids, raw_left_ids) = Self::create_raw_connector(
            &right_ids_tmp,
            &left_ids_tmp,
            &raw_indices,
            &mut scorer_builder,
        );

        Ok(Self {
            matrix_connector,
            matrix_right_id_map,
            matrix_left_id_map,
            raw_right_ids: U31x8::to_simd_vec(&raw_right_ids),
            raw_left_ids: U31x8::to_simd_vec(&raw_left_ids),
            raw_scorer: scorer_builder.build(),
        })
    }
}

impl Connector for DualConnector {
    #[inline(always)]
    fn num_left(&self) -> usize {
        self.matrix_left_id_map.len()
    }

    #[inline(always)]
    fn num_right(&self) -> usize {
        self.matrix_right_id_map.len()
    }

    fn map_connection_ids(&mut self, mapper: &ConnIdMapper) {
        assert_eq!(mapper.num_left(), self.num_left());
        assert_eq!(mapper.num_right(), self.num_right());

        let mut new_raw_right_ids = vec![U31x8::default(); self.raw_right_ids.len()];
        let mut new_right_id_map = vec![0; self.matrix_right_id_map.len()];
        for right_id in 0..self.num_right() {
            let new_id = usize::from(mapper.right(u16::try_from(right_id).unwrap()));
            new_raw_right_ids[new_id] = self.raw_right_ids[right_id];
            new_right_id_map[new_id] = self.matrix_right_id_map[right_id];
        }
        self.raw_right_ids = new_raw_right_ids;
        self.matrix_right_id_map = new_right_id_map;

        let mut new_raw_left_ids = vec![U31x8::default(); self.raw_left_ids.len()];
        let mut new_left_id_map = vec![0; self.matrix_left_id_map.len()];
        for left_id in 0..self.num_left() {
            let new_id = usize::from(mapper.left(u16::try_from(left_id).unwrap()));
            new_raw_left_ids[new_id] = self.raw_left_ids[left_id];
            new_left_id_map[new_id] = self.matrix_left_id_map[left_id];
        }
        self.raw_left_ids = new_raw_left_ids;
        self.matrix_left_id_map = new_left_id_map;

        let mut matrix_mapper_left = vec![u16::MAX; self.matrix_connector.num_left()];
        let mut matrix_mapper_right = vec![u16::MAX; self.matrix_connector.num_right()];
        let mut left_id = 0;
        let mut right_id = 0;
        for i in &mut self.matrix_left_id_map {
            let map = &mut matrix_mapper_left[usize::from(*i)];
            if *map != u16::MAX {
                *i = *map;
                continue;
            }
            *map = left_id;
            *i = left_id;
            left_id += 1;
        }
        for i in &mut self.matrix_right_id_map {
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
        let matrix_right_id = self.matrix_right_id_map[usize::from(right_id)];
        let matrix_left_id = self.matrix_left_id_map[usize::from(left_id)];
        let matrix_cost = self.matrix_connector.cost(matrix_right_id, matrix_left_id);
        let raw_cost = self.raw_scorer.accumulate_cost(
            &[self.raw_right_ids[usize::from(right_id)]],
            &[self.raw_left_ids[usize::from(left_id)]],
        );
        matrix_cost + raw_cost
    }

    #[inline(always)]
    unsafe fn cost_unchecked(&self, right_id: u16, left_id: u16) -> i32 {
        let matrix_right_id = *self
            .matrix_right_id_map
            .get_unchecked(usize::from(right_id));
        let matrix_left_id = *self.matrix_left_id_map.get_unchecked(usize::from(left_id));
        let matrix_cost = self
            .matrix_connector
            .cost_unchecked(matrix_right_id, matrix_left_id);
        let raw_cost = self.raw_scorer.accumulate_cost(
            &[*self.raw_right_ids.get_unchecked(usize::from(right_id))],
            &[*self.raw_left_ids.get_unchecked(usize::from(left_id))],
        );
        matrix_cost + raw_cost
    }
}
