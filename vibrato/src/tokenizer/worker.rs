//! Provider of a routine for tokenization.
use crate::dictionary::connector::Connector;
use crate::dictionary::mapper::{ConnIdCounter, ConnIdProbs};
use crate::sentence::Sentence;
use crate::token::{Token, TokenIter};
use crate::tokenizer::lattice::{Lattice, Node};
use crate::tokenizer::Tokenizer;

/// Provider of a routine for tokenization.
///
/// It holds the internal data structures used in tokenization,
/// which can be reused to avoid unnecessary memory reallocation.
pub struct Worker<'t> {
    pub(crate) tokenizer: &'t Tokenizer,
    pub(crate) sent: Sentence,
    pub(crate) lattice: Lattice,
    pub(crate) top_nodes: Vec<(usize, Node)>,
    pub(crate) counter: Option<ConnIdCounter>,
}

impl<'t> Worker<'t> {
    /// Creates a new instance.
    pub(crate) fn new(tokenizer: &'t Tokenizer) -> Self {
        Self {
            tokenizer,
            sent: Sentence::new(),
            lattice: Lattice::default(),
            top_nodes: vec![],
            counter: None,
        }
    }

    /// Resets the input sentence to be tokenized.
    pub fn reset_sentence<S>(&mut self, input: S)
    where
        S: AsRef<str>,
    {
        self.sent.clear();
        self.top_nodes.clear();
        let input = input.as_ref();
        if !input.is_empty() {
            self.sent.set_sentence(input);
            self.sent.compile(self.tokenizer.dictionary().char_prop());
        }
    }

    /// Tokenizes the input sentence set in `state`,
    /// returning the result through `state`.
    pub fn tokenize(&mut self) {
        if self.sent.chars().is_empty() {
            return;
        }
        self.tokenizer.build_lattice(&self.sent, &mut self.lattice);
        self.lattice.append_top_nodes(&mut self.top_nodes);
    }

    /// Gets the number of resultant tokens.
    #[inline(always)]
    pub fn num_tokens(&self) -> usize {
        self.top_nodes.len()
    }

    /// Gets the `i`-th resultant token.
    #[inline(always)]
    pub fn token<'w>(&'w self, i: usize) -> Token<'w, 't> {
        let index = self.num_tokens() - i - 1;
        Token::new(self, index)
    }

    /// Creates an iterator of resultant tokens.
    #[inline(always)]
    pub const fn token_iter<'w>(&'w self) -> TokenIter<'w, 't> {
        TokenIter::new(self, 0)
    }

    /// Initializes a counter to compute occurrence probabilities of connection ids.
    pub fn init_connid_counter(&mut self) {
        let connector = self.tokenizer.dictionary().connector();
        self.counter = Some(ConnIdCounter::new(
            connector.num_left(),
            connector.num_right(),
        ));
    }

    /// Updates frequencies of connection ids at the last tokenization.
    ///
    /// # Panics
    ///
    /// It will panic when [`Self::init_connid_counter()`] has never been called.
    pub fn update_connid_counts(&mut self) {
        self.lattice
            .add_connid_counts(self.counter.as_mut().unwrap());
    }

    /// Computes the computed occurrence probabilities of connection ids,
    /// returning those for left- and right-ids.
    ///
    /// # Panics
    ///
    /// It will panic when [`Self::init_connid_counter()`] has never been called.
    pub fn compute_connid_probs(&self) -> (ConnIdProbs, ConnIdProbs) {
        self.counter.as_ref().unwrap().compute_probs()
    }
}
