//! Tokens
use std::cell::{Ref, RefCell};
use std::ops::Range;
use std::rc::Rc;

use crate::dictionary::{Dictionary, LexType};
use crate::sentence::Sentence;
use crate::tokenizer::Node;

/// List of tokens.
pub struct TokenList<'a> {
    pub(crate) dict: &'a Dictionary,
    pub(crate) sent: Rc<RefCell<Sentence>>,
    pub(crate) nodes: Vec<(usize, Node)>,
}

impl<'a> TokenList<'a> {
    pub(crate) fn new(dict: &'a Dictionary) -> Self {
        Self {
            dict,
            sent: Rc::default(),
            nodes: vec![],
        }
    }

    /// Gets the number of tokens.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Checks if the list is empty.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.nodes.len() == 0
    }

    /// Creates an iterator of tokens.
    #[inline(always)]
    pub const fn iter(&'a self) -> TokenIter<'a> {
        TokenIter { list: self, i: 0 }
    }

    /// Gets the `i`-th token.
    #[inline(always)]
    pub fn get(&self, i: usize) -> Token {
        let index = self.index(i);
        Token { list: self, index }
    }

    #[inline(always)]
    fn index(&self, i: usize) -> usize {
        self.nodes.len() - i - 1
    }
}

/// Token.
pub struct Token<'a> {
    list: &'a TokenList<'a>,
    index: usize,
}

impl<'a> Token<'a> {
    /// Gets the position range of the token in characters.
    #[inline(always)]
    pub fn range_char(&self) -> Range<usize> {
        let (end_word, node) = &self.list.nodes[self.index];
        node.start_word()..*end_word
    }

    /// Gets the position range of the token in bytes.
    #[inline(always)]
    pub fn range_byte(&self) -> Range<usize> {
        let sent = self.list.sent.borrow();
        let range_char = self.range_char();
        sent.byte_position(range_char.start)..sent.byte_position(range_char.end)
    }

    /// Gets the surface string of the token.
    #[inline(always)]
    pub fn surface(&self) -> Ref<str> {
        let sent = self.list.sent.borrow();
        Ref::map(sent, |s| &s.raw()[self.range_byte()])
    }

    /// Gets the feature string of the token.
    #[inline(always)]
    pub fn feature(&self) -> &str {
        let (_, node) = &self.list.nodes[self.index];
        self.list.dict.word_feature(node.word_idx())
    }

    /// Checks if the token is unknown one.
    #[inline(always)]
    pub fn lex_type(&self) -> LexType {
        let (_, node) = &self.list.nodes[self.index];
        node.word_idx().lex_type()
    }

    /// Gets the total cost of the token's node.
    #[inline(always)]
    pub fn total_cost(&self) -> i32 {
        let (_, node) = &self.list.nodes[self.index];
        node.min_cost()
    }
}

/// Iterator of tokens.
pub struct TokenIter<'a> {
    list: &'a TokenList<'a>,
    i: usize,
}

impl<'a> Iterator for TokenIter<'a> {
    type Item = Token<'a>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.i < self.list.len() {
            let t = self.list.get(self.i);
            self.i += 1;
            Some(t)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use crate::dictionary::*;
    use crate::tokenizer::*;

    #[test]
    fn test_iter() {
        let lexicon_csv = "自然,0,0,1,sizen
言語,0,0,4,gengo
処理,0,0,3,shori
自然言語,0,0,6,sizengengo
言語処理,0,0,5,gengoshori";
        let matrix_def = "1 1\n0 0 0";
        let char_def = "DEFAULT 0 1 0";
        let unk_def = "DEFAULT,0,0,100,*";

        let dict = Dictionary::new(
            Lexicon::from_reader(lexicon_csv.as_bytes(), LexType::System).unwrap(),
            None,
            Connector::from_reader(matrix_def.as_bytes()).unwrap(),
            CharProperty::from_reader(char_def.as_bytes()).unwrap(),
            UnkHandler::from_reader(unk_def.as_bytes()).unwrap(),
        );

        let mut tokenizer = Tokenizer::new(&dict);
        let tokens = tokenizer.tokenize("自然言語処理").unwrap();
        assert_eq!(tokens.len(), 2);

        let mut it = tokens.iter();
        for i in 0..tokens.len() {
            let lhs = tokens.get(i);
            let rhs = it.next().unwrap();
            assert_eq!(lhs.surface().deref(), rhs.surface().deref());
        }
        assert!(it.next().is_none());
    }
}
