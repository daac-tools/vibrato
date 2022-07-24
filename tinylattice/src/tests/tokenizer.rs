use crate::dictionary::{
    CharProperty, Connector, Dictionary, LexType, Lexicon, UnkHandler, WordIdx,
};
use crate::{Sentence, Tokenizer};

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
    let mut sentence = Sentence::new();

    sentence.set_sentence("東京都");
    tokenizer.tokenize(&mut sentence);

    let morphs = sentence.morphs();
    assert_eq!(morphs.len(), 1);
    assert_eq!(morphs[0].range_byte(), 0..9);
    assert_eq!(morphs[0].range_char(), 0..3);
    assert_eq!(morphs[0].word_idx(), WordIdx::new(LexType::System, 6));

    //   c=0      c=5320       c=0
    //  [BOS] -- [東京都] -- [EOS]
    //     r=0  l=6   r=8  l=0
    let connector = tokenizer.dictionary().connector();
    assert_eq!(connector.cost(0, 6), -79);
    assert_eq!(morphs[0].total_cost(), -79 + 5320);
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
    let mut sentence = Sentence::new();

    sentence.set_sentence("京都東京都京都");
    tokenizer.tokenize(&mut sentence);

    let morphs = sentence.morphs();
    assert_eq!(morphs.len(), 3);
    assert_eq!(morphs[0].range_byte(), 0..6);
    assert_eq!(morphs[0].range_char(), 0..2);
    assert_eq!(morphs[0].word_idx(), WordIdx::new(LexType::System, 3));
    assert_eq!(morphs[1].range_byte(), 6..15);
    assert_eq!(morphs[1].range_char(), 2..5);
    assert_eq!(morphs[1].word_idx(), WordIdx::new(LexType::System, 6));
    assert_eq!(morphs[2].range_byte(), 15..21);
    assert_eq!(morphs[2].range_char(), 5..7);
    assert_eq!(morphs[2].word_idx(), WordIdx::new(LexType::System, 3));

    //   c=0     c=5293    c=5320    c=5293    c=0
    //  [BOS] -- [京都] -- [東京都] -- [京都] -- [EOS]
    //     r=0  l=6  r=6  l=6  r=8  l=6  r=6  l=0
    let connector = tokenizer.dictionary().connector();
    assert_eq!(connector.cost(0, 6), -79);
    assert_eq!(connector.cost(6, 6), 569);
    assert_eq!(connector.cost(8, 6), -352);
    assert_eq!(morphs[0].total_cost(), -79 + 5293);
    assert_eq!(morphs[1].total_cost(), morphs[0].total_cost() + 569 + 5320);
    assert_eq!(morphs[2].total_cost(), morphs[1].total_cost() - 352 + 5293);
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
    let mut sentence = Sentence::new();

    sentence.set_sentence("kampersanda");
    tokenizer.tokenize(&mut sentence);

    let morphs = sentence.morphs();
    assert_eq!(morphs.len(), 1);
    assert_eq!(morphs[0].range_byte(), 0..11);
    assert_eq!(morphs[0].range_char(), 0..11);
    assert_eq!(morphs[0].word_idx(), WordIdx::new(LexType::Unknown, 2));

    //   c=0        c=11633         c=0
    //  [BOS] -- [kampersanda] -- [EOS]
    //     r=0  l=7         r=7  l=0
    let connector = tokenizer.dictionary().connector();
    assert_eq!(connector.cost(0, 7), 887);
    assert_eq!(morphs[0].total_cost(), 887 + 11633);
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
    let mut sentence = Sentence::new();

    sentence.set_sentence("東京県に行く");
    tokenizer.tokenize(&mut sentence);

    let morphs = sentence.morphs();
    assert_eq!(morphs.len(), 4);
}
