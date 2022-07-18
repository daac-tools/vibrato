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

    // new_len is in characters
    pub fn reset(&mut self, new_len_char: usize) {
        Self::reset_vec(&mut self.ends, new_len_char + 1);
        self.len_char = new_len_char;
        self.eos = None;
        self.insert_bos();
    }

    /// Number of characters of the input sentence.
    #[inline(always)]
    pub fn len_char(&self) -> usize {
        self.len_char
    }

    fn insert_bos(&mut self) {
        self.ends[0].push(Node {
            word_idx: WordIdx::default(),
            begin_char: u16::MAX,
            right_id: 0,
            min_idx: INVALID_IDX,
            min_cost: 0,
        });
    }

    pub fn insert_eos(&mut self, connector: &Connector) {
        let (min_idx, min_cost) = self.search_min_node(self.len_char(), 0, connector).unwrap();
        self.eos = Some(Node {
            word_idx: WordIdx::default(),
            begin_char: self.len_char() as u16,
            right_id: i16::MAX,
            min_idx,
            min_cost,
        });
    }

    pub fn insert_node(
        &mut self,
        begin_char: usize,
        end_char: usize,
        word_idx: WordIdx,
        word_param: WordParam,
        connector: &Connector,
    ) {
        let (min_idx, min_cost) = self
            .search_min_node(begin_char, word_param.left_id as usize, connector)
            .unwrap();
        self.ends[end_char].push(Node {
            word_idx,
            begin_char: begin_char as u16,
            right_id: word_param.right_id,
            min_idx,
            min_cost: min_cost + word_param.word_cost as i32,
        });
    }

    fn search_min_node(
        &self,
        begin_char: usize,
        left_id: usize,
        connector: &Connector,
    ) -> Option<(u16, i32)> {
        if self.ends[begin_char].is_empty() {
            return None;
        }
        let mut min_idx = INVALID_IDX;
        let mut min_cost = MAX_COST;
        for (i, left_node) in self.ends[begin_char].iter().enumerate() {
            assert!(left_node.is_connected_to_bos());
            let conn_cost = connector.cost(left_node.right_id(), left_id) as i32;
            let new_cost = left_node.min_cost() + conn_cost;
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

    pub fn fill_best_path(&self, result: &mut Vec<(usize, Node)>) {
        let mut pos_char = self.len_char();
        let mut min_idx = self.eos.as_ref().unwrap().min_idx();
        while pos_char != 0 {
            let node = &self.ends[pos_char][min_idx];
            result.push((pos_char, node.clone()));
            (pos_char, min_idx) = (node.begin_char(), node.min_idx());
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

#[derive(Default, Debug, Clone)]
pub struct Node {
    word_idx: WordIdx,
    begin_char: u16,
    right_id: i16,
    min_idx: u16,
    min_cost: i32,
}

impl Node {
    #[inline(always)]
    pub fn word_idx(&self) -> WordIdx {
        self.word_idx
    }

    #[inline(always)]
    pub fn begin_char(&self) -> usize {
        self.begin_char as usize
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
