use std::ops::Deref;

use crate::dictionary::{CharProperty, Connector, Dictionary, LexType, Lexicon, UnkHandler};
use crate::Tokenizer;

const LEX_CSV: &str = include_str!("./resources/lex.csv");
const USER_CSV: &str = include_str!("./resources/user.csv");
const MATRIX_DEF: &str = include_str!("./resources/matrix.def");
const CHAR_DEF: &str = include_str!("./resources/char.def");
const UNK_DEF: &str = include_str!("./resources/unk.def");

#[test]
fn test_tokenize_tokyo() {
    let dict = Dictionary::new(
        Lexicon::from_reader(LEX_CSV.as_bytes(), LexType::System).unwrap(),
        None,
        Connector::from_reader(MATRIX_DEF.as_bytes()).unwrap(),
        CharProperty::from_reader(CHAR_DEF.as_bytes()).unwrap(),
        UnkHandler::from_reader(UNK_DEF.as_bytes()).unwrap(),
    );

    let mut tokenizer = Tokenizer::new(&dict);
    let tokens = tokenizer.tokenize("東京都");

    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens.surface(0).deref(), "東京都");
    assert_eq!(tokens.range_char(0), 0..3);
    assert_eq!(tokens.range_byte(0), 0..9);
    assert_eq!(
        tokens.feature(0),
        "東京都,名詞,固有名詞,地名,一般,*,*,トウキョウト,東京都,*,B,5/9,*,5/9,*"
    );

    //   c=0      c=5320       c=0
    //  [BOS] -- [東京都] -- [EOS]
    //     r=0  l=6   r=8  l=0
    //      c=-79
    assert_eq!(tokens.total_cost(0), -79 + 5320);
}

#[test]
fn test_tokenize_kyotokyo() {
    let dict = Dictionary::new(
        Lexicon::from_reader(LEX_CSV.as_bytes(), LexType::System).unwrap(),
        None,
        Connector::from_reader(MATRIX_DEF.as_bytes()).unwrap(),
        CharProperty::from_reader(CHAR_DEF.as_bytes()).unwrap(),
        UnkHandler::from_reader(UNK_DEF.as_bytes()).unwrap(),
    );

    let mut tokenizer = Tokenizer::new(&dict);
    let tokens = tokenizer.tokenize("京都東京都京都");

    assert_eq!(tokens.len(), 3);

    assert_eq!(tokens.surface(0).deref(), "京都");
    assert_eq!(tokens.range_char(0), 0..2);
    assert_eq!(tokens.range_byte(0), 0..6);
    assert_eq!(
        tokens.feature(0),
        "京都,名詞,固有名詞,地名,一般,*,*,キョウト,京都,*,A,*,*,*,1/5"
    );
    assert_eq!(tokens.surface(1).deref(), "東京都");
    assert_eq!(tokens.range_char(1), 2..5);
    assert_eq!(tokens.range_byte(1), 6..15);
    assert_eq!(
        tokens.feature(1),
        "東京都,名詞,固有名詞,地名,一般,*,*,トウキョウト,東京都,*,B,5/9,*,5/9,*"
    );
    assert_eq!(tokens.surface(2).deref(), "京都");
    assert_eq!(tokens.range_char(2), 5..7);
    assert_eq!(tokens.range_byte(2), 15..21);
    assert_eq!(
        tokens.feature(2),
        "京都,名詞,固有名詞,地名,一般,*,*,キョウト,京都,*,A,*,*,*,1/5"
    );

    //   c=0     c=5293    c=5320    c=5293    c=0
    //  [BOS] -- [京都] -- [東京都] -- [京都] -- [EOS]
    //     r=0  l=6  r=6  l=6  r=8  l=6  r=6  l=0
    //      c=-79     c=569     c=-352
    assert_eq!(tokens.total_cost(0), -79 + 5293);
    assert_eq!(tokens.total_cost(1), tokens.total_cost(0) + 569 + 5320);
    assert_eq!(tokens.total_cost(2), tokens.total_cost(1) - 352 + 5293);
}

