use crate::dictionary::connector::ConnectorCost;
use crate::dictionary::lexicon::WordParam;
use crate::dictionary::mapper::ConnIdCounter;
use crate::dictionary::word_idx::WordIdx;
use crate::dictionary::LexType;

use crate::common::{BOS_EOS_CONNECTION_ID, MAX_SENTENCE_LENGTH};

const MAX_COST: i32 = i32::MAX;
const INVALID_IDX: u16 = u16::MAX;

/// 160 bits of each without extra padding.
#[derive(Default, Debug, Clone)]
pub struct Node {
    pub word_id: u32,
    pub lex_type: LexType, // 8 bits
    pub start_node: u16,
    pub start_word: u16,
    pub left_id: u16,
    pub right_id: u16,
    pub min_idx: u16,
    pub min_cost: i32,
}

impl Node {
    #[inline(always)]
    pub const fn word_idx(&self) -> WordIdx {
        WordIdx::new(self.lex_type, self.word_id)
    }

    #[inline(always)]
    pub const fn is_connected_to_bos(&self) -> bool {
        self.min_cost != MAX_COST
    }
}

/// This implementation inspired by sudachi.rs.
#[derive(Default)]
pub struct Lattice {
    ends: Vec<Vec<Node>>,
    eos: Option<Node>,
    len_char: u16, // needed for avoiding to free ends
}

impl Lattice {
    pub fn reset(&mut self, len_char: u16) {
        Self::reset_vec(&mut self.ends, len_char + 1);
        self.len_char = len_char;
        self.eos = None;
        self.insert_bos();
    }

    fn reset_vec<T>(data: &mut Vec<Vec<T>>, new_len: u16) {
        for v in data.iter_mut() {
            v.clear();
        }
        let cur_len = data.len() as u16;
        if cur_len <= new_len {
            data.reserve(usize::from(new_len - cur_len));
            for _ in cur_len..new_len {
                data.push(Vec::with_capacity(16))
            }
        }
    }

    /// Returns the number of characters of the set sentence.
    #[inline(always)]
    pub const fn len_char(&self) -> u16 {
        self.len_char
    }

    fn insert_bos(&mut self) {
        self.ends[0].push(Node {
            word_id: u32::MAX,
            lex_type: LexType::default(),
            start_node: MAX_SENTENCE_LENGTH,
            start_word: MAX_SENTENCE_LENGTH,
            left_id: u16::MAX,
            right_id: BOS_EOS_CONNECTION_ID,
            min_idx: INVALID_IDX,
            min_cost: 0,
        });
    }

    pub fn insert_eos<C>(&mut self, start_node: u16, connector: &C)
    where
        C: ConnectorCost,
    {
        let (min_idx, min_cost) =
            self.search_min_node(start_node, BOS_EOS_CONNECTION_ID, connector);
        self.eos = Some(Node {
            word_id: u32::MAX,
            lex_type: LexType::default(),
            start_node,
            start_word: self.len_char(),
            left_id: BOS_EOS_CONNECTION_ID,
            right_id: u16::MAX,
            min_idx,
            min_cost,
        });
    }

    pub unsafe fn insert_eos_unchecked<C>(&mut self, start_node: u16, connector: &C)
    where
        C: ConnectorCost,
    {
        let (min_idx, min_cost) =
            self.search_min_node_unchecked(start_node, BOS_EOS_CONNECTION_ID, connector);
        self.eos = Some(Node {
            word_id: u32::MAX,
            lex_type: LexType::default(),
            start_node,
            start_word: self.len_char(),
            left_id: BOS_EOS_CONNECTION_ID,
            right_id: u16::MAX,
            min_idx,
            min_cost,
        });
    }

    pub fn insert_node<C>(
        &mut self,
        start_node: u16,
        start_word: u16,
        end_word: u16,
        word_idx: WordIdx,
        word_param: WordParam,
        connector: &C,
    ) where
        C: ConnectorCost,
    {
        debug_assert!(start_node <= start_word);
        debug_assert!(start_word < end_word);
        let (min_idx, min_cost) = self.search_min_node(start_node, word_param.left_id, connector);
        self.ends[usize::from(end_word)].push(Node {
            word_id: word_idx.word_id,
            lex_type: word_idx.lex_type,
            start_node,
            start_word,
            left_id: word_param.left_id,
            right_id: word_param.right_id,
            min_idx,
            min_cost: min_cost + i32::from(word_param.word_cost),
        });
    }

