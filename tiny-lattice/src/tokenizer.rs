pub mod lattice;

use crate::connect::ConnectionMatrix;
use crate::lexicon::word_param::WordParam;
use crate::lexicon::Lexicon;
use crate::sentence::Sentence;
use lattice::Lattice;

pub struct Tokenizer {
    lexicon: Lexicon,
    matrix: ConnectionMatrix,
    input: Sentence,
    lattice: Lattice,
}

impl Tokenizer {
    pub fn new(lexicon: Lexicon, matrix: ConnectionMatrix) -> Self {
        Self {
            lexicon,
            matrix,
            input: Sentence::default(),
            lattice: Lattice::default(),
        }
    }

    pub fn do_tokenize(&mut self, input: &str) {}

    fn build_lattice(&mut self) {
        self.lattice.reset(self.input.current_chars().len());
        let input_bytes = self.input.current().as_bytes();

        for (char_off, &byte_off) in self.input.curr_byte_offsets().iter().enumerate() {
            if !self.lattice.has_previous_node(char_off) {
                continue;
            }

            for e in self.lexicon.lookup(input_bytes, byte_off) {
                // do we really need input.can_bow condition?
                if (e.end_byte < input_bytes.len()) && !self.input.can_bow(e.end_byte) {
                    continue;
                }
                let end_char = self.input.ch_idx(e.end_byte);
                self.lattice
                    .insert(char_off, end_char, e.word_param, &self.matrix);
            }
        }
    }
}
