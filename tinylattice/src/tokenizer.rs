pub mod lattice;

use crate::dictionary::unknown::UnkWord;
pub use crate::dictionary::Dictionary;
use crate::sentence::Sentence;
use crate::Morpheme;
use lattice::{Lattice, Node};

pub struct Tokenizer {
    dict: Dictionary,
    lattice: Lattice,
    unk_words: Vec<UnkWord>,
    top_nodes: Vec<(usize, Node)>,
}

impl Tokenizer {
    pub fn new(dict: Dictionary) -> Self {
        Self {
            dict,
            lattice: Lattice::default(),
            unk_words: Vec::with_capacity(16),
            top_nodes: vec![],
        }
    }

    #[inline(always)]
    pub fn tokenize(&mut self, sent: &mut Sentence) {
        if sent.raw().is_empty() {
            return;
        }
        sent.compile(self.dict.char_prop());
        self.build_lattice(sent);
        self.resolve_best_path(sent);
    }

    #[inline(always)]
    pub fn feature(&self, morph: &Morpheme) -> &str {
        self.dict.word_feature(morph.word_idx())
    }

    #[inline(always)]
    pub const fn dictionary(&self) -> &Dictionary {
        &self.dict
    }

    #[inline(always)]
    pub const fn lattice(&self) -> &Lattice {
        &self.lattice
    }

    fn build_lattice(&mut self, sent: &Sentence) {
        let input_chars = sent.chars();
        self.lattice.reset(input_chars.len());

        for start_char in 0..input_chars.len() {
            if !self.lattice.has_previous_node(start_char) {
                continue;
            }

            let mut has_matched = false;

            for m in self
                .dict
                .lexicon()
                .common_prefix_iterator(&input_chars[start_char..])
            {
                debug_assert!(start_char + m.end_char() <= input_chars.len());
                self.lattice.insert_node(
                    start_char,
                    start_char + m.end_char(),
                    m.word_idx(),
                    m.word_param(),
                    self.dict.connector(),
                );
                has_matched = true;
            }

            self.unk_words.clear();
            self.dict.unk_handler().gen_unk_words(
                sent,
                start_char,
                has_matched,
                &mut self.unk_words,
            );

            for w in &self.unk_words {
                self.lattice.insert_node(
                    w.start_char(),
                    w.end_char(),
                    w.word_idx(),
                    w.word_param(),
                    self.dict.connector(),
                );
            }
        }

        self.lattice.insert_eos(self.dict.connector());
    }

    fn resolve_best_path(&mut self, sent: &mut Sentence) {
        self.top_nodes.clear();
        self.lattice.fill_best_path(&mut self.top_nodes);

        let mut morphs = sent.take_morphs();

        morphs.clear();
        morphs.resize(self.top_nodes.len(), Morpheme::default());

        for (i, (end_char, node)) in self.top_nodes.iter().rev().enumerate() {
            let end_char = *end_char;
            morphs[i] = Morpheme {
                start_byte: sent.byte_position(node.start_char()) as u16,
                end_byte: sent.byte_position(end_char) as u16,
                start_char: node.start_char() as u16,
                end_char: end_char as u16,
                word_idx: node.word_idx(),
                total_cost: node.min_cost(),
            };
        }

        sent.set_morphs(morphs);
    }

    pub fn new_connid_occ(&self) -> Vec<Vec<usize>> {
        let num_left = self.dict.connector().num_left();
        let num_right = self.dict.connector().num_right();
        vec![vec![0; num_right]; num_left]
    }