#[test]
fn test_tokenize_kyotokyo_with_user() {
    let dict = Dictionary::new(
        Lexicon::from_reader(LEX_CSV.as_bytes(), LexType::System).unwrap(),
        Some(Lexicon::from_reader(USER_CSV.as_bytes(), LexType::User).unwrap()),
        Connector::from_reader(MATRIX_DEF.as_bytes()).unwrap(),
        CharProperty::from_reader(CHAR_DEF.as_bytes()).unwrap(),
        UnkHandler::from_reader(UNK_DEF.as_bytes()).unwrap(),
    );

    let mut tokenizer = Tokenizer::new(&dict);
    let tokens = tokenizer.tokenize("京都東京都京都");

    assert_eq!(tokens.len(), 2);

    assert_eq!(tokens.surface(0).deref(), "京都東京都");
    assert_eq!(tokens.range_char(0), 0..5);
    assert_eq!(tokens.range_byte(0), 0..15);
    assert_eq!(tokens.feature(0), "カスタム名詞");

    assert_eq!(tokens.surface(1).deref(), "京都");
    assert_eq!(tokens.range_char(1), 5..7);
    assert_eq!(tokens.range_byte(1), 15..21);
    assert_eq!(
        tokens.feature(1),
        "京都,名詞,固有名詞,地名,一般,*,*,キョウト,京都,*,A,*,*,*,1/5"
    );

    //   c=0      c=-1000      c=5293    c=0
    //  [BOS] -- [京都東京都] -- [京都] -- [EOS]
    //     r=0  l=6      r=8  l=6  r=6  l=0
    //      c=-79         c=-352
    assert_eq!(tokens.total_cost(0), -79 - 1000);
    assert_eq!(tokens.total_cost(1), tokens.total_cost(0) - 352 + 5293);
}

#[test]
fn test_tokenize_tokyoto_with_space() {
    let dict = Dictionary::new(
        Lexicon::from_reader(LEX_CSV.as_bytes(), LexType::System).unwrap(),
        None,
        Connector::from_reader(MATRIX_DEF.as_bytes()).unwrap(),
        CharProperty::from_reader(CHAR_DEF.as_bytes()).unwrap(),
        UnkHandler::from_reader(UNK_DEF.as_bytes()).unwrap(),
    );

    let mut tokenizer = Tokenizer::new(&dict);
    let tokens = tokenizer.tokenize("東京 都");

    assert_eq!(tokens.len(), 3);

    assert_eq!(tokens.surface(0).deref(), "東京");
    assert_eq!(tokens.range_char(0), 0..2);
    assert_eq!(tokens.range_byte(0), 0..6);
    assert_eq!(
        tokens.feature(0),
        "東京,名詞,固有名詞,地名,一般,*,*,トウキョウ,東京,*,A,*,*,*,*"
    );

    assert_eq!(tokens.surface(1).deref(), " ");
    assert_eq!(tokens.range_char(1), 2..3);
    assert_eq!(tokens.range_byte(1), 6..7);
    assert_eq!(tokens.feature(1), " ,空白,*,*,*,*,*, , ,*,A,*,*,*,*");

    assert_eq!(tokens.surface(2).deref(), "都");
    assert_eq!(tokens.range_char(2), 3..4);
    assert_eq!(tokens.range_byte(2), 7..10);
    assert_eq!(
        tokens.feature(2),
        "都,名詞,普通名詞,一般,*,*,*,ト,都,*,A,*,*,*,*"
    );

    //   c=0     c=2816 c=-20000 c=2914   c=0
    //  [BOS] -- [東京] -- [ ] -- [都] -- [EOS]
    //     r=0  l=6 r=6 l=8 r=8 l=8 r=8 l=0
    //      c=-79    c=-390  c=1134  c=-522
    assert_eq!(tokens.total_cost(0), -79 + 2816);
    assert_eq!(tokens.total_cost(1), tokens.total_cost(0) - 390 - 20000);
    assert_eq!(tokens.total_cost(2), tokens.total_cost(1) + 1134 + 2914);
}

