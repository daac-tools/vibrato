pub mod lattice;

use crate::dictionary::Dictionary;
use crate::sentence::Sentence;
use crate::Morpheme;
use lattice::{EndNode, Lattice};

pub struct Tokenizer {
    dict: Dictionary,
    input: Sentence,
    lattice: Lattice,
    best_path: Vec<(usize, EndNode)>,
}

impl Tokenizer {
    pub fn new(dict: Dictionary) -> Self {
        Self {
            dict,
            input: Sentence::default(),
            lattice: Lattice::default(),
            best_path: vec![],
        }
    }

    pub fn tokenize(&mut self, input: &str, output: &mut Vec<Morpheme>) {
        self.input.set_sentence(input, &self.dict.cate_table);
        self.build_lattice(input);
        self.resolve_best_path(output);
    }

    pub fn dict_ref(&self) -> &Dictionary {
        &self.dict
    }

    fn build_lattice(&mut self, input: &str) {
        self.lattice.reset(self.input.chars().len());
        let input_bytes = input.as_bytes();

        for (char_pos, &byte_pos) in self.input.c2b_offsets().iter().enumerate() {
            if !self.lattice.has_previous_node(char_pos) {
                continue;
            }

            let mut matched = false;

            for m in self
                .dict
                .lexicon
                .common_prefix_iterator(&input_bytes[byte_pos..])
            {
                assert!(m.end_byte() + byte_pos <= input_bytes.len());
                self.lattice.insert_node(
                    char_pos,
                    self.input.char_offset(m.end_byte() + byte_pos),
                    m.word_id(),
                    m.word_param(),
                    &self.dict.matrix,
                );
                matched = true;
            }

            if matched {
                let oov = self.dict.oov_generator.gen_oov_word(&self.input, char_pos);
                self.lattice.insert_node(
                    char_pos,
                    char_pos + oov.word_len(),
                    oov.word_id(),
                    oov.word_param(),
                    &self.dict.matrix,
                );
            }
        }
        self.lattice.insert_eos(&self.dict.matrix);
    }

    fn resolve_best_path(&mut self, output: &mut Vec<Morpheme>) {
        self.best_path.clear();
        self.lattice.fill_best_path(&mut self.best_path);

        output.clear();
        output.resize(self.best_path.len(), Morpheme::default());

        for (i, (end_pos, end_node)) in self.best_path.iter().rev().enumerate() {
            output[i] = Morpheme {
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

#[cfg(test)]
mod tests {
    use super::*;

    //     #[test]
    //     fn test_tokenize_1() {
    //         // surface,left_id,right_id,cost
    //         let lexicon_csv = "自然,1,1,1
    // 言語,1,1,2
    // 処理,1,1,3
    // 自然言語,1,1,4
    // 言語処理,1,1,5";

    //         // All costs are zero
    //         let matrix_def = "2 2
    // 0 0 0
    // 0 1 0
    // 1 0 0
    // 1 1 0";

    //         let entries = crate::lexicon::parser::entries_from_csv(lexicon_csv.split('\n'));
    //         let lexicon = Lexicon::from_raw_entries(&entries);
    //         let matrix = crate::matrix::parser::matrix_from_text(matrix_def.split('\n'));

    //         let mut tokenizer = Tokenizer::new(lexicon, matrix);
    //         let mut morphs = vec![];
    //         tokenizer.tokenize("自然言語処理", &mut morphs);

    //         assert_eq!(
    //             morphs,
    //             vec![
    //                 // 自然
    //                 Morpheme {
    //                     begin_byte: 0,
    //                     end_byte: 6,
    //                     begin_char: 0,
    //                     end_char: 2,
    //                     word_id: 0,
    //                     total_cost: 1,
    //                 },
    //                 // 言語処理
    //                 Morpheme {
    //                     begin_byte: 6,
    //                     end_byte: 18,
    //                     begin_char: 2,
    //                     end_char: 6,
    //                     word_id: 4,
    //                     total_cost: 6,
    //                 },
    //             ]
    //         );
    //     }
}
