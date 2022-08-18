//! Maintainer of an input sentence and tokenized results.
use crate::dictionary::mapper::{ConnIdCounter, ConnIdProbs};
use crate::dictionary::Dictionary;
use crate::errors::Result;
use crate::sentence::Sentence;
use crate::token::{Token, TokenIter};
use crate::tokenizer::lattice::{Lattice, Node};

/// Maintainer of an input sentence and tokenized results.
///
/// It also holds the internal data structures used in tokenization,
/// which can be reused to avoid unnecessary memory reallocation.
pub struct State<'a> {
    pub(crate) dict: &'a Dictionary,
    pub(crate) sent: Sentence,
    pub(crate) lattice: Lattice,
    pub(crate) top_nodes: Vec<(u16, Node)>,
    pub(crate) counter: Option<ConnIdCounter>,
}

impl<'a> State<'a> {
    /// Creates a new instance.
    pub(crate) fn new(dict: &'a Dictionary) -> Self {
        Self {
            dict,
            sent: Sentence::new(),
            lattice: Lattice::default(),
            top_nodes: vec![],
            counter: None,
        }
    }

    /// Resets the input sentence to be tokenized.
    ///
    /// # Errors
    ///
    /// When the input sentence includes characters more than
    /// [`MAX_SENTENCE_LENGTH`](crate::common::MAX_SENTENCE_LENGTH),
    /// an error will be returned.
    pub fn reset_sentence<S>(&mut self, input: S) -> Result<()>
    where
        S: AsRef<str>,
    {
        self.sent.clear();
        self.top_nodes.clear();
        let input = input.as_ref();
        if !input.is_empty() {
            self.sent.set_sentence(input);
            self.sent.compile(self.dict.char_prop())?;
        }
        Ok(())
    }

    /// Gets the number of resultant tokens.
    #[inline(always)]
    pub fn num_tokens(&self) -> usize {
        self.top_nodes.len()
    }

    /// Gets the `i`-th resultant token.
    #[inline(always)]
    pub fn token(&self, i: usize) -> Token {
        let index = self.num_tokens() - i - 1;
        Token::new(self, index)
    }

    /// Creates an iterator of resultant tokens.
    #[inline(always)]
    pub const fn token_iter(&'a self) -> TokenIter<'a> {
        TokenIter::new(self, 0)
    }

    /// Initializes a counter to train occurrence probabilities of connection ids.
    pub fn init_connid_counter(&mut self) {
        let connector = self.dict.connector();
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

    /// Computes the trained occurrence probabilities of connection ids,
    /// returning those for left- and right-ids.
    ///
    /// # Panics
    ///
    /// It will panic when [`Self::init_connid_counter()`] has never been called.
    pub fn compute_connid_probs(&self) -> (ConnIdProbs, ConnIdProbs) {
        self.counter.as_ref().unwrap().compute_probs()
    }
}