    pub fn count_connid_occ(&self, lid_to_rid_occ: &mut [Vec<usize>]) {
        self.lattice.count_connid_occ(lid_to_rid_occ);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::*;

    #[test]
    fn test_tokenize_1() {
        let lexicon_csv = "自然,0,0,1
言語,0,0,4
処理,0,0,3
自然言語,0,0,6
言語処理,0,0,5";
        let matrix_def = "1 1\n0 0 0";
        let char_def = "DEFAULT 0 1 0";
        let unk_def = "DEFAULT,0,0,100,*";

        let dict = Dictionary::new(
            Lexicon::from_reader(lexicon_csv.as_bytes(), LexType::System).unwrap(),
            Connector::from_reader(matrix_def.as_bytes()).unwrap(),
            CharProperty::from_reader(char_def.as_bytes()).unwrap(),
            UnkHandler::from_reader(unk_def.as_bytes()).unwrap(),
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
                    start_byte: 0,
                    end_byte: 6,
                    start_char: 0,
                    end_char: 2,
                    word_idx: WordIdx::new(LexType::System, 0),
                    total_cost: 1,
                },
                // 言語処理
                Morpheme {
                    start_byte: 6,
                    end_byte: 18,
                    start_char: 2,
                    end_char: 6,
                    word_idx: WordIdx::new(LexType::System, 4),
                    total_cost: 6,
                },
            ]
        );
    }

    #[test]
    fn test_tokenize_2() {
        let lexicon_csv = "自然,0,0,1
言語,0,0,4
処理,0,0,3
自然言語,0,0,6
言語処理,0,0,5";
        let matrix_def = "1 1\n0 0 0";
        let char_def = "DEFAULT 0 1 0";
        let unk_def = "DEFAULT,0,0,100,*";

        let dict = Dictionary::new(
            Lexicon::from_reader(lexicon_csv.as_bytes(), LexType::System).unwrap(),
            Connector::from_reader(matrix_def.as_bytes()).unwrap(),
            CharProperty::from_reader(char_def.as_bytes()).unwrap(),
            UnkHandler::from_reader(unk_def.as_bytes()).unwrap(),
        );

        let mut tokenizer = Tokenizer::new(dict);
        let mut sentence = Sentence::new();

        sentence.set_sentence("自然日本語処理");
        tokenizer.tokenize(&mut sentence);

        assert_eq!(
            sentence.morphs(),
            vec![
                // 自然
                Morpheme {
                    start_byte: 0,
                    end_byte: 6,
                    start_char: 0,
                    end_char: 2,
                    word_idx: WordIdx::new(LexType::System, 0),
                    total_cost: 1,
                },
                // 日本語処理
                Morpheme {
                    start_byte: 6,
                    end_byte: 21,
                    start_char: 2,
                    end_char: 7,
                    word_idx: WordIdx::new(LexType::Unknown, 0),
                    total_cost: 101,
                },
            ]
        );
    }

    #[test]
    fn test_tokenize_3() {
        let lexicon_csv = "自然,0,0,1
言語,0,0,4
処理,0,0,3
自然言語,0,0,6
言語処理,0,0,5";
        let matrix_def = "1 1\n0 0 0";
        let char_def = "DEFAULT 0 0 3";
        let unk_def = "DEFAULT,0,0,100,*";

        let dict = Dictionary::new(
            Lexicon::from_reader(lexicon_csv.as_bytes(), LexType::System).unwrap(),
            Connector::from_reader(matrix_def.as_bytes()).unwrap(),
            CharProperty::from_reader(char_def.as_bytes()).unwrap(),
            UnkHandler::from_reader(unk_def.as_bytes()).unwrap(),
        );

        let mut tokenizer = Tokenizer::new(dict);
        let mut sentence = Sentence::new();

        sentence.set_sentence("不自然言語処理");
        tokenizer.tokenize(&mut sentence);

        assert_eq!(
            sentence.morphs(),
            vec![
                // 不自然
                Morpheme {
                    start_byte: 0,
                    end_byte: 9,
                    start_char: 0,
                    end_char: 3,
                    word_idx: WordIdx::new(LexType::Unknown, 0),
                    total_cost: 100,
                },
                // 言語処理
                Morpheme {
                    start_byte: 9,
                    end_byte: 21,
                    start_char: 3,
                    end_char: 7,
                    word_idx: WordIdx::new(LexType::System, 4),
                    total_cost: 105,
                },
            ]
        );
    }
}
