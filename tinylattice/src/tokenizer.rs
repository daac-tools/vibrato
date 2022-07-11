pub mod lattice;

use crate::lexicon::Lexicon;
use crate::matrix::CostMatrix;
use crate::sentence::Sentence;
use lattice::{EndNode, Lattice};

pub struct Tokenizer {
    lexicon: Lexicon,
    matrix: CostMatrix,
    input: Sentence,
    lattice: Lattice,
    best_path: Vec<(usize, EndNode)>,
    output: Vec<Output>,
}

impl Tokenizer {
    pub fn new(lexicon: Lexicon, matrix: CostMatrix) -> Self {
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

    pub fn lexicon(&self) -> &Lexicon {
        &self.lexicon
    }

    fn build_lattice(&mut self, input: &str) {
        self.lattice.reset(self.input.chars().len());
        let input_bytes = input.as_bytes();

        for (char_offset, &byte_offset) in self.input.c2b_offsets().iter().enumerate() {
            if !self.lattice.has_previous_node(char_offset) {
                continue;
            }
            for m in self
                .lexicon
                .common_prefix_iterator(&input_bytes[byte_offset..])
            {
                assert!(m.end_byte() + byte_offset <= input_bytes.len());
                self.lattice.insert_node(
                    char_offset,
                    self.input.char_offset(m.end_byte() + byte_offset),
                    m.word_id(),
                    m.word_param(),
                    &self.matrix,
                );
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

        for (i, (end_pos, end_node)) in self.best_path.iter().rev().enumerate() {
            self.output[i] = Output {
                begin_byte: self.input.byte_offset(end_node.begin()),
                end_byte: self.input.byte_offset(*end_pos),
                begin_char: end_node.begin(),
                end_char: *end_pos,
                word_id: end_node.word_id(),
                total_cost: end_node.min_cost(),
            };
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct Output {
    begin_byte: usize,
    end_byte: usize,
    begin_char: usize,
    end_char: usize,
    word_id: u32,
    total_cost: i32,
}

impl Output {
    #[inline(always)]
    pub fn begin_byte(&self) -> usize {
        self.begin_byte
    }

    #[inline(always)]
    pub fn end_byte(&self) -> usize {
        self.end_byte
    }

    #[inline(always)]
    pub fn begin_char(&self) -> usize {
        self.begin_char
    }

    #[inline(always)]
    pub fn end_char(&self) -> usize {
        self.end_char
    }

    #[inline(always)]
    pub fn word_id(&self) -> u32 {
        self.word_id
    }

    #[inline(always)]
    pub fn total_cost(&self) -> i32 {
        self.total_cost
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

        let entries = crate::lexicon::parser::entries_from_csv(lexicon_csv.split('\n'));
        let lexicon = Lexicon::from_raw_entries(&entries);
        let matrix = crate::matrix::parser::matrix_from_text(matrix_def.split('\n'));

        let mut tokenizer = Tokenizer::new(lexicon, matrix);
        tokenizer.tokenize("自然言語処理");
        let output = tokenizer.output_ref();

        eprintln!("{:?}", output);
    }
}
