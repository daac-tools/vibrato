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
    best_path: Vec<(usize, Node)>,
}

impl Tokenizer {
    pub fn new(dict: Dictionary) -> Self {
        Self {
            dict,
            lattice: Lattice::default(),
            unk_words: Vec::with_capacity(16),
            best_path: vec![],
        }
    }

    #[inline(always)]
    pub fn tokenize(&mut self, sent: &mut Sentence) {
        sent.compile(self.dict.char_prop());
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
        let start_positions = &sent.c2b()[..sent.chars().len()];

        for (pos_char, &pos_byte) in start_positions.iter().enumerate() {
            if !self.lattice.has_previous_node(pos_char) {
                continue;
            }

            let mut has_matched = false;

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
                has_matched = true;
            }

            self.unk_words.clear();
            self.dict
                .unk_handler()
                .gen_unk_words(sent, pos_char, has_matched, &mut self.unk_words);

            dbg!(pos_char);
            dbg!(&self.unk_words);

            for w in &self.unk_words {
                self.lattice.insert_node(
                    w.begin_char(),
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
        let lexicon_csv = "自然,0,0,1
言語,0,0,2
処理,0,0,3
自然言語,0,0,4
言語処理,0,0,5";
        let matrix_def = "1 1\n0 0 0";
        let char_def = "DEFAULT 0 1 0";
        let unk_def = "DEFAULT,0,0,100,*";

        let dict = Dictionary::new(
            Lexicon::from_lines(lexicon_csv.split('\n'), LexType::System).unwrap(),
            Connector::from_lines(matrix_def.split('\n')).unwrap(),
            CharProperty::from_lines(char_def.split('\n')).unwrap(),
            UnkHandler::from_lines(unk_def.split('\n')).unwrap(),
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

    #[test]
    fn test_tokenize_2() {
        let lexicon_csv = "自然,0,0,1
言語,0,0,2
処理,0,0,3
自然言語,0,0,4
言語処理,0,0,5";
        let matrix_def = "1 1\n0 0 0";
        let char_def = "DEFAULT 0 1 0";
        let unk_def = "DEFAULT,0,0,100,*";

        let dict = Dictionary::new(
            Lexicon::from_lines(lexicon_csv.split('\n'), LexType::System).unwrap(),
            Connector::from_lines(matrix_def.split('\n')).unwrap(),
            CharProperty::from_lines(char_def.split('\n')).unwrap(),
            UnkHandler::from_lines(unk_def.split('\n')).unwrap(),
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
                    begin_byte: 0,
                    end_byte: 6,
                    begin_char: 0,
                    end_char: 2,
                    word_idx: WordIdx::new(LexType::System, 0),
                    total_cost: 1,
                },
                // 日本語処理
                Morpheme {
                    begin_byte: 6,
                    end_byte: 21,
                    begin_char: 2,
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
言語,0,0,2
処理,0,0,3
自然言語,0,0,4
言語処理,0,0,5";
        let matrix_def = "1 1\n0 0 0";
        let char_def = "DEFAULT 0 0 3";
        let unk_def = "DEFAULT,0,0,100,*";

        let dict = Dictionary::new(
            Lexicon::from_lines(lexicon_csv.split('\n'), LexType::System).unwrap(),
            Connector::from_lines(matrix_def.split('\n')).unwrap(),
            CharProperty::from_lines(char_def.split('\n')).unwrap(),
            UnkHandler::from_lines(unk_def.split('\n')).unwrap(),
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
                    begin_byte: 0,
                    end_byte: 9,
                    begin_char: 0,
                    end_char: 3,
                    word_idx: WordIdx::new(LexType::Unknown, 0),
                    total_cost: 100,
                },
                // 言語処理
                Morpheme {
                    begin_byte: 9,
                    end_byte: 21,
                    begin_char: 3,
                    end_char: 7,
                    word_idx: WordIdx::new(LexType::System, 4),
                    total_cost: 105,
                },
            ]
        );
    }
}
