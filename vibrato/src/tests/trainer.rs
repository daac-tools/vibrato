use std::io::BufRead;

use crate::trainer::{Corpus, Trainer, TrainerConfig};
use crate::utils;

const TRAIN_LEX_CSV: &[u8] = include_bytes!("./resources/train_lex.csv");
const CHAR_DEF: &[u8] = include_bytes!("./resources/char.def");
const TRAIN_UNK_DEF: &[u8] = include_bytes!("./resources/train_unk.def");
const REWRITE_DEF: &[u8] = include_bytes!("./resources/rewrite.def");
const FEATURE_DEF: &[u8] = include_bytes!("./resources/feature.def");
const CORPUS_TXT: &[u8] = include_bytes!("./resources/corpus.txt");

#[test]
fn test_lexicon_format() {
    let config = TrainerConfig::from_readers(
        TRAIN_LEX_CSV,
        CHAR_DEF,
        TRAIN_UNK_DEF,
        FEATURE_DEF,
        REWRITE_DEF,
    )
    .unwrap();
    let corpus = Corpus::from_reader(CORPUS_TXT).unwrap();
    let trainer = Trainer::new(config).unwrap().max_iter(5);

    let mut lex = vec![];
    let mut matrix = vec![];
    let mut unk = vec![];
    trainer
        .train(corpus, &mut lex, &mut matrix, &mut unk)
        .unwrap();

    // Retrieves the number of right and left connection IDs.
    let (right_len, left_len) = {
        let line = matrix.lines().next().unwrap().unwrap();
        let mut spl = line.split(' ');
        let right_len = spl.next().unwrap().parse::<usize>().unwrap();
        let left_len = spl.next().unwrap().parse::<usize>().unwrap();
        (right_len, left_len)
    };

    let seed_lex_lines: Vec<String> = TRAIN_LEX_CSV.lines().map(|line| line.unwrap()).collect();
    let result_lex_lines: Vec<String> = lex.lines().map(|line| line.unwrap()).collect();

    // Checks the number of lines
    assert_eq!(result_lex_lines.len(), 25);

    // The expected content of the lex file is too long to write here.
    for i in 0..25 {
        let seed_row = utils::parse_csv_row(&seed_lex_lines[i]);
        let result_row = utils::parse_csv_row(&result_lex_lines[i]);

        // surface
        assert_eq!(seed_row[0], result_row[0]);

        // surface
        assert!(result_row[1].parse::<usize>().unwrap() < left_len);
        assert!(result_row[2].parse::<usize>().unwrap() < right_len);
        assert!(result_row[3].parse::<i16>().is_ok());

        // features
        assert_eq!(seed_row[4..], result_row[4..]);
    }
}

#[test]
fn test_unk_format() {
    let config = TrainerConfig::from_readers(
        TRAIN_LEX_CSV,
        CHAR_DEF,
        TRAIN_UNK_DEF,
        FEATURE_DEF,
        REWRITE_DEF,
    )
    .unwrap();
    let corpus = Corpus::from_reader(CORPUS_TXT).unwrap();
    let trainer = Trainer::new(config).unwrap().max_iter(5);

    let mut lex = vec![];
    let mut matrix = vec![];
    let mut unk = vec![];
    trainer
        .train(corpus, &mut lex, &mut matrix, &mut unk)
        .unwrap();

    // Retrieves the number of right and left connection IDs.
    let (right_len, left_len) = {
        let line = matrix.lines().next().unwrap().unwrap();
        let mut spl = line.split(' ');
        let right_len = spl.next().unwrap().parse::<usize>().unwrap();
        let left_len = spl.next().unwrap().parse::<usize>().unwrap();
        (right_len, left_len)
    };

    let result_unk_lines: Vec<String> = unk.lines().map(|line| line.unwrap()).collect();

    // Checks the number of lines
    assert_eq!(result_unk_lines.len(), 4);

    {
        let result_row = utils::parse_csv_row(&result_unk_lines[0]);
        assert_eq!(result_row[0], "DEFAULT");
        assert!(result_row[1].parse::<usize>().unwrap() < left_len);
        assert!(result_row[2].parse::<usize>().unwrap() < right_len);
        assert!(result_row[3].parse::<i16>().is_ok());
        assert_eq!(result_row[4..], ["補助記号", "一般", "*", "*"]);
    }
    // ALPHA is defined earlier than KANJI in unk.def, but KANJI is defined earlier than ALPHA
    // in char.def.
    // The trainer sorts results in the order defined in char.def.
    {
        let result_row = utils::parse_csv_row(&result_unk_lines[1]);
        assert_eq!(result_row[0], "KANJI");
        assert!(result_row[1].parse::<usize>().unwrap() < left_len);
        assert!(result_row[2].parse::<usize>().unwrap() < right_len);
        assert!(result_row[3].parse::<i16>().is_ok());
        assert_eq!(result_row[4..], ["名詞", "普通名詞", "一般", "*"]);
    }
    {
        let result_row = utils::parse_csv_row(&result_unk_lines[2]);
        assert_eq!(result_row[0], "ALPHA");
        assert!(result_row[1].parse::<usize>().unwrap() < left_len);
        assert!(result_row[2].parse::<usize>().unwrap() < right_len);
        assert!(result_row[3].parse::<i16>().is_ok());
        assert_eq!(result_row[4..], ["名詞", "普通名詞", "一般", "*"]);
    }
    {
        let result_row = utils::parse_csv_row(&result_unk_lines[3]);
        assert_eq!(result_row[0], "KANJINUMERIC");
        assert!(result_row[1].parse::<usize>().unwrap() < left_len);
        assert!(result_row[2].parse::<usize>().unwrap() < right_len);
        assert!(result_row[3].parse::<i16>().is_ok());
        assert_eq!(result_row[4..], ["名詞", "数", "*", "*"]);
    }
}

#[test]
fn test_matrix_format() {
    let config = TrainerConfig::from_readers(
        TRAIN_LEX_CSV,
        CHAR_DEF,
        TRAIN_UNK_DEF,
        FEATURE_DEF,
        REWRITE_DEF,
    )
    .unwrap();
    let corpus = Corpus::from_reader(CORPUS_TXT).unwrap();
    let trainer = Trainer::new(config).unwrap().max_iter(5);

    let mut lex = vec![];
    let mut matrix = vec![];
    let mut unk = vec![];
    trainer
        .train(corpus, &mut lex, &mut matrix, &mut unk)
        .unwrap();

    let mut matrix_it = matrix.lines();

    let firstline = matrix_it.next().unwrap().unwrap();

    // Retrieves the number of right and left connection IDs.
    let (right_len, left_len) = {
        let mut spl = firstline.split(' ');
        let right_len = spl.next().unwrap().parse::<usize>().unwrap();
        let left_len = spl.next().unwrap().parse::<usize>().unwrap();
        assert!(spl.next().is_none());
        (right_len, left_len)
    };

    for line in matrix_it {
        let line = line.unwrap();
        let mut spl = line.split(' ');
        // right ID, left ID, weight
        assert!(spl.next().unwrap().parse::<usize>().unwrap() < right_len);
        assert!(spl.next().unwrap().parse::<usize>().unwrap() < left_len);
        assert!(spl.next().unwrap().parse::<i16>().is_ok());
        assert!(spl.next().is_none());
    }
}