#[test]
fn test_tokenize_tokyoto_with_space_ignored() {
    let dict = Dictionary::new(
        Lexicon::from_reader(LEX_CSV.as_bytes(), LexType::System).unwrap(),
        None,
        Connector::from_reader(MATRIX_DEF.as_bytes()).unwrap(),
        CharProperty::from_reader(CHAR_DEF.as_bytes()).unwrap(),
        UnkHandler::from_reader(UNK_DEF.as_bytes()).unwrap(),
    );

    let mut tokenizer = Tokenizer::new(&dict).ignore_space(true);
    let tokens = tokenizer.tokenize("東京 都");

    assert_eq!(tokens.len(), 2);

    assert_eq!(tokens.surface(0).deref(), "東京");
    assert_eq!(tokens.range_char(0), 0..2);
    assert_eq!(tokens.range_byte(0), 0..6);
    assert_eq!(
        tokens.feature(0),
        "東京,名詞,固有名詞,地名,一般,*,*,トウキョウ,東京,*,A,*,*,*,*"
    );

    assert_eq!(tokens.surface(1).deref(), "都");
    assert_eq!(tokens.range_char(1), 3..4);
    assert_eq!(tokens.range_byte(1), 7..10);
    assert_eq!(
        tokens.feature(1),
        "都,名詞,普通名詞,一般,*,*,*,ト,都,*,A,*,*,*,*"
    );

    //   c=0     c=2816   c=2914   c=0
    //  [BOS] -- [東京] -- [都] -- [EOS]
    //     r=0  l=6 r=6  l=8 r=8 l=0
    //      c=-79    c=-390  c=-522
    assert_eq!(tokens.total_cost(0), -79 + 2816);
    assert_eq!(tokens.total_cost(1), tokens.total_cost(0) - 390 + 2914);
}

#[test]
fn test_tokenize_tokyoto_with_spaces_ignored() {
    let dict = Dictionary::new(
        Lexicon::from_reader(LEX_CSV.as_bytes(), LexType::System).unwrap(),
        None,
        Connector::from_reader(MATRIX_DEF.as_bytes()).unwrap(),
        CharProperty::from_reader(CHAR_DEF.as_bytes()).unwrap(),
        UnkHandler::from_reader(UNK_DEF.as_bytes()).unwrap(),
    );

    let mut tokenizer = Tokenizer::new(&dict).ignore_space(true);
    let tokens = tokenizer.tokenize("東京   都");

    assert_eq!(tokens.len(), 2);

    assert_eq!(tokens.surface(0).deref(), "東京");
    assert_eq!(tokens.range_char(0), 0..2);
    assert_eq!(tokens.range_byte(0), 0..6);
    assert_eq!(
        tokens.feature(0),
        "東京,名詞,固有名詞,地名,一般,*,*,トウキョウ,東京,*,A,*,*,*,*"
    );

    assert_eq!(tokens.surface(1).deref(), "都");
    assert_eq!(tokens.range_char(1), 5..6);
    assert_eq!(tokens.range_byte(1), 9..12);
    assert_eq!(
        tokens.feature(1),
        "都,名詞,普通名詞,一般,*,*,*,ト,都,*,A,*,*,*,*"
    );

    //   c=0     c=2816   c=2914   c=0
    //  [BOS] -- [東京] -- [都] -- [EOS]
    //     r=0  l=6 r=6  l=8 r=8 l=0
    //      c=-79    c=-390  c=-522
    assert_eq!(tokens.total_cost(0), -79 + 2816);
    assert_eq!(tokens.total_cost(1), tokens.total_cost(0) - 390 + 2914);
}

