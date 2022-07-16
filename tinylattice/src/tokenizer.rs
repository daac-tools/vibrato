pub mod lattice;

pub use crate::dictionary::Dictionary;
use crate::sentence::Sentence;
use crate::Morpheme;
use lattice::{Lattice, Node};

pub struct Tokenizer {
    dict: Dictionary,
    lattice: Lattice,
    best_path: Vec<(usize, Node)>,
}

impl Tokenizer {
    pub fn new(dict: Dictionary) -> Self {
        Self {
            dict,
            lattice: Lattice::default(),
            best_path: vec![],
        }
    }

    #[inline(always)]
    pub fn tokenize(&mut self, sent: &mut Sentence) {
        sent.compute_bow(self.dict.category_map());
        self.build_lattice(sent);
        self.resolve_best_path(sent);
    }

    #[inline(always)]
    pub fn dictionary(&self) -> &Dictionary {
        &self.dict
    }

    fn build_lattice(&mut self, sent: &Sentence) {
        self.lattice.reset(sent.chars().len());
        let input_bytes = sent.bytes();

        for (pos_char, &pos_byte) in sent.c2b().iter().enumerate() {
            if !self.lattice.has_previous_node(pos_char) {
                continue;
            }

            let mut matched = false;

            for m in self
                .dict
                .lexicon()
                .common_prefix_iterator(&input_bytes[pos_byte..])
            {
                assert!(m.end_byte() + pos_byte <= input_bytes.len());
                self.lattice.insert_node(
                    pos_char,
                    sent.char_position(m.end_byte() + pos_byte),
                    m.word_idx(),
                    m.word_param(),
                    &self.dict.connector(),
                );
                matched = true;
            }

            if !matched {
                for w in self.dict.unk_handler().unk_words(sent, pos_char) {
                    self.lattice.insert_node(
                        w.begin_char(),
                        w.end_char(),
                        w.word_idx(),
                        w.word_param(),
                        self.dict.connector(),
                    );
                }
            }
        }
        self.lattice.insert_eos(self.dict.connector());
    }

    fn resolve_best_path(&mut self, sent: &mut Sentence) {
        self.best_path.clear();
        self.lattice.fill_best_path(&mut self.best_path);

        let mut morphs = sent.take_morphs();

        morphs.clear();
        morphs.resize(self.best_path.len(), Morpheme::default());

        for (i, (end_char, node)) in self.best_path.iter().rev().enumerate() {
            let end_char = *end_char;
            morphs[i] = Morpheme {
                begin_byte: sent.byte_position(node.begin_char()) as u16,
                end_byte: sent.byte_position(end_char) as u16,
                begin_char: node.begin_char() as u16,
                end_char: end_char as u16,
                word_idx: node.word_idx(),
                total_cost: node.min_cost(),
            };
        }

        sent.set_morphs(morphs);
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

        let unk_entry = unknown::UnkEntry {
            cate_type: CategoryTypes::DEFAULT,
            left_id: 0,
            right_id: 0,
            word_cost: 10,
            feature: "".to_string(),
        };

        let dict = Dictionary::new(
            Lexicon::from_lines(lexicon_csv.split('\n'), LexType::System).unwrap(),
            Connector::from_lines(matrix_def.split('\n')).unwrap(),
            CategoryMap::default(),
            SimpleUnkHandler::new(unk_entry),
        );

        let mut tokenizer = Tokenizer::new(dict);
        let mut sentence = Sentence::new();

        sentence.set_sentence("自然言語処理");
        tokenizer.tokenize(&mut sentence);

        assert_eq!(
            sentence.morphs(),
            vec![
                // 自然
                Morpheme {
                    begin_byte: 0,
                    end_byte: 6,
                    begin_char: 0,
                    end_char: 2,
                    word_idx: WordIdx::new(LexType::System, 0),
                    total_cost: 1,
                },
                // 言語処理
                Morpheme {
                    begin_byte: 6,
                    end_byte: 18,
                    begin_char: 2,
                    end_char: 6,
                    word_idx: WordIdx::new(LexType::System, 4),
                    total_cost: 6,
                },
            ]
        );
    }
}
