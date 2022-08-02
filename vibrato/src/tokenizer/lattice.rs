use crate::dictionary::{lexicon::WordParam, LexType};
use crate::dictionary::{ConnIdCounter, Connector, WordIdx};

const MAX_COST: i32 = i32::MAX;
const INVALID_IDX: u16 = u16::MAX;

/// 160 bits of each
#[derive(Default, Debug, Clone)]
pub struct Node {
    word_id: u32,
    lex_type: LexType, // 8 bits
    start_node: u16,
    start_word: u16,
    left_id: i16,
    right_id: i16,
    min_idx: u16,
    min_cost: i32,
}

impl Node {
    #[inline(always)]
    pub const fn word_idx(&self) -> WordIdx {
        WordIdx::new(self.lex_type, self.word_id)
    }

    #[inline(always)]
    pub const fn start_node(&self) -> usize {
        self.start_node as usize
    }

    #[inline(always)]
    pub const fn start_word(&self) -> usize {
        self.start_word as usize
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

#[derive(Default)]
pub struct Lattice {
    ends: Vec<Vec<Node>>,
    eos: Option<Node>,
    len_char: usize, // needed for avoiding to be free ends
}

impl Lattice {
    pub fn reset(&mut self, new_len_char: usize) {
        Self::reset_vec(&mut self.ends, new_len_char + 1);
        self.len_char = new_len_char;
        self.eos = None;
        self.insert_bos();
    }

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

    /// Returns the number of characters of the set sentence.
    #[inline(always)]
    pub const fn len_char(&self) -> usize {
        self.len_char
    }

    fn insert_bos(&mut self) {
        self.ends[0].push(Node {
            word_id: u32::MAX,
            lex_type: LexType::default(),
            start_node: u16::MAX,
            start_word: u16::MAX,
            left_id: -1,
            right_id: 0,
            min_idx: INVALID_IDX,
            min_cost: 0,
        });
    }

    pub fn insert_eos(&mut self, start_node: usize, connector: &Connector) {
        let (min_idx, min_cost) = self.search_min_node(start_node, 0, connector).unwrap();
        self.eos = Some(Node {
            word_id: u32::MAX,
            lex_type: LexType::default(),
            start_node: start_node as u16,
            start_word: self.len_char() as u16,
            left_id: 0,
            right_id: -1,
            min_idx,
            min_cost,
        });
    }

    pub fn insert_node(
        &mut self,
        start_node: usize,
        start_word: usize,
        end_word: usize,
        word_idx: WordIdx,
        word_param: WordParam,
        connector: &Connector,
    ) {
        debug_assert!(start_node <= start_word);
        debug_assert!(start_word < end_word);

        let (min_idx, min_cost) = self
            .search_min_node(start_node, word_param.left_id as usize, connector)
            .unwrap();

        self.ends[end_word].push(Node {
            word_id: word_idx.word_id(),
            lex_type: word_idx.lex_type(),
            start_node: start_node as u16,
            start_word: start_word as u16,
            left_id: word_param.left_id,
            right_id: word_param.right_id,
            min_idx,
            min_cost: min_cost + word_param.word_cost as i32,
        });
    }

    fn search_min_node(
        &self,
        start_node: usize,
        left_id: usize,
        connector: &Connector,
    ) -> Option<(u16, i32)> {
        if self.ends[start_node].is_empty() {
            return None;
        }

        let mut min_idx = INVALID_IDX;
        let mut min_cost = MAX_COST;

        for (i, left_node) in self.ends[start_node].iter().enumerate() {
            debug_assert!(left_node.is_connected_to_bos());
            let conn_cost = connector.cost(left_node.right_id(), left_id) as i32;
            let new_cost = left_node.min_cost() + conn_cost;

            // Use <= to produce the same tokenization as MeCab
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

    pub fn append_top_nodes(&self, top_nodes: &mut Vec<(usize, Node)>) {
        let eos = self.eos.as_ref().unwrap();
        let mut end_node = eos.start_node();
        let mut min_idx = eos.min_idx();
        while end_node != 0 {
            let node = &self.ends[end_node][min_idx];
            top_nodes.push((end_node, node.clone()));
            (end_node, min_idx) = (node.start_node(), node.min_idx());
        }
    }

    pub fn add_connid_counts(&self, counter: &mut ConnIdCounter) {
        for end_char in 1..=self.len_char() {
            for r_node in &self.ends[end_char] {
                let start_node = r_node.start_node();
                for l_node in &self.ends[start_node] {
                    counter.add(r_node.left_id(), l_node.right_id(), 1);
                }
            }
        }
        let r_node = self.eos.as_ref().unwrap();
        for l_node in &self.ends[self.len_char()] {
            counter.add(r_node.left_id(), l_node.right_id(), 1);
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