#[test]
fn test_tokenize_tokyoto_startswith_spaces_ignored() {
    let dict = Dictionary::new(
        Lexicon::from_reader(LEX_CSV.as_bytes(), LexType::System).unwrap(),
        None,
        Connector::from_reader(MATRIX_DEF.as_bytes()).unwrap(),
        CharProperty::from_reader(CHAR_DEF.as_bytes()).unwrap(),
        UnkHandler::from_reader(UNK_DEF.as_bytes()).unwrap(),
    );

    let mut tokenizer = Tokenizer::new(&dict).ignore_space(true);
    let tokens = tokenizer.tokenize("   東京都");

    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens.surface(0).deref(), "東京都");
    assert_eq!(tokens.range_char(0), 3..6);
    assert_eq!(tokens.range_byte(0), 3..12);
    assert_eq!(
        tokens.feature(0),
        "東京都,名詞,固有名詞,地名,一般,*,*,トウキョウト,東京都,*,B,5/9,*,5/9,*"
    );

    //   c=0      c=5320       c=0
    //  [BOS] -- [東京都] -- [EOS]
    //     r=0  l=6   r=8  l=0
    //      c=-79
    assert_eq!(tokens.total_cost(0), -79 + 5320);
}

#[test]
fn test_tokenize_tokyoto_endswith_spaces_ignored() {
    let dict = Dictionary::new(
        Lexicon::from_reader(LEX_CSV.as_bytes(), LexType::System).unwrap(),
        None,
        Connector::from_reader(MATRIX_DEF.as_bytes()).unwrap(),
        CharProperty::from_reader(CHAR_DEF.as_bytes()).unwrap(),
        UnkHandler::from_reader(UNK_DEF.as_bytes()).unwrap(),
    );

    let mut tokenizer = Tokenizer::new(&dict).ignore_space(true);
    let tokens = tokenizer.tokenize("東京都   ");

    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens.surface(0).deref(), "東京都");
    assert_eq!(tokens.range_char(0), 0..3);
    assert_eq!(tokens.range_byte(0), 0..9);
    assert_eq!(
        tokens.feature(0),
        "東京都,名詞,固有名詞,地名,一般,*,*,トウキョウト,東京都,*,B,5/9,*,5/9,*"
    );

    //   c=0      c=5320       c=0
    //  [BOS] -- [東京都] -- [EOS]
    //     r=0  l=6   r=8  l=0
    //      c=-79
    assert_eq!(tokens.total_cost(0), -79 + 5320);
}

#[test]
fn test_tokenize_kampersanda() {
    let dict = Dictionary::new(
        Lexicon::from_reader(LEX_CSV.as_bytes(), LexType::System).unwrap(),
        None,
        Connector::from_reader(MATRIX_DEF.as_bytes()).unwrap(),
        CharProperty::from_reader(CHAR_DEF.as_bytes()).unwrap(),
        UnkHandler::from_reader(UNK_DEF.as_bytes()).unwrap(),
    );

    let mut tokenizer = Tokenizer::new(&dict);
    let tokens = tokenizer.tokenize("kampersanda");

    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens.surface(0).deref(), "kampersanda");
    assert_eq!(tokens.range_char(0), 0..11);
    assert_eq!(tokens.range_byte(0), 0..11);
    assert_eq!(tokens.feature(0), "名詞,普通名詞,一般,*,*,*");

    //   c=0        c=11633         c=0
    //  [BOS] -- [kampersanda] -- [EOS]
    //     r=0  l=7         r=7  l=0
    //      c=887
    assert_eq!(tokens.total_cost(0), 887 + 11633);
}

#[test]
fn test_tokenize_kampersanda_with_user() {
    let dict = Dictionary::new(
        Lexicon::from_reader(LEX_CSV.as_bytes(), LexType::System).unwrap(),
        Some(Lexicon::from_reader(USER_CSV.as_bytes(), LexType::User).unwrap()),
        Connector::from_reader(MATRIX_DEF.as_bytes()).unwrap(),
        CharProperty::from_reader(CHAR_DEF.as_bytes()).unwrap(),
        UnkHandler::from_reader(UNK_DEF.as_bytes()).unwrap(),
    );

    let mut tokenizer = Tokenizer::new(&dict);
    let tokens = tokenizer.tokenize("kampersanda");

    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens.surface(0).deref(), "kampersanda");
    assert_eq!(tokens.range_char(0), 0..11);
    assert_eq!(tokens.range_byte(0), 0..11);
    assert_eq!(tokens.feature(0), "カスタム名詞");

    //   c=0        c=-2000        c=0
    //  [BOS] -- [kampersanda] -- [EOS]
    //     r=0  l=7         r=7  l=0
    //      c=887
    assert_eq!(tokens.total_cost(0), 887 - 2000);
}

