pub mod lattice;

use crate::lexicon::Lexicon;
use crate::matrix::ConnectionMatrix;
use crate::sentence::Sentence;
use lattice::Lattice;

pub struct Tokenizer {
    lexicon: Lexicon,
    matrix: ConnectionMatrix,
    input: Sentence,
    lattice: Lattice,
    best_path: Vec<(usize, usize)>,
    output: Vec<Output>,
}

impl Tokenizer {
    pub fn new(lexicon: Lexicon, matrix: ConnectionMatrix) -> Self {
        Self {
            lexicon,
            matrix,
            input: Sentence::default(),
            lattice: Lattice::default(),
            best_path: vec![],
            output: vec![],
        }
    }

    pub fn tokenize(&mut self, input: &str) {
        self.input.set_sentence(input);
        self.build_lattice(input);
        self.resolve_best_path();
    }

    pub fn output_ref(&self) -> &[Output] {
        &self.output
    }

    fn build_lattice(&mut self, input: &str) {
        self.lattice.reset(self.input.chars().len());
        let input_bytes = input.as_bytes();

        for (char_offset, &byte_offset) in self.input.c2b_offsets().iter().enumerate() {
            dbg!(char_offset, byte_offset);

            if !self.lattice.has_previous_node(char_offset) {
                continue;
            }

            for e in self
                .lexicon
                .common_prefix_iterator(&input_bytes[byte_offset..])
            {
                dbg!(&e);
                assert!(e.end_byte + byte_offset <= input_bytes.len());
                let end_char = self.input.char_offset(e.end_byte + byte_offset);
                dbg!(end_char);
                self.lattice
                    .insert_node(char_offset, end_char, e.word_param, &self.matrix);
            }

            // TODO: OOV
        }
        self.lattice.insert_eos(&self.matrix);
    }

    fn resolve_best_path(&mut self) {
        self.best_path.clear();
        self.output.clear();

        self.lattice.fill_best_path(&mut self.best_path);
        self.output.resize(self.best_path.len(), Output::default());

        for (i, &(b, e)) in self.best_path.iter().rev().enumerate() {
            self.output[i] = Output {
                begin_byte: self.input.byte_offset(b),
                end_byte: self.input.byte_offset(e),
            };
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct Output {
    begin_byte: usize,
    end_byte: usize,
}

impl Output {
    pub fn begin_byte(&self) -> usize {
        self.begin_byte
    }

    pub fn end_byte(&self) -> usize {
        self.end_byte
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_1() {
        // surface,left_id,right_id,cost
        let lexicon_csv = "自然,1,1,1
言語,1,1,2
処理,1,1,3
自然言語,1,1,4
言語処理,1,1,5";

        // All costs are zero
        let matrix_def = "2 2
0 0 0
0 1 0
1 0 0
1 1 0";

        let lexicon = {
            let mut b = crate::lexicon::builder::LexiconBuilder::new();
            let raw_entries = crate::lexicon::parser::entries_from_csv(lexicon_csv.split('\n'));
            b.extend_from_raw_entries(raw_entries);
            b.build()
        };
        let matrix = crate::matrix::parser::matrix_from_text(matrix_def.split('\n'));

        let mut tokenizer = Tokenizer::new(lexicon, matrix);
        tokenizer.tokenize("自然言語処理");
        let output = tokenizer.output_ref();

        eprintln!("{:?}", output);
    }
}
