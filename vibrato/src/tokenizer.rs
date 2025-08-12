//! Viterbi-based tokenizer.
pub(crate) mod lattice;
pub mod worker;

use crate::dictionary::connector::{ConnectorCost, ConnectorWrapper};
use crate::dictionary::Dictionary;
use crate::errors::{Result, VibratoError};
use crate::sentence::Sentence;
use crate::tokenizer::lattice::Lattice;
use crate::tokenizer::worker::Worker;

/// Tokenizer.
pub struct Tokenizer {
    dict: Dictionary,
    // For the MeCab compatibility
    space_cateset: Option<u32>,
    max_grouping_len: Option<usize>,
}

impl Tokenizer {
    /// Creates a new instance.
    ///
    /// # Arguments
    ///
    ///  - `dict`: Dictionary to be used.
    pub const fn new(dict: Dictionary) -> Self {
        Self {
            dict,
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
    ///    The default value is 0, indicating the infinity length.
    pub const fn max_grouping_len(mut self, max_grouping_len: usize) -> Self {
        if max_grouping_len != 0 {
            self.max_grouping_len = Some(max_grouping_len);
        } else {
            self.max_grouping_len = None;
        }
        self
    }

    /// Gets the reference to the dictionary.
    pub const fn dictionary(&self) -> &Dictionary {
        &self.dict
    }

    /// Creates a new worker.
    pub fn new_worker(&self) -> Worker<'_> {
        Worker::new(self)
    }

    pub(crate) fn build_lattice(&self, sent: &Sentence, lattice: &mut Lattice) {
        match self.dict.connector() {
            ConnectorWrapper::Matrix(c) => self.build_lattice_inner(sent, lattice, c),
            ConnectorWrapper::Raw(c) => self.build_lattice_inner(sent, lattice, c),
            ConnectorWrapper::Dual(c) => self.build_lattice_inner(sent, lattice, c),
        }
    }

    fn build_lattice_inner<C>(&self, sent: &Sentence, lattice: &mut Lattice, connector: &C)
    where
        C: ConnectorCost,
    {
        lattice.reset(sent.len_char());

        // These variables indicate the starting character positions of words currently stored
        // in the lattice. If ignore_space() is unset, these always have the same values, and
        // start_node is practically non-functional. If ignore_space() is set, start_node and
        // start_word indicate the starting positions containing and ignoring a space character,
        // respectively. Suppose handle sentence "mens second" at position 4. start_node indicates
        // position 4, and start_word indicates position 5.
        let mut start_node = 0;
        let mut start_word = 0;

        while start_word < sent.len_char() {
            if !lattice.has_previous_node(start_node) {
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
            if start_word == sent.len_char() {
                break;
            }

            self.add_lattice_edges(sent, lattice, start_node, start_word, connector);

            start_word += 1;
            start_node = start_word;
        }

        lattice.insert_eos(start_node, connector);
    }

    fn add_lattice_edges<C>(
        &self,
        sent: &Sentence,
        lattice: &mut Lattice,
        start_node: usize,
        start_word: usize,
        connector: &C,
    ) where
        C: ConnectorCost,
    {
        let mut has_matched = false;

        let suffix = &sent.chars()[start_word..];

        if let Some(user_lexicon) = self.dict.user_lexicon() {
            for m in user_lexicon.common_prefix_iterator(suffix) {
                debug_assert!(start_word + m.end_char <= sent.len_char());
                lattice.insert_node(
                    start_node,
                    start_word,
                    start_word + m.end_char,
                    m.word_idx,
                    m.word_param,
                    connector,
                );
                has_matched = true;
            }
        }

        for m in self.dict.system_lexicon().common_prefix_iterator(suffix) {
            debug_assert!(start_word + m.end_char <= sent.len_char());
            lattice.insert_node(
                start_node,
                start_word,
                start_word + m.end_char,
                m.word_idx,
                m.word_param,
                connector,
            );
            has_matched = true;
        }

        self.dict.unk_handler().gen_unk_words(
            sent,
            start_word,
            has_matched,
            self.max_grouping_len,
            |w| {
                lattice.insert_node(
                    start_node,
                    w.start_char(),
                    w.end_char(),
                    w.word_idx(),
                    w.word_param(),
                    connector,
                );
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::dictionary::SystemDictionaryBuilder;

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

        let dict = SystemDictionaryBuilder::from_readers(
            lexicon_csv.as_bytes(),
            matrix_def.as_bytes(),
            char_def.as_bytes(),
            unk_def.as_bytes(),
        )
        .unwrap();

        let tokenizer = Tokenizer::new(dict);
        let mut worker = tokenizer.new_worker();
        worker.reset_sentence("自然言語処理");
        worker.tokenize();
        assert_eq!(worker.num_tokens(), 2);

        {
            let t = worker.token(0);
            assert_eq!(t.surface(), "自然");
            assert_eq!(t.range_char(), 0..2);
            assert_eq!(t.range_byte(), 0..6);
            assert_eq!(t.feature(), "sizen");
            assert_eq!(t.total_cost(), 1);
        }
        {
            let t = worker.token(1);
            assert_eq!(t.surface(), "言語処理");
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

        let dict = SystemDictionaryBuilder::from_readers(
            lexicon_csv.as_bytes(),
            matrix_def.as_bytes(),
            char_def.as_bytes(),
            unk_def.as_bytes(),
        )
        .unwrap();

        let tokenizer = Tokenizer::new(dict);
        let mut worker = tokenizer.new_worker();
        worker.reset_sentence("自然日本語処理");
        worker.tokenize();
        assert_eq!(worker.num_tokens(), 2);

        {
            let t = worker.token(0);
            assert_eq!(t.surface(), "自然");
            assert_eq!(t.range_char(), 0..2);
            assert_eq!(t.range_byte(), 0..6);
            assert_eq!(t.feature(), "sizen");
            assert_eq!(t.total_cost(), 1);
        }
        {
            let t = worker.token(1);
            assert_eq!(t.surface(), "日本語処理");
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

        let dict = SystemDictionaryBuilder::from_readers(
            lexicon_csv.as_bytes(),
            matrix_def.as_bytes(),
            char_def.as_bytes(),
            unk_def.as_bytes(),
        )
        .unwrap();

        let tokenizer = Tokenizer::new(dict);
        let mut worker = tokenizer.new_worker();
        worker.reset_sentence("不自然言語処理");
        worker.tokenize();
        assert_eq!(worker.num_tokens(), 2);

        {
            let t = worker.token(0);
            assert_eq!(t.surface(), "不自然");
            assert_eq!(t.range_char(), 0..3);
            assert_eq!(t.range_byte(), 0..9);
            assert_eq!(t.feature(), "*");
            assert_eq!(t.total_cost(), 100);
        }
        {
            let t = worker.token(1);
            assert_eq!(t.surface(), "言語処理");
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

        let dict = SystemDictionaryBuilder::from_readers(
            lexicon_csv.as_bytes(),
            matrix_def.as_bytes(),
            char_def.as_bytes(),
            unk_def.as_bytes(),
        )
        .unwrap();

        let tokenizer = Tokenizer::new(dict);
        let mut worker = tokenizer.new_worker();
        worker.reset_sentence("");
        worker.tokenize();
        assert_eq!(worker.num_tokens(), 0);
    }
}
