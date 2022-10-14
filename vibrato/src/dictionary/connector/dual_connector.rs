use std::io::{prelude::*, BufReader, Read};

use bincode::{Decode, Encode};
use hashbrown::{HashMap, HashSet};

use crate::dictionary::connector::raw_connector::scorer::{Scorer, ScorerBuilder, SIMD_SIZE};
use crate::dictionary::connector::raw_connector::RawConnectorBuilder;
use crate::dictionary::connector::{Connector, ConnectorCost, MatrixConnector, RawConnector};
use crate::dictionary::mapper::ConnIdMapper;
use crate::errors::{Result, VibratoError};
use crate::utils;

#[derive(Decode, Encode)]
pub struct DualConnector {
    matrix_connector: MatrixConnector,
    raw_connector: RawConnector,
    left_id_map: Vec<u16>,
    right_id_map: Vec<u16>,
}

impl DualConnector {
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
        //
        // Removes feature templates so that the matrix size is smaller using greedy search.
        let mut raw_set = HashSet::new();
        eprintln!(
            "Initial matrix size: {}",
            left_ids_tmp.len() * right_ids_tmp.len()
        );
        for _ in 0..SIMD_SIZE {
            let mut candidate_idx = 0;
            let mut min_matrix_size = left_ids_tmp.len() * right_ids_tmp.len();
            for trial_idx in 0..col_size {
                if raw_set.contains(&trial_idx) {
                    continue;
                }
                let mut right_map = HashMap::new();
                let mut left_map = HashMap::new();

                for right_features in &right_ids_tmp {
                    let mut new_right_features = vec![];
                    for (i, right) in right_features.iter().enumerate() {
                        if !raw_set.contains(&i) && i != trial_idx {
                            new_right_features.push(*right);
                        }
                    }
                    *right_map.entry(new_right_features).or_insert(0) += 1;
                }
                for left_features in &left_ids_tmp {
                    let mut new_left_features = vec![];
                    for (i, left) in left_features.iter().enumerate() {
                        if !raw_set.contains(&i) && i != trial_idx {
                            new_left_features.push(*left);
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
            raw_set.insert(candidate_idx);
        }

        let mut raw_ids = vec![];
        let mut matrix_ids = vec![];
        for i in 0..col_size {
            if raw_set.contains(&i) {
                raw_ids.push(i);
            } else {
                matrix_ids.push(i);
            }
        }

        // Creates a MatrixConnector
        let mut right_id_map = vec![];
        let mut left_id_map = vec![];
        let mut right_features_map = HashMap::new();
        let mut left_features_map = HashMap::new();
        for right_features in &right_ids_tmp {
            let mut right_feature_ids = vec![];
            for &raw_id in &matrix_ids {
                right_feature_ids.push(right_features[raw_id]);
            }
            let right_new_id = right_features_map.len();
            let right_id = *right_features_map
                .entry(right_feature_ids)
                .or_insert(right_new_id);
            right_id_map.push(u16::try_from(right_id).unwrap());
        }
        for left_features in &left_ids_tmp {
            let mut left_feature_ids = vec![];
            for &raw_id in &matrix_ids {
                left_feature_ids.push(left_features[raw_id]);
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
                let cost = scorer.accumulate_cost(right_features, left_features);
                let index = *lid * right_features_map.len() + *rid;
                matrix[index] = i16::try_from(cost).unwrap_or(i16::MAX);
            }
        }
        let matrix_connector =
            MatrixConnector::new(matrix, right_features_map.len(), left_features_map.len());

        // Creates a RawConnector from the removed feature templates.
        let mut right_ids = vec![];
        let mut left_ids = vec![];
        for right_features in &right_ids_tmp {
            for &raw_id in &raw_ids {
                right_ids.push(right_features[raw_id]);
            }
        }
        for left_features in &left_ids_tmp {
            for &raw_id in &raw_ids {
                left_ids.push(left_features[raw_id]);
            }
        }
        let right_used_features: HashSet<_> = right_ids.iter().cloned().collect();
        let left_used_features: HashSet<_> = left_ids.iter().cloned().collect();
        for (i, left_map) in scorer_builder.trie.iter_mut().enumerate() {
            if right_used_features.contains(&u32::try_from(i).unwrap()) {
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
        let raw_connector =
            RawConnector::new(right_ids, left_ids, SIMD_SIZE, scorer_builder.build());

        Ok(Self {
            matrix_connector,
            raw_connector,
            right_id_map,
            left_id_map,
        })
    }
}

impl Connector for DualConnector {
    #[inline(always)]
    fn num_left(&self) -> usize {
        self.left_id_map.len()
    }

    #[inline(always)]
    fn num_right(&self) -> usize {
        self.right_id_map.len()
    }

    fn do_mapping(&mut self, mapper: &ConnIdMapper) {
        self.raw_connector.do_mapping(mapper);

        let mut new_left_id_map = vec![0; self.left_id_map.len()];
        let mut new_right_id_map = vec![0; self.right_id_map.len()];
        for id in 0..self.left_id_map.len() {
            let new_id = mapper.left(u16::try_from(id).unwrap());
            new_left_id_map[usize::from(new_id)] = self.left_id_map[id];
        }
        for id in 0..self.right_id_map.len() {
            let new_id = mapper.right(u16::try_from(id).unwrap());
            new_right_id_map[usize::from(new_id)] = self.right_id_map[id];
        }
        let mut matrix_mapper_left = vec![u16::MAX; self.matrix_connector.num_left()];
        let mut matrix_mapper_right = vec![u16::MAX; self.matrix_connector.num_right()];
        let mut left_id = 0;
        let mut right_id = 0;
        for i in &mut new_left_id_map {
            let map = &mut matrix_mapper_left[usize::from(*i)];
            if *map != u16::MAX {
                *i = *map;
                continue;
            }
            *map = left_id;
            *i = left_id;
            left_id += 1;
        }
        for i in &mut new_right_id_map {
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
        self.matrix_connector.do_mapping(&matrix_mapper);
    }
}

impl ConnectorCost for DualConnector {
    #[inline(always)]
    fn cost(&self, right_id: u16, left_id: u16) -> i32 {
        let matrix_right_id = self.right_id_map[usize::from(right_id)];
        let matrix_left_id = self.left_id_map[usize::from(left_id)];
        let matrix_cost = self.matrix_connector.cost(matrix_right_id, matrix_left_id);
        let raw_cost = self.raw_connector.cost(right_id, left_id);
        matrix_cost + raw_cost
    }

    #[inline(always)]
    unsafe fn cost_unchecked(&self, right_id: u16, left_id: u16) -> i32 {
        let matrix_right_id = *self.right_id_map.get_unchecked(usize::from(right_id));
        let matrix_left_id = *self.left_id_map.get_unchecked(usize::from(left_id));
        let matrix_cost = self
            .matrix_connector
            .cost_unchecked(matrix_right_id, matrix_left_id);
        let raw_cost = self.raw_connector.cost_unchecked(right_id, left_id);
        matrix_cost + raw_cost
    }
}
