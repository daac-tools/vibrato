use crate::connect::ConnectionMatrix;
use crate::lexicon::word_param::WordParam;

#[derive(Default, Clone)]
pub struct Node {
    begin: u16,
    end: u16,
    left_id: i16,
    right_id: i16,
    cost: i16,
    min_idx: u16,
    min_cost: i32,
}

impl Node {
    #[inline(always)]
    fn begin(&self) -> usize {
        self.begin as usize
    }

    #[inline(always)]
    fn end(&self) -> usize {
        self.end as usize
    }

    #[inline(always)]
    fn left_id(&self) -> usize {
        self.left_id as usize
    }

    #[inline(always)]
    fn right_id(&self) -> usize {
        self.right_id as usize
    }

    #[inline(always)]
    fn cost(&self) -> i32 {
        self.cost as i32
    }

    #[inline(always)]
    fn min_idx(&self) -> usize {
        self.min_idx as usize
    }

    #[inline(always)]
    fn min_cost(&self) -> i32 {
        self.min_cost
    }

    #[inline(always)]
    fn is_connected_to_bos(&self) -> bool {
        self.min_cost != i32::MAX
    }
}

#[derive(Default)]
pub struct Lattice {
    ends: Vec<Vec<Node>>,
    // end, min_idx, min_cost
    eos: Option<(usize, usize, i32)>,
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

    pub fn reset(&mut self, new_len: usize) {
        Self::reset_vec(&mut self.ends, new_len + 1);
    }

    fn connect_bos(&mut self) {
        self.ends[0].push(Node {
            begin: 0,
            end: 0,
            left_id: 0,
            right_id: 0,
            cost: 0,
            min_idx: 0,
            min_cost: 0,
        });
    }

    pub fn insert(&mut self, begin: usize, end: usize, param: WordParam, conn: &ConnectionMatrix) {
        let mut node = Node {
            begin: begin as u16,
            end: end as u16,
            left_id: param.left_id,
            right_id: param.right_id,
            cost: param.cost,
            min_idx: 0,
            min_cost: 0,
        };
        let (min_idx, min_cost) = self.connect_node(&node, conn);
        node.min_idx = min_idx;
        node.min_cost = min_cost;
        self.ends[end].push(node);
    }

    /// Find the path with the minimal cost through the lattice to the attached node
    /// Assumption: lattice for all previous boundaries is already constructed
    #[inline]
    pub fn connect_node(&self, r_node: &Node, conn: &ConnectionMatrix) -> (u16, i32) {
        let begin = r_node.begin();

        let node_cost = r_node.cost() as i32;
        let mut min_idx = u16::MAX;
        let mut min_cost = i32::MAX;

        for (i, l_node) in self.ends[begin].iter().enumerate() {
            if !l_node.is_connected_to_bos() {
                continue;
            }
            let connect_cost = conn.cost(l_node.right_id(), r_node.left_id()) as i32;
            let new_cost = l_node.min_cost() + connect_cost + node_cost;
            if new_cost < min_cost {
                min_idx = i as u16;
                min_cost = new_cost;
            }
        }
        (min_idx, min_cost)
    }

    /// Checks if there exist at least one at the word end boundary
    pub fn has_previous_node(&self, i: usize) -> bool {
        self.ends.get(i).map(|d| !d.is_empty()).unwrap_or(false)
    }

    /// Lookup a node for the index
    pub fn node(&self, end: usize, idx: usize) -> &Node {
        &self.ends[end][idx]
    }

    /// Fill the path with the minimum cost (indices only).
    /// **Attention**: the path will be reversed (end to beginning) and will need to be traversed
    /// in the reverse order.
    pub fn fill_best_path(&self, result: &mut Vec<Node>) {
        if self.eos.is_none() {
            return;
        }
        let (mut end, mut min_idx, _) = self.eos.unwrap();
        while end != 0 {
            let node = &self.ends[end][min_idx];
            result.push(node.clone());
            (end, min_idx) = (node.end(), node.min_idx());
        }
    }
}
