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

    #[inline(always)]
    pub fn tokenize(&mut self, sent: &str, morphs: &mut Vec<Morpheme>) {
        self.sent.set_sentence(sent, self.dict.category_map());
        self.build_lattice(sent);
        self.resolve_best_path(morphs);
    }

    #[inline(always)]
    pub fn dictionary(&self) -> &Dictionary {
        &self.dict
    }

    fn build_lattice(&mut self, sent: &str) {
        self.lattice.reset(self.sent.chars().len());
        let input_bytes = sent.as_bytes();

        for (char_pos, &byte_pos) in self.sent.c2b().iter().enumerate() {
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
                    self.sent.char_position(m.end_byte() + byte_pos),
                    m.word_idx(),
                    m.word_param(),
                    &self.dict.connector(),
                );
                matched = true;
            }

            if !matched {
                if let Some(p) = self.dict.simple_oov_provider() {
                    let oov = p.oov_word(&self.sent, char_pos);
                    self.lattice.insert_node(
                        char_pos,
                        char_pos + oov.word_len(),
                        oov.word_idx(),
                        oov.word_param(),
                        self.dict.connector(),
                    );
                }
            }
        }
        self.lattice.insert_eos(self.dict.connector());
    }

    fn resolve_best_path(&mut self, morphs: &mut Vec<Morpheme>) {
        self.best_path.clear();
        self.lattice.fill_best_path(&mut self.best_path);

        morphs.clear();
        morphs.resize(self.best_path.len(), Morpheme::default());

        for (i, (end_pos, end_node)) in self.best_path.iter().rev().enumerate() {
            let end_pos = *end_pos;
            morphs[i] = Morpheme {
                byte_begin: self.sent.byte_position(end_node.begin()) as u16,
                byte_end: self.sent.byte_position(end_pos) as u16,
                char_begin: end_node.begin() as u16,
                char_end: end_pos as u16,
                word_idx: end_node.word_idx(),
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

        let lexicon = Lexicon::from_lines(lexicon_csv.split('\n'));
        let connector = Connector::from_lines(matrix_def.split('\n'));
        let dict = Dictionary::new(lexicon, connector, CategoryMap::default(), None);

        let mut tokenizer = Tokenizer::new(dict);
        let mut morphs = vec![];
        tokenizer.tokenize("自然言語処理", &mut morphs);

        assert_eq!(
            morphs,
            vec![
                // 自然
                Morpheme {
                    byte_begin: 0,
                    byte_end: 6,
                    char_begin: 0,
                    char_end: 2,
                    word_idx: WordIdx::new(0, 0),
                    total_cost: 1,
                },
                // 言語処理
                Morpheme {
                    byte_begin: 6,
                    byte_end: 18,
                    char_begin: 2,
                    char_end: 6,
                    word_idx: WordIdx::new(0, 4),
                    total_cost: 6,
                },
            ]
        );
    }
}
