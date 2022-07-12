use crate::dictionary::*;
use crate::Tokenizer;

const LEX_TEXT: &str = include_str!("./resources/lex.csv");
const MATRIX_TEXT: &str = include_str!("./resources/matrix_10x10.def");
const CATE_TEXT: &str = include_str!("./resources/char.def");

fn make_lexicon() -> Lexicon {
    let entries = lexicon::parser::entries_from_csv(LEX_TEXT.split('\n'));
    Lexicon::from_raw_entries(&entries)
}

fn make_matrix() -> Connector {
    connector::parser::matrix_from_text(MATRIX_TEXT.split('\n'))
}

fn make_category_map() -> CategoryMap {
    CategoryMap::from_lines(CATE_TEXT.split('\n')).unwrap()
}

#[test]
fn test_tokenize_1() {
    let dict = Dictionary::new(make_lexicon(), make_matrix(), make_category_map(), None);
    let mut tok = Tokenizer::new(dict);

    let mut morphs = vec![];
    tok.tokenize("東京都", &mut morphs);

    assert_eq!(morphs.len(), 1);
    assert_eq!(morphs[0].byte_range(), 0..9);
    assert_eq!(morphs[0].char_range(), 0..3);
    assert_eq!(morphs[0].word_id(), 6);

    //   c=0      c=5320       c=0
    //  [BOS] -- [東京都] -- [EOS]
    //     r=0  l=6   r=8  l=0
    let connector = tok.dictionary().connector();
    assert_eq!(connector.cost(0, 6), -79);
    assert_eq!(morphs[0].total_cost(), -79 + 5320);
}

#[test]
fn test_tokenize_2() {
    let dict = Dictionary::new(make_lexicon(), make_matrix(), make_category_map(), None);
    let mut tok = Tokenizer::new(dict);

    let mut morphs = vec![];
    tok.tokenize("京都東京都京都", &mut morphs);

    assert_eq!(morphs.len(), 3);
    assert_eq!(morphs[0].byte_range(), 0..6);
    assert_eq!(morphs[0].char_range(), 0..2);
    assert_eq!(morphs[0].word_id(), 3);
    assert_eq!(morphs[1].byte_range(), 6..15);
    assert_eq!(morphs[1].char_range(), 2..5);
    assert_eq!(morphs[1].word_id(), 6);
    assert_eq!(morphs[2].byte_range(), 15..21);
    assert_eq!(morphs[2].char_range(), 5..7);
    assert_eq!(morphs[2].word_id(), 3);

    //   c=0     c=5293    c=5320    c=5293    c=0
    //  [BOS] -- [京都] -- [東京都] -- [京都] -- [EOS]
    //     r=0  l=6  r=6  l=6  r=8  l=6  r=6  l=0
    let connector = tok.dictionary().connector();
    assert_eq!(connector.cost(0, 6), -79);
    assert_eq!(connector.cost(6, 6), 569);
    assert_eq!(connector.cost(8, 6), -352);
    assert_eq!(morphs[0].total_cost(), -79 + 5293);
    assert_eq!(morphs[1].total_cost(), morphs[0].total_cost() + 569 + 5320);
    assert_eq!(morphs[2].total_cost(), morphs[1].total_cost() - 352 + 5293);
}
