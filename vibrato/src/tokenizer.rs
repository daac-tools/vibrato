//! Viterbi-based tokenizer.
mod lattice;

use std::cell::RefCell;
use std::rc::Rc;

use crate::dictionary::mapper::ConnIdCounter;
use crate::dictionary::Dictionary;
use crate::errors::{Result, VibratoError};
use crate::sentence::Sentence;
use crate::token::TokenList;
use lattice::Lattice;

pub(crate) use lattice::Node;

use crate::common::MAX_SENTENCE_LENGTH;

/// Tokenizer.
pub struct Tokenizer<'a> {
    dict: &'a Dictionary,
    sent: Rc<RefCell<Sentence>>,
    lattice: Lattice,
    tokens: TokenList<'a>,
    // For the MeCab compatibility
    space_cateset: Option<u32>,
    max_grouping_len: Option<u16>,
}

impl<'a> Tokenizer<'a> {
    /// Creates a new instance.
    ///
    /// # Arguments
    ///
    ///  - `dict`: Dictionary to be used.
    pub fn new(dict: &'a Dictionary) -> Self {
        Self {
            dict,
            sent: Rc::new(RefCell::new(Sentence::new())),
            lattice: Lattice::default(),
            tokens: TokenList::new(dict),
            space_cateset: None,
            max_grouping_len: None,
        }
    }

    /// Ignores spaces from tokens.
    ///
    /// This option is for compatibility with MeCab.
    /// Enable this if you want to obtain the same results as MeCab.
    ///
    /// # Errors
    ///
    /// [`VibratoError`] is returned when category `SPACE` is not defined in the input dictionary.
    pub fn ignore_space(mut self, yes: bool) -> Result<Self> {
        if yes {
            let cate_id = self.dict.char_prop().cate_id("SPACE").ok_or_else(|| {
                VibratoError::invalid_argument(
                    "dict",
                    "SPACE is not defined in the input dictionary (i.e., char.def).",
                )
            })?;
            self.space_cateset = Some(1 << cate_id);
        } else {
            self.space_cateset = None;
        }
        Ok(self)
    }

    /// Specifies the maximum grouping length for unknown words.
    /// By default, the length is infinity.
    ///
    /// This option is for compatibility with MeCab.
    /// Specifies the argument with `24` if you want to obtain the same results as MeCab.
    ///
    /// # Arguments
    ///
    ///  - `max_grouping_len`: The maximum grouping length for unknown words.
    ///                        The default value is 0, indicating the infinity length.
    pub fn max_grouping_len(mut self, max_grouping_len: usize) -> Self {
        if max_grouping_len != 0 && max_grouping_len <= usize::from(MAX_SENTENCE_LENGTH) {
            self.max_grouping_len = Some(max_grouping_len as u16);
        } else {
            self.max_grouping_len = None;
        }
        self
    }

    /// Tokenizes an input text.
    ///
    /// # Errors
    ///
    /// When the input text includes characters more than [`MAX_SENTENCE_LENGTH`],
    /// an error will be returned.
    pub fn tokenize<S>(&mut self, input: S) -> Result<&TokenList>
    where
        S: AsRef<str>,
    {
        self.tokens.sent = Rc::default();
        self.tokens.nodes.clear();

        let input = input.as_ref();
        if input.is_empty() {
            return Ok(&self.tokens);
        }

        self.sent.borrow_mut().set_sentence(input);
        self.sent.borrow_mut().compile(self.dict.char_prop())?;
        self.build_lattice();

        self.tokens.sent = self.sent.clone();
        self.lattice.append_top_nodes(&mut self.tokens.nodes);

        Ok(&self.tokens)
    }

    fn build_lattice(&mut self) {
        let sent = self.sent.borrow();
        let input_chars = sent.chars();
        let input_len = sent.len_char();

        self.lattice.insert_bos(input_len, self.dict.context_ids());

        let mut start_node = 0;
        let mut start_word = 0;

        while start_word < input_len {
            if !self.lattice.has_previous_node(start_node) {
                start_word += 1;
                start_node = start_word;
                continue;
            }

            // on mecab compatible mode
            if let Some(space_cateset) = self.space_cateset {
                let is_space = (sent.char_info(start_node).cate_idset() & space_cateset) != 0;
                start_word += if !is_space {
                    0
                } else {
                    // Skips space characters.
                    sent.groupable(start_node)
                };
            }

            // Does the input end with spaces?
            if start_word == input_len {
                break;
            }

            let mut has_matched = false;

            // Safety: `start_word < input_len` is already checked.
            let suffix = unsafe { input_chars.get_unchecked(usize::from(start_word)..) };

            if let Some(user_lexicon) = self.dict.user_lexicon() {
                for m in user_lexicon.common_prefix_iterator(suffix) {
                    debug_assert!(start_word + m.end_char <= input_len);
                    self.lattice.insert_node(
                        start_node,
                        start_word,
                        start_word + m.end_char,
                        m.word_idx,
                        m.word_param,
                        self.dict.connector(),
                    );
                    has_matched = true;
                }
            }

            for m in self.dict.system_lexicon().common_prefix_iterator(suffix) {
                debug_assert!(start_word + m.end_char <= input_len);
                self.lattice.insert_node(
                    start_node,
                    start_word,
                    start_word + m.end_char,
                    m.word_idx,
                    m.word_param,
                    self.dict.connector(),
                );
                has_matched = true;
            }

            self.dict.unk_handler().gen_unk_words(
                &sent,
                start_word,
                has_matched,
                self.max_grouping_len,
                |w| {
                    self.lattice.insert_node(
                        start_node,
                        w.start_char(),
                        w.end_char(),
                        w.word_idx(),
                        w.word_param(),
                        self.dict.connector(),
                    );
                },
            );

            start_word += 1;
            start_node = start_word;
        }

        self.lattice
            .insert_eos(start_node, self.dict.connector(), self.dict.context_ids());
    }

