use crate::dictionary::{Connector, WordIdx, WordParam};

const MAX_COST: i32 = i32::MAX;
const INVALID_IDX: u16 = u16::MAX;

#[derive(Default)]
pub struct Lattice {
    ends: Vec<Vec<EndNode>>,
    eos: Option<EndNode>,
}

impl Lattice {
    fn reset_vec<T>(data: &mut Vec<Vec<T>>, new_len: usize) {
        for v in data.iter_mut() {
            v.clear();
        }
        let cur_len = data.len();
        if cur_len <= new_len {
            data.reserve(new_len - cur_len);
            for _ in cur_len..new_len {
                data.push(Vec::with_capacity(16))
            }
        }
    }

    // new_len is in characters
    pub fn reset(&mut self, new_len: usize) {
        Self::reset_vec(&mut self.ends, new_len + 1);
        self.eos = None;
        self.insert_bos();
    }

    /// Number of characters of the input sentence.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.ends.len() - 1
    }

    fn insert_bos(&mut self) {
        self.ends[0].push(EndNode {
            word_idx: WordIdx::default(),
            begin: u16::MAX,
            right_id: 0,
            min_idx: INVALID_IDX,
            min_cost: 0,
        });
    }

    pub fn insert_eos(&mut self, connector: &Connector) {
        let (min_idx, min_cost) = self.search_min_node(self.len(), 0, connector).unwrap();
        self.eos = Some(EndNode {
            word_idx: WordIdx::default(),
            begin: self.len() as u16,
            right_id: i16::MAX,
            min_idx,
            min_cost,
        });
    }

    pub fn insert_node(
        &mut self,
        begin: usize,
        end: usize,
        word_idx: WordIdx,
        param: WordParam,
        connector: &Connector,
    ) {
        let (min_idx, min_cost) = self
            .search_min_node(begin, param.left_id as usize, connector)
            .unwrap();
        self.ends[end].push(EndNode {
            word_idx,
            begin: begin as u16,
            right_id: param.right_id,
            min_idx,
            min_cost: min_cost + param.cost as i32,
        });
    }

    fn search_min_node(
        &self,
        start: usize,
        left_id: usize,
        connector: &Connector,
    ) -> Option<(u16, i32)> {
        if self.ends[start].is_empty() {
            return None;
        }
        let mut min_idx = INVALID_IDX;
        let mut min_cost = MAX_COST;
        for (i, l_node) in self.ends[start].iter().enumerate() {
            assert!(l_node.is_connected_to_bos());
            let connect_cost = connector.cost(l_node.right_id(), left_id) as i32;
            let new_cost = l_node.min_cost() + connect_cost;
            if new_cost < min_cost {
                min_idx = i as u16;
                min_cost = new_cost;
            }
        }
        debug_assert_ne!(min_idx, INVALID_IDX);
        Some((min_idx, min_cost))
    }

    /// Checks if there exist at least one at the word end boundary
    #[inline(always)]
    pub fn has_previous_node(&self, i: usize) -> bool {
        self.ends.get(i).map(|d| !d.is_empty()).unwrap_or(false)
    }

    pub fn fill_best_path(&self, result: &mut Vec<(usize, EndNode)>) {
        let mut end_pos = self.len();
        let mut min_idx = self.eos.as_ref().unwrap().min_idx();
        while end_pos != 0 {
            let node = &self.ends[end_pos][min_idx];
            result.push((end_pos, node.clone()));
            (end_pos, min_idx) = (node.begin(), node.min_idx());
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct EndNode {
    word_idx: WordIdx,
    begin: u16,
    right_id: i16,
    min_idx: u16,
    min_cost: i32,
}

impl EndNode {
    #[inline(always)]
    pub fn word_idx(&self) -> WordIdx {
        self.word_idx
    }

    #[inline(always)]
    pub fn begin(&self) -> usize {
        self.begin as usize
    }

    #[inline(always)]
    pub fn right_id(&self) -> usize {
        self.right_id as usize
    }

    #[inline(always)]
    pub fn min_idx(&self) -> usize {
        self.min_idx as usize
    }

    #[inline(always)]
    pub fn min_cost(&self) -> i32 {
        self.min_cost
    }

    #[inline(always)]
    pub fn is_connected_to_bos(&self) -> bool {
        self.min_cost != MAX_COST
    }
}
