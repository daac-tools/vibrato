use crate::dictionary::*;
use crate::{Sentence, Tokenizer};

const LEX_TEXT: &str = include_str!("./resources/lex.csv");
const MATRIX_TEXT: &str = include_str!("./resources/matrix_10x10.def");
const CATE_TEXT: &str = include_str!("./resources/char.def");

fn make_lexicon() -> Lexicon {
    Lexicon::from_lines(LEX_TEXT.split('\n'))
}

fn make_connector() -> Connector {
    Connector::from_lines(MATRIX_TEXT.split('\n'))
}

fn make_category_map() -> CategoryMap {
    CategoryMap::from_lines(CATE_TEXT.split('\n')).unwrap()
}

fn make_simple_oov_provider(lex_id: u32) -> SimpleOovProvider {
    SimpleOovProvider::new(
        lex_id,
        WordParam::new(8, 8, 6000),
        "名詞,普通名詞,一般,*,*,*".to_string(),
    )
}

#[test]
fn test_tokenize_tokyo() {
    let dict = Dictionary::new(
        make_lexicon(),
        make_connector(),
        make_category_map(),
        Some(make_simple_oov_provider(1)),
    );

    let mut tokenizer = Tokenizer::new(dict);
    let mut sentence = Sentence::new();

    sentence.set_sentence("東京都");
    tokenizer.tokenize(&mut sentence);

    let morphs = sentence.morphs();
    assert_eq!(morphs.len(), 1);
    assert_eq!(morphs[0].byte_range(), 0..9);
    assert_eq!(morphs[0].char_range(), 0..3);
    assert_eq!(morphs[0].word_idx(), WordIdx::new(0, 6));

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
        make_lexicon(),
        make_connector(),
        make_category_map(),
        Some(make_simple_oov_provider(1)),
    );
    let mut tokenizer = Tokenizer::new(dict);
    let mut sentence = Sentence::new();

    sentence.set_sentence("京都東京都京都");
    tokenizer.tokenize(&mut sentence);

    let morphs = sentence.morphs();
    assert_eq!(morphs.len(), 3);
    assert_eq!(morphs[0].byte_range(), 0..6);
    assert_eq!(morphs[0].char_range(), 0..2);
    assert_eq!(morphs[0].word_idx(), WordIdx::new(0, 3));
    assert_eq!(morphs[1].byte_range(), 6..15);
    assert_eq!(morphs[1].char_range(), 2..5);
    assert_eq!(morphs[1].word_idx(), WordIdx::new(0, 6));
    assert_eq!(morphs[2].byte_range(), 15..21);
    assert_eq!(morphs[2].char_range(), 5..7);
    assert_eq!(morphs[2].word_idx(), WordIdx::new(0, 3));

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
        make_lexicon(),
        make_connector(),
        make_category_map(),
        Some(make_simple_oov_provider(1)),
    );

    let mut tokenizer = Tokenizer::new(dict);
    let mut sentence = Sentence::new();

    sentence.set_sentence("kampersanda");
    tokenizer.tokenize(&mut sentence);

    let morphs = sentence.morphs();
    assert_eq!(morphs.len(), 1);
    assert_eq!(morphs[0].byte_range(), 0..11);
    assert_eq!(morphs[0].char_range(), 0..11);
    assert_eq!(morphs[0].word_idx(), WordIdx::new(1, 0));

    //   c=0        c=6000         c=0
    //  [BOS] -- [kampersanda] -- [EOS]
    //     r=0  l=8         r=8  l=0
    let connector = tokenizer.dictionary().connector();
    assert_eq!(connector.cost(0, 8), 447);
    assert_eq!(morphs[0].total_cost(), 447 + 6000);
}

#[test]
fn test_tokenize_tokyoken() {
    let dict = Dictionary::new(
        make_lexicon(),
        make_connector(),
        make_category_map(),
        Some(make_simple_oov_provider(1)),
    );

    let mut tokenizer = Tokenizer::new(dict);
    let mut sentence = Sentence::new();

    sentence.set_sentence("東京県に行く");
    tokenizer.tokenize(&mut sentence);

    let morphs = sentence.morphs();
    assert_eq!(morphs.len(), 4);
}