    /// Creates a counter for frequencies of connection ids to train mappings.
    pub fn new_connid_counter(&self) -> ConnIdCounter {
        let connector = self.dict.connector();
        ConnIdCounter::new(connector.num_left(), connector.num_right())
    }

    /// Adds frequencies of connection ids at the last tokenization.
    pub fn add_connid_counts(&self, counter: &mut ConnIdCounter) {
        self.lattice.add_connid_counts(counter);
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use super::*;

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
        let left_id_def = "0 BOS/EOS";
        let right_id_def = "0 BOS/EOS";

        let dict = Dictionary::from_readers(
            lexicon_csv.as_bytes(),
            matrix_def.as_bytes(),
            char_def.as_bytes(),
            unk_def.as_bytes(),
            left_id_def.as_bytes(),
            right_id_def.as_bytes(),
        )
        .unwrap();

        let mut tokenizer = Tokenizer::new(&dict);
        let tokens = tokenizer.tokenize("自然言語処理").unwrap();

        assert_eq!(tokens.len(), 2);
        {
            let t = tokens.get(0);
            assert_eq!(t.surface().deref(), "自然");
            assert_eq!(t.range_char(), 0..2);
            assert_eq!(t.range_byte(), 0..6);
            assert_eq!(t.feature(), "sizen");
            assert_eq!(t.total_cost(), 1);
        }
        {
            let t = tokens.get(1);
            assert_eq!(t.surface().deref(), "言語処理");
            assert_eq!(t.range_char(), 2..6);
            assert_eq!(t.range_byte(), 6..18);
            assert_eq!(t.feature(), "gengoshori");
            assert_eq!(t.total_cost(), 6);
        }
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
        let left_id_def = "0 BOS/EOS";
        let right_id_def = "0 BOS/EOS";

        let dict = Dictionary::from_readers(
            lexicon_csv.as_bytes(),
            matrix_def.as_bytes(),
            char_def.as_bytes(),
            unk_def.as_bytes(),
            left_id_def.as_bytes(),
            right_id_def.as_bytes(),
        )
        .unwrap();

        let mut tokenizer = Tokenizer::new(&dict);
        let tokens = tokenizer.tokenize("自然日本語処理").unwrap();

        assert_eq!(tokens.len(), 2);
        {
            let t = tokens.get(0);
            assert_eq!(t.surface().deref(), "自然");
            assert_eq!(t.range_char(), 0..2);
            assert_eq!(t.range_byte(), 0..6);
            assert_eq!(t.feature(), "sizen");
            assert_eq!(t.total_cost(), 1);
        }
        {
            let t = tokens.get(1);
            assert_eq!(t.surface().deref(), "日本語処理");
            assert_eq!(t.range_char(), 2..7);
            assert_eq!(t.range_byte(), 6..21);
            assert_eq!(t.feature(), "*");
            assert_eq!(t.total_cost(), 101);
        }
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
        let left_id_def = "0 BOS/EOS";
        let right_id_def = "0 BOS/EOS";

        let dict = Dictionary::from_readers(
            lexicon_csv.as_bytes(),
            matrix_def.as_bytes(),
            char_def.as_bytes(),
            unk_def.as_bytes(),
            left_id_def.as_bytes(),
            right_id_def.as_bytes(),
        )
        .unwrap();

        let mut tokenizer = Tokenizer::new(&dict);
        let tokens = tokenizer.tokenize("不自然言語処理").unwrap();

        assert_eq!(tokens.len(), 2);
        {
            let t = tokens.get(0);
            assert_eq!(t.surface().deref(), "不自然");
            assert_eq!(t.range_char(), 0..3);
            assert_eq!(t.range_byte(), 0..9);
            assert_eq!(t.feature(), "*");
            assert_eq!(t.total_cost(), 100);
        }
        {
            let t = tokens.get(1);
            assert_eq!(t.surface().deref(), "言語処理");
            assert_eq!(t.range_char(), 3..7);
            assert_eq!(t.range_byte(), 9..21);
            assert_eq!(t.feature(), "gengoshori");
            assert_eq!(t.total_cost(), 105);
        }
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
        let left_id_def = "0 BOS/EOS";
        let right_id_def = "0 BOS/EOS";

        let dict = Dictionary::from_readers(
            lexicon_csv.as_bytes(),
            matrix_def.as_bytes(),
            char_def.as_bytes(),
            unk_def.as_bytes(),
            left_id_def.as_bytes(),
            right_id_def.as_bytes(),
        )
        .unwrap();

        let mut tokenizer = Tokenizer::new(&dict);
        let tokens = tokenizer.tokenize("").unwrap();

        assert_eq!(tokens.len(), 0);
    }
}
