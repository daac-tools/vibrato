use crate::dictionary::{Connector, WordIdx, WordParam};

const MAX_COST: i32 = i32::MAX;
const INVALID_IDX: u16 = u16::MAX;

#[derive(Default)]
pub struct Lattice {
    ends: Vec<Vec<Node>>,
    len_char: usize, // needed for avoiding to be free ends
    eos: Option<Node>,
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

    pub fn reset(&mut self, new_len_char: usize) {
        Self::reset_vec(&mut self.ends, new_len_char + 1);
        self.len_char = new_len_char;
        self.eos = None;
        self.insert_bos();
    }

    /// Number of characters of the input sentence.
    #[inline(always)]
    pub const fn len_char(&self) -> usize {
        self.len_char
    }

    fn insert_bos(&mut self) {
        self.ends[0].push(Node {
            word_idx: WordIdx::default(),
            start_char: u16::MAX,
            left_id: -1,
            right_id: 0,
            min_idx: INVALID_IDX,
            min_cost: 0,
        });
    }

    pub fn insert_eos(&mut self, connector: &Connector) {
        let (min_idx, min_cost) = self.search_min_node(self.len_char(), 0, connector).unwrap();
        self.eos = Some(Node {
            word_idx: WordIdx::default(),
            start_char: self.len_char() as u16,
            left_id: 0,
            right_id: -1,
            min_idx,
            min_cost,
        });
    }

    pub fn insert_node(
        &mut self,
        start_char: usize,
        end_char: usize,
        word_idx: WordIdx,
        word_param: WordParam,
        connector: &Connector,
    ) {
        let (min_idx, min_cost) = self
            .search_min_node(start_char, word_param.left_id as usize, connector)
            .unwrap();
        self.ends[end_char].push(Node {
            word_idx,
            start_char: start_char as u16,
            left_id: word_param.left_id,
            right_id: word_param.right_id,
            min_idx,
            min_cost: min_cost + word_param.word_cost as i32,
        });
    }

    #[allow(unused_variables)] // for exp-ideal
    fn search_min_node(
        &self,
        start_char: usize,
        left_id: usize,
        connector: &Connector,
    ) -> Option<(u16, i32)> {
        if self.ends[start_char].is_empty() {
            return None;
        }
        let mut min_idx = INVALID_IDX;
        let mut min_cost = MAX_COST;
        for (i, left_node) in self.ends[start_char].iter().enumerate() {
            assert!(left_node.is_connected_to_bos());
            #[cfg(feature = "exp-ideal")]
            let conn_cost = 0;
            #[cfg(not(feature = "exp-ideal"))]
            let conn_cost = connector.cost(left_node.right_id(), left_id) as i32;

            let new_cost = left_node.min_cost() + conn_cost;

            // <= allows the same analysis as MeCab
            if new_cost <= min_cost {
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

    pub fn fill_best_path(&self, result: &mut Vec<(usize, Node)>) {
        let mut pos_char = self.len_char();
        let mut min_idx = self.eos.as_ref().unwrap().min_idx();
        while pos_char != 0 {
            let node = &self.ends[pos_char][min_idx];
            result.push((pos_char, node.clone()));
            (pos_char, min_idx) = (node.start_char(), node.min_idx());
        }
    }

    pub fn count_connid_occ(&self, lid_to_rid_occ: &mut [Vec<usize>]) {
        for end_char in 1..=self.len_char() {
            for r_node in &self.ends[end_char] {
                let start_char = r_node.start_char();
                for l_node in &self.ends[start_char] {
                    lid_to_rid_occ[r_node.left_id()][l_node.right_id()] += 1;
                }
            }
        }
        let r_node = self.eos.as_ref().unwrap();
        for l_node in &self.ends[self.len_char()] {
            lid_to_rid_occ[r_node.left_id()][l_node.right_id()] += 1;
        }
    }
}

impl std::fmt::Debug for Lattice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Lattice {{ eos: {:?}, ends: [", &self.eos)?;
        for (i, e) in self.ends[..=self.len_char()].iter().enumerate() {
            writeln!(f, "{} => {:?}", i, e)?;
        }
        writeln!(f, "]}}")
    }
}

// Def: 160 bits
//
#[derive(Default, Debug, Clone)]
pub struct Node {
    word_idx: WordIdx,
    start_char: u16,
    left_id: i16,
    right_id: i16,
    min_idx: u16,
    min_cost: i32,
}

impl Node {
    #[inline(always)]
    pub const fn word_idx(&self) -> WordIdx {
        self.word_idx
    }

    #[inline(always)]
    pub const fn start_char(&self) -> usize {
        self.start_char as usize
    }

    #[inline(always)]
    pub const fn left_id(&self) -> usize {
        self.left_id as usize
    }

    #[inline(always)]
    pub const fn right_id(&self) -> usize {
        self.right_id as usize
    }

    #[inline(always)]
    pub const fn min_idx(&self) -> usize {
        self.min_idx as usize
    }

    #[inline(always)]
    pub const fn min_cost(&self) -> i32 {
        self.min_cost
    }

    #[inline(always)]
    pub const fn is_connected_to_bos(&self) -> bool {
        self.min_cost != MAX_COST
    }
}