    pub unsafe fn insert_node_unchecked<C>(
        &mut self,
        start_node: u16,
        start_word: u16,
        end_word: u16,
        word_idx: WordIdx,
        word_param: WordParam,
        connector: &C,
    ) where
        C: ConnectorCost,
    {
        debug_assert!(start_node <= start_word);
        debug_assert!(start_word < end_word);
        let (min_idx, min_cost) =
            self.search_min_node_unchecked(start_node, word_param.left_id, connector);
        self.ends[usize::from(end_word)].push(Node {
            word_id: word_idx.word_id,
            lex_type: word_idx.lex_type,
            start_node,
            start_word,
            left_id: word_param.left_id,
            right_id: word_param.right_id,
            min_idx,
            min_cost: min_cost + i32::from(word_param.word_cost),
        });
    }

    fn search_min_node<C>(&self, start_node: u16, left_id: u16, connector: &C) -> (u16, i32)
    where
        C: ConnectorCost,
    {
        debug_assert!(!self.ends[usize::from(start_node)].is_empty());

        let mut min_idx = INVALID_IDX;
        let mut min_cost = MAX_COST;
        for (i, left_node) in self.ends[usize::from(start_node)].iter().enumerate() {
            debug_assert!(left_node.is_connected_to_bos());
            let conn_cost = connector.cost(left_node.right_id, left_id);
            let new_cost = left_node.min_cost + conn_cost;
            // Depending on the order of tie-breaking, the result can be different from MeCab.
            // Using <= (not <) will produce results identical to MeCab in most case (empirically).
            if new_cost <= min_cost {
                min_idx = i as u16;
                min_cost = new_cost;
            }
        }

        debug_assert_ne!(min_idx, INVALID_IDX);
        (min_idx, min_cost)
    }

    unsafe fn search_min_node_unchecked<C>(
        &self,
        start_node: u16,
        left_id: u16,
        connector: &C,
    ) -> (u16, i32)
    where
        C: ConnectorCost,
    {
        debug_assert!(!self.ends[usize::from(start_node)].is_empty());

        let mut min_idx = INVALID_IDX;
        let mut min_cost = MAX_COST;
        for (i, left_node) in self.ends[usize::from(start_node)].iter().enumerate() {
            debug_assert!(left_node.is_connected_to_bos());
            let conn_cost = connector.cost_unchecked(left_node.right_id, left_id);
            let new_cost = left_node.min_cost + conn_cost;
            // Depending on the order of tie-breaking, the result can be different from MeCab.
            // Using <= (not <) will produce results identical to MeCab in most case (empirically).
            if new_cost <= min_cost {
                min_idx = i as u16;
                min_cost = new_cost;
            }
        }

        debug_assert_ne!(min_idx, INVALID_IDX);
        (min_idx, min_cost)
    }

    /// Checks if there exist at least one at the word end boundary
    #[inline(always)]
    pub fn has_previous_node(&self, i: u16) -> bool {
        self.ends
            .get(usize::from(i))
            .map(|d| !d.is_empty())
            .unwrap_or(false)
    }

    pub fn append_top_nodes(&self, top_nodes: &mut Vec<(u16, Node)>) {
        let eos = self.eos.as_ref().unwrap();
        let mut end_node = eos.start_node;
        let mut min_idx = eos.min_idx;
        while end_node != 0 {
            let node = &self.ends[usize::from(end_node)][usize::from(min_idx)];
            top_nodes.push((end_node, node.clone()));
            (end_node, min_idx) = (node.start_node, node.min_idx);
        }
    }

    pub fn add_connid_counts(&self, counter: &mut ConnIdCounter) {
        for end_char in 1..=self.len_char() {
            for r_node in &self.ends[usize::from(end_char)] {
                let start_node = r_node.start_node;
                for l_node in &self.ends[usize::from(start_node)] {
                    counter.add(r_node.left_id, l_node.right_id, 1);
                }
            }
        }
        let r_node = self.eos.as_ref().unwrap();
        for l_node in &self.ends[usize::from(self.len_char())] {
            counter.add(r_node.left_id, l_node.right_id, 1);
        }
    }
}

impl std::fmt::Debug for Lattice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Lattice {{ eos: {:?}, ends: [", &self.eos)?;
        for (i, e) in self.ends[..=usize::from(self.len_char())]
            .iter()
            .enumerate()
        {
            writeln!(f, "{} => {:?}", i, e)?;
        }
        writeln!(f, "]}}")
    }
}
