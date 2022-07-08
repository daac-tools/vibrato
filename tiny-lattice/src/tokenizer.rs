pub mod lattice;

use crate::lexicon::Lexicon;
use crate::matrix::ConnectionMatrix;
use crate::sentence::Sentence;
use lattice::{Lattice, Node};

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

    pub fn do_tokenize(&mut self, input: &str) -> &[Output] {
        self.build_lattice(input);
        self.resolve_best_path();
        &self.output
    }

    fn build_lattice(&mut self, input: &str) {
        self.lattice.reset(self.input.chars().len());
        let input_bytes = input.as_bytes();

        for (char_off, &byte_off) in self.input.char_to_byte_offsets().iter().enumerate() {
            if !self.lattice.has_previous_node(char_off) {
                continue;
            }
            for e in self.lexicon.lookup(input_bytes, byte_off) {
                assert!(e.end_byte < input_bytes.len());
                let end_char = self.input.char_offset(e.end_byte);
                self.lattice
                    .insert(char_off, end_char, e.word_param, &self.matrix);
            }
        }
    }

    fn resolve_best_path(&mut self) {
        self.best_path.clear();
        self.output.clear();

        self.lattice.fill_best_path(&mut self.best_path);
        self.output.resize(self.best_path.len(), Output::default());

        let c2b = self.input.char_to_byte_offsets();
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