#[test]
fn test_tokenize_kampersanda_with_max_grouping() {
    let dict = Dictionary::new(
        Lexicon::from_reader(LEX_CSV.as_bytes(), LexType::System).unwrap(),
        None,
        Connector::from_reader(MATRIX_DEF.as_bytes()).unwrap(),
        CharProperty::from_reader(CHAR_DEF.as_bytes()).unwrap(),
        UnkHandler::from_reader(UNK_DEF.as_bytes()).unwrap(),
    );

    let mut tokenizer = Tokenizer::new(&dict).ignore_space(true).max_grouping_len(9);
    let tokens = tokenizer.tokenize("kampersanda");

    assert_eq!(tokens.len(), 2);

    assert_eq!(tokens.surface(0).deref(), "k");
    assert_eq!(tokens.range_char(0), 0..1);
    assert_eq!(tokens.range_byte(0), 0..1);
    assert_eq!(tokens.feature(0), "名詞,普通名詞,一般,*,*,*");

    assert_eq!(tokens.surface(1).deref(), "ampersanda");
    assert_eq!(tokens.range_char(1), 1..11);
    assert_eq!(tokens.range_byte(1), 1..11);
    assert_eq!(tokens.feature(1), "名詞,普通名詞,一般,*,*,*");

    //   c=0   c=11633    c=11633        c=0
    //  [BOS] -- [k] -- [ampersanda] -- [EOS]
    //     r=0 l=7 r=7 l=7        r=7  l=0
    //      c=887   c=2341
    assert_eq!(tokens.total_cost(0), 887 + 11633);
    assert_eq!(tokens.total_cost(1), tokens.total_cost(0) + 2341 + 11633);
}

#[test]
fn test_tokenize_tokyoken() {
    let dict = Dictionary::new(
        Lexicon::from_reader(LEX_CSV.as_bytes(), LexType::System).unwrap(),
        None,
        Connector::from_reader(MATRIX_DEF.as_bytes()).unwrap(),
        CharProperty::from_reader(CHAR_DEF.as_bytes()).unwrap(),
        UnkHandler::from_reader(UNK_DEF.as_bytes()).unwrap(),
    );

    let mut tokenizer = Tokenizer::new(&dict);
    let tokens = tokenizer.tokenize("東京県に行く");

    assert_eq!(tokens.len(), 4);
}

/// This test is to check if the category order in char.def is preserved.
#[test]
fn test_tokenize_kanjinumeric() {
    let dict = Dictionary::new(
        Lexicon::from_reader(LEX_CSV.as_bytes(), LexType::System).unwrap(),
        None,
        Connector::from_reader(MATRIX_DEF.as_bytes()).unwrap(),
        CharProperty::from_reader(CHAR_DEF.as_bytes()).unwrap(),
        UnkHandler::from_reader(UNK_DEF.as_bytes()).unwrap(),
    );

    let mut tokenizer = Tokenizer::new(&dict);
    let tokens = tokenizer.tokenize("一橋大学大学院");

    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens.surface(0).deref(), "一橋大学大学院");
    assert_eq!(tokens.range_char(0), 0..7);
    assert_eq!(tokens.range_byte(0), 0..21);
    assert_eq!(tokens.feature(0), "名詞,数,*,*,*,*,*");
}

#[test]
fn test_tokenize_empty() {
    let dict = Dictionary::new(
        Lexicon::from_reader(LEX_CSV.as_bytes(), LexType::System).unwrap(),
        None,
        Connector::from_reader(MATRIX_DEF.as_bytes()).unwrap(),
        CharProperty::from_reader(CHAR_DEF.as_bytes()).unwrap(),
        UnkHandler::from_reader(UNK_DEF.as_bytes()).unwrap(),
    );

    let mut tokenizer = Tokenizer::new(&dict);
    let tokens = tokenizer.tokenize("");

    assert_eq!(tokens.len(), 0);
}
