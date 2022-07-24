use std::ops::Deref;

use crate::dictionary::{CharProperty, Connector, Dictionary, LexType, Lexicon, UnkHandler};
use crate::Tokenizer;

const LEX_CSV: &str = include_str!("./resources/lex.csv");
const MATRIX_DEF: &str = include_str!("./resources/matrix.def");
const CHAR_DEF: &str = include_str!("./resources/char.def");
const UNK_DEF: &str = include_str!("./resources/unk.def");

#[test]
fn test_tokenize_tokyo() {
    let dict = Dictionary::new(
        Lexicon::from_reader(LEX_CSV.as_bytes(), LexType::System).unwrap(),
        Connector::from_reader(MATRIX_DEF.as_bytes()).unwrap(),
        CharProperty::from_reader(CHAR_DEF.as_bytes()).unwrap(),
        UnkHandler::from_reader(UNK_DEF.as_bytes()).unwrap(),
    );

    let mut tokenizer = Tokenizer::new(&dict);
    let morphs = tokenizer.tokenize("東京都");

    assert_eq!(morphs.len(), 1);
    assert_eq!(morphs.surface(0).deref(), "東京都");
    assert_eq!(morphs.range_char(0), 0..3);
    assert_eq!(morphs.range_byte(0), 0..9);
    assert_eq!(
        morphs.feature(0),
        "東京都,名詞,固有名詞,地名,一般,*,*,トウキョウト,東京都,*,B,5/9,*,5/9,*"
    );

    //   c=0      c=5320       c=0
    //  [BOS] -- [東京都] -- [EOS]
    //     r=0  l=6   r=8  l=0
    //      c=-79
    assert_eq!(morphs.total_cost(0), -79 + 5320);
}

#[test]
fn test_tokenize_kyotokyo() {
    let dict = Dictionary::new(
        Lexicon::from_reader(LEX_CSV.as_bytes(), LexType::System).unwrap(),
        Connector::from_reader(MATRIX_DEF.as_bytes()).unwrap(),
        CharProperty::from_reader(CHAR_DEF.as_bytes()).unwrap(),
        UnkHandler::from_reader(UNK_DEF.as_bytes()).unwrap(),
    );

    let mut tokenizer = Tokenizer::new(&dict);
    let morphs = tokenizer.tokenize("京都東京都京都");

    assert_eq!(morphs.len(), 3);

    assert_eq!(morphs.surface(0).deref(), "京都");
    assert_eq!(morphs.range_char(0), 0..2);
    assert_eq!(morphs.range_byte(0), 0..6);
    assert_eq!(
        morphs.feature(0),
        "京都,名詞,固有名詞,地名,一般,*,*,キョウト,京都,*,A,*,*,*,1/5"
    );
    assert_eq!(morphs.surface(1).deref(), "東京都");
    assert_eq!(morphs.range_char(1), 2..5);
    assert_eq!(morphs.range_byte(1), 6..15);
    assert_eq!(
        morphs.feature(1),
        "東京都,名詞,固有名詞,地名,一般,*,*,トウキョウト,東京都,*,B,5/9,*,5/9,*"
    );
    assert_eq!(morphs.surface(2).deref(), "京都");
    assert_eq!(morphs.range_char(2), 5..7);
    assert_eq!(morphs.range_byte(2), 15..21);
    assert_eq!(
        morphs.feature(2),
        "京都,名詞,固有名詞,地名,一般,*,*,キョウト,京都,*,A,*,*,*,1/5"
    );

    //   c=0     c=5293    c=5320    c=5293    c=0
    //  [BOS] -- [京都] -- [東京都] -- [京都] -- [EOS]
    //     r=0  l=6  r=6  l=6  r=8  l=6  r=6  l=0
    //      c=-79     c=569     c=-352
    assert_eq!(morphs.total_cost(0), -79 + 5293);
    assert_eq!(morphs.total_cost(1), morphs.total_cost(0) + 569 + 5320);
    assert_eq!(morphs.total_cost(2), morphs.total_cost(1) - 352 + 5293);
}

#[test]
fn test_tokenize_kampersanda() {
    let dict = Dictionary::new(
        Lexicon::from_reader(LEX_CSV.as_bytes(), LexType::System).unwrap(),
        Connector::from_reader(MATRIX_DEF.as_bytes()).unwrap(),
        CharProperty::from_reader(CHAR_DEF.as_bytes()).unwrap(),
        UnkHandler::from_reader(UNK_DEF.as_bytes()).unwrap(),
    );

    let mut tokenizer = Tokenizer::new(&dict);
    let morphs = tokenizer.tokenize("kampersanda");

    assert_eq!(morphs.len(), 1);
    assert_eq!(morphs.surface(0).deref(), "kampersanda");
    assert_eq!(morphs.range_char(0), 0..11);
    assert_eq!(morphs.range_byte(0), 0..11);
    assert_eq!(morphs.feature(0), "名詞,普通名詞,一般,*,*,*");

    //   c=0        c=11633         c=0
    //  [BOS] -- [kampersanda] -- [EOS]
    //     r=0  l=7         r=7  l=0
    //      c=887
    assert_eq!(morphs.total_cost(0), 887 + 11633);
}

#[test]
fn test_tokenize_tokyoken() {
    let dict = Dictionary::new(
        Lexicon::from_reader(LEX_CSV.as_bytes(), LexType::System).unwrap(),
        Connector::from_reader(MATRIX_DEF.as_bytes()).unwrap(),
        CharProperty::from_reader(CHAR_DEF.as_bytes()).unwrap(),
        UnkHandler::from_reader(UNK_DEF.as_bytes()).unwrap(),
    );

    let mut tokenizer = Tokenizer::new(&dict);
    let morphs = tokenizer.tokenize("東京県に行く");

    assert_eq!(morphs.len(), 4);
}

#[test]
fn test_tokenize_empty() {
    let dict = Dictionary::new(
        Lexicon::from_reader(LEX_CSV.as_bytes(), LexType::System).unwrap(),
        Connector::from_reader(MATRIX_DEF.as_bytes()).unwrap(),
        CharProperty::from_reader(CHAR_DEF.as_bytes()).unwrap(),
        UnkHandler::from_reader(UNK_DEF.as_bytes()).unwrap(),
    );

    let mut tokenizer = Tokenizer::new(&dict);
    let morphs = tokenizer.tokenize("");

    assert_eq!(morphs.len(), 0);
}
