pub mod lattice;

use crate::lexicon::{Lexicon, WordParam};
use crate::matrix::ConnectionMatrix;
use crate::sentence::Sentence;
use lattice::{Lattice, Node};

const OOV_PARAM: WordParam = WordParam::new(-1, -1, 1000);

pub struct Tokenizer {
    lexicon: Lexicon,
    matrix: ConnectionMatrix,
    input: Sentence,
    lattice: Lattice,
    best_path: Vec<Node>,
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

    pub fn tokenize(&mut self, input: &str) -> &[Output] {
        self.build_lattice(input);
        self.resolve_best_path();
        &self.output
    }

    fn build_lattice(&mut self, input: &str) {
        self.lattice.reset(self.input.chars().len());
        let input_bytes = input.as_bytes();

        for (off_char, &off_byte) in self.input.c2b_offsets().iter().enumerate() {
            if !self.lattice.has_previous_node(off_char) {
                continue;
            }

            let mut found = false;
            for e in self
                .lexicon
                .common_prefix_iterator(&input_bytes[off_byte..])
            {
                assert!(e.end_byte + off_byte < input_bytes.len());
                let end_char = self.input.char_offset(e.end_byte + off_byte);
                self.lattice
                    .insert(off_char, end_char, e.word_param, &self.matrix);
                found = true;
            }

            // oov
            if !found {
                self.lattice
                    .insert(off_char, off_char + 1, OOV_PARAM, &self.matrix);
            }
        }
    }

    fn resolve_best_path(&mut self) {
        self.best_path.clear();
        self.output.clear();

        self.lattice.fill_best_path(&mut self.best_path);
        self.output.resize(self.best_path.len(), Output::default());

        let c2b = self.input.c2b_offsets();
        for (i, node) in self.best_path.iter().rev().enumerate() {
            self.output[i] = Output {
                begin_byte: c2b[i],
                end_byte: c2b[i + 1],
                node: node.clone(),
            };
        }
    }
}

#[derive(Default, Clone)]
pub struct Output {
    begin_byte: usize,
    end_byte: usize,
    node: Node,
}

impl Output {
    pub fn begin_byte(&self) -> usize {
        self.begin_byte
    }

    pub fn end_byte(&self) -> usize {
        self.end_byte
    }

    pub fn node(&self) -> &Node {
        &self.node
    }
}
