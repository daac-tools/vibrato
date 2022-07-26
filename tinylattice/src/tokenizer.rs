mod lattice;

use std::cell::RefCell;
use std::rc::Rc;

use crate::dictionary::{ConnIdCounter, Dictionary};
use crate::sentence::Sentence;
use crate::token::Tokens;
use lattice::Lattice;

pub(crate) use lattice::Node;

/// Tokenizer
pub struct Tokenizer<'a> {
    dict: &'a Dictionary,
    sent: Rc<RefCell<Sentence>>,
    lattice: Lattice,
    tokens: Tokens<'a>,
}

impl<'a> Tokenizer<'a> {
    /// Creates an instance of [`Tokenizer`].
    pub fn new(dict: &'a Dictionary) -> Self {
        Self {
            dict,
            sent: Rc::new(RefCell::new(Sentence::new())),
            lattice: Lattice::default(),
            tokens: Tokens::new(dict),
        }
    }

    /// Tokenizes an input text.
    pub fn tokenize<S>(&mut self, input: S) -> &Tokens
    where
        S: AsRef<str>,
    {
        self.tokens.sent = Rc::default();
        self.tokens.nodes.clear();

        let input = input.as_ref();
        if input.is_empty() {
            return &self.tokens;
        }

        self.sent.borrow_mut().set_sentence(input);
        self.sent.borrow_mut().compile(self.dict.char_prop());
        self.build_lattice();

        self.tokens.sent = self.sent.clone();
        self.lattice.append_top_nodes(&mut self.tokens.nodes);

        &self.tokens
    }

    fn build_lattice(&mut self) {
        let sent = self.sent.borrow();
        let input_chars = sent.chars();

        self.lattice.reset(input_chars.len());

        for start_char in 0..input_chars.len() {
            if !self.lattice.has_previous_node(start_char) {
                continue;
            }

            let mut has_matched = false;

            if let Some(user_lexicon) = self.dict.user_lexicon() {
                for m in user_lexicon.common_prefix_iterator(&input_chars[start_char..]) {
                    debug_assert!(start_char + m.end_char() <= input_chars.len());
                    self.lattice.insert_node(
                        start_char,
                        start_char + m.end_char(),
                        m.word_idx(),
                        m.word_param(),
                        self.dict.connector(),
                    );
                    has_matched = true;
                }
            }

            for m in self
                .dict
                .lexicon()
                .common_prefix_iterator(&input_chars[start_char..])
            {
                debug_assert!(start_char + m.end_char() <= input_chars.len());
                self.lattice.insert_node(
                    start_char,
                    start_char + m.end_char(),
                    m.word_idx(),
                    m.word_param(),
                    self.dict.connector(),
                );
                has_matched = true;
            }

            self.dict
                .unk_handler()
                .gen_unk_words(&sent, start_char, has_matched, |w| {
                    self.lattice.insert_node(
                        w.start_char(),
                        w.end_char(),
                        w.word_idx(),
                        w.word_param(),
                        self.dict.connector(),
                    );
                });
        }

        self.lattice.insert_eos(self.dict.connector());
    }

    #[doc(hidden)]
    pub fn add_connid_counts(&self, counter: &mut ConnIdCounter) {
        self.lattice.add_connid_counts(counter);
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use super::*;
    use crate::dictionary::*;

    #[test]
    fn test_tokenize_1() {
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
        let tokens = tokenizer.tokenize("自然言語処理");

        assert_eq!(tokens.len(), 2);

        assert_eq!(tokens.surface(0).deref(), "自然");
        assert_eq!(tokens.range_char(0), 0..2);
        assert_eq!(tokens.range_byte(0), 0..6);
        assert_eq!(tokens.feature(0), "sizen");
        assert_eq!(tokens.total_cost(0), 1);

        assert_eq!(tokens.surface(1).deref(), "言語処理");
        assert_eq!(tokens.range_char(1), 2..6);
        assert_eq!(tokens.range_byte(1), 6..18);
        assert_eq!(tokens.feature(1), "gengoshori");
        assert_eq!(tokens.total_cost(1), 6);
    }

    #[test]
    fn test_tokenize_2() {
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
        let tokens = tokenizer.tokenize("自然日本語処理");

        assert_eq!(tokens.len(), 2);

        assert_eq!(tokens.surface(0).deref(), "自然");
        assert_eq!(tokens.range_char(0), 0..2);
        assert_eq!(tokens.range_byte(0), 0..6);
        assert_eq!(tokens.feature(0), "sizen");
        assert_eq!(tokens.total_cost(0), 1);

        assert_eq!(tokens.surface(1).deref(), "日本語処理");
        assert_eq!(tokens.range_char(1), 2..7);
        assert_eq!(tokens.range_byte(1), 6..21);
        assert_eq!(tokens.feature(1), "*");
        assert_eq!(tokens.total_cost(1), 101);
    }

    #[test]
    fn test_tokenize_3() {
        let lexicon_csv = "自然,0,0,1,sizen
言語,0,0,4,gengo
処理,0,0,3,shori
自然言語,0,0,6,sizengengo
言語処理,0,0,5,gengoshori";
        let matrix_def = "1 1\n0 0 0";
        let char_def = "DEFAULT 0 0 3";
        let unk_def = "DEFAULT,0,0,100,*";

        let dict = Dictionary::new(
            Lexicon::from_reader(lexicon_csv.as_bytes(), LexType::System).unwrap(),
            None,
            Connector::from_reader(matrix_def.as_bytes()).unwrap(),
            CharProperty::from_reader(char_def.as_bytes()).unwrap(),
            UnkHandler::from_reader(unk_def.as_bytes()).unwrap(),
        );

        let mut tokenizer = Tokenizer::new(&dict);
        let tokens = tokenizer.tokenize("不自然言語処理");

        assert_eq!(tokens.len(), 2);

        assert_eq!(tokens.surface(0).deref(), "不自然");
        assert_eq!(tokens.range_char(0), 0..3);
        assert_eq!(tokens.range_byte(0), 0..9);
        assert_eq!(tokens.feature(0), "*");
        assert_eq!(tokens.total_cost(0), 100);

        assert_eq!(tokens.surface(1).deref(), "言語処理");
        assert_eq!(tokens.range_char(1), 3..7);
        assert_eq!(tokens.range_byte(1), 9..21);
        assert_eq!(tokens.feature(1), "gengoshori");
        assert_eq!(tokens.total_cost(1), 105);
    }

    #[test]
    fn test_tokenize_empty() {
        let lexicon_csv = "自然,0,0,1,sizen
言語,0,0,4,gengo
処理,0,0,3,shori
自然言語,0,0,6,sizengengo
言語処理,0,0,5,gengoshori";
        let matrix_def = "1 1\n0 0 0";
        let char_def = "DEFAULT 0 0 3";
        let unk_def = "DEFAULT,0,0,100,*";

        let dict = Dictionary::new(
            Lexicon::from_reader(lexicon_csv.as_bytes(), LexType::System).unwrap(),
            None,
            Connector::from_reader(matrix_def.as_bytes()).unwrap(),
            CharProperty::from_reader(char_def.as_bytes()).unwrap(),
            UnkHandler::from_reader(unk_def.as_bytes()).unwrap(),
        );

        let mut tokenizer = Tokenizer::new(&dict);
        let tokens = tokenizer.tokenize("");

        assert_eq!(tokens.len(), 0);
    }
}
