pub mod lattice;

use crate::dictionary::Dictionary;
use crate::sentence::Sentence;
use crate::Morpheme;
use lattice::{EndNode, Lattice};

pub struct Tokenizer {
    dict: Dictionary,
    sent: Sentence,
    lattice: Lattice,
    best_path: Vec<(usize, EndNode)>,
}

impl Tokenizer {
    pub fn new(dict: Dictionary) -> Self {
        Self {
            dict,
            sent: Sentence::default(),
            lattice: Lattice::default(),
            best_path: vec![],
        }
    }

    pub fn tokenize(&mut self, sent: &str, morphs: &mut Vec<Morpheme>) {
        self.sent.set_sentence(sent, self.dict.category_map());
        self.build_lattice(sent);
        self.resolve_best_path(morphs);
    }

    pub fn dict_ref(&self) -> &Dictionary {
        &self.dict
    }

    fn build_lattice(&mut self, sent: &str) {
        self.lattice.reset(self.sent.chars().len());
        let input_bytes = sent.as_bytes();

        for (char_pos, &byte_pos) in self.sent.c2b_offsets().iter().enumerate() {
            if !self.lattice.has_previous_node(char_pos) {
                continue;
            }

            let mut matched = false;
            for m in self
                .dict
                .lexicon()
                .common_prefix_iterator(&input_bytes[byte_pos..])
            {
                assert!(m.end_byte() + byte_pos <= input_bytes.len());
                self.lattice.insert_node(
                    char_pos,
                    self.sent.char_offset(m.end_byte() + byte_pos),
                    m.word_id(),
                    m.word_param(),
                    &self.dict.conn_matrix(),
                );
                matched = true;
            }

            if !matched {
                if let Some(gen) = self.dict.simple_oov_generator() {
                    let oov = gen.gen_oov_word(&self.sent, char_pos);
                    self.lattice.insert_node(
                        char_pos,
                        char_pos + oov.word_len(),
                        oov.word_id(),
                        oov.word_param(),
                        self.dict.conn_matrix(),
                    );
                }
            }
        }
        self.lattice.insert_eos(self.dict.conn_matrix());
    }

    fn resolve_best_path(&mut self, morphs: &mut Vec<Morpheme>) {
        self.best_path.clear();
        self.lattice.fill_best_path(&mut self.best_path);

        morphs.clear();
        morphs.resize(self.best_path.len(), Morpheme::default());

        for (i, (end_pos, end_node)) in self.best_path.iter().rev().enumerate() {
            morphs[i] = Morpheme {
                begin_byte: self.sent.byte_offset(end_node.begin()),
                end_byte: self.sent.byte_offset(*end_pos),
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
    use crate::dictionary::*;

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

        let entries = lexicon::parser::entries_from_csv(lexicon_csv.split('\n'));
        let lexicon = Lexicon::from_raw_entries(&entries);
        let matrix = connection::parser::matrix_from_text(matrix_def.split('\n'));
        let dict = Dictionary::new(lexicon, matrix, CategoryMap::default(), None);

        let mut tokenizer = Tokenizer::new(dict);
        let mut morphs = vec![];
        tokenizer.tokenize("自然言語処理", &mut morphs);

        assert_eq!(
            morphs,
            vec![
                // 自然
                Morpheme {
                    begin_byte: 0,
                    end_byte: 6,
                    begin_char: 0,
                    end_char: 2,
                    word_id: 0,
                    total_cost: 1,
                },
                // 言語処理
                Morpheme {
                    begin_byte: 6,
                    end_byte: 18,
                    begin_char: 2,
                    end_char: 6,
                    word_id: 4,
                    total_cost: 6,
                },
            ]
        );
    }
}
