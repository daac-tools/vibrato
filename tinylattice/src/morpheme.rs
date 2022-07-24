use std::cell::{Ref, RefCell};
use std::ops::Range;
use std::rc::Rc;

use crate::dictionary::Dictionary;
use crate::sentence::Sentence;
use crate::tokenizer::Node;

/// List of resulting morphemes.
pub struct MorphemeList<'a> {
    pub(crate) dict: &'a Dictionary,
    pub(crate) sent: Rc<RefCell<Sentence>>,
    pub(crate) nodes: Vec<(usize, Node)>,
}

impl<'a> MorphemeList<'a> {
    pub(crate) fn new(dict: &'a Dictionary) -> Self {
        Self {
            dict,
            sent: Rc::default(),
            nodes: vec![],
        }
    }

    #[inline(always)]
    fn index(&self, i: usize) -> usize {
        self.nodes.len() - i - 1
    }

    /// Gets the number of morphemes.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Checks if the list is empty.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.nodes.len() == 0
    }

    /// Gets the position range of the `i`-th morpheme in characters.
    #[inline(always)]
    pub fn range_char(&self, i: usize) -> Range<usize> {
        let index = self.index(i);
        let (end_char, node) = &self.nodes[index];
        node.start_char()..*end_char
    }

    /// Gets the position range of the `i`-th morpheme in bytes.
    #[inline(always)]
    pub fn range_byte(&self, i: usize) -> Range<usize> {
        let sent = self.sent.borrow();
        let range_char = self.range_char(i);
        sent.byte_position(range_char.start)..sent.byte_position(range_char.end)
    }

    /// Gets the surface string of the `i`-th morpheme.
    #[inline(always)]
    pub fn surface(&self, i: usize) -> Ref<str> {
        let sent = self.sent.borrow();
        Ref::map(sent, |s| &s.raw()[self.range_byte(i)])
    }

    /// Gets the feature string of the `i`-th morpheme.
    #[inline(always)]
    pub fn feature(&self, i: usize) -> &str {
        let index = self.index(i);
        let (_, node) = &self.nodes[index];
        self.dict.word_feature(node.word_idx())
    }

    /// Gets the total cost of the `i`-th morpheme's node.
    #[inline(always)]
    pub fn total_cost(&self, i: usize) -> i32 {
        let index = self.index(i);
        let (_, node) = &self.nodes[index];
        node.min_cost()
    }
}
