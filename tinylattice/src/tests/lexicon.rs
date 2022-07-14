use crate::dictionary::{lexicon::LexMatch, LexType, Lexicon, WordIdx, WordParam};

const LEX_TEXT: &str = include_str!("./resources/lex.csv");

fn make_lexicon() -> Lexicon {
    Lexicon::from_lines(LEX_TEXT.split('\n'), LexType::System).unwrap()
}

#[test]
fn test_common_prefix_iterator_1() {
    let lexicon = make_lexicon();
    let mut it = lexicon.common_prefix_iterator("東京都に行く".as_bytes());
    // 東
    assert_eq!(
        it.next(),
        Some(LexMatch::new(
            WordIdx::new(LexType::System, 4),
            WordParam::new(7, 7, 4675),
            3
        ))
    );
    // 東京
    assert_eq!(
        it.next(),
        Some(LexMatch::new(
            WordIdx::new(LexType::System, 5),
            WordParam::new(6, 6, 2816),
            6
        ))
    );
    // 東京都
    assert_eq!(
        it.next(),
        Some(LexMatch::new(
            WordIdx::new(LexType::System, 6),
            WordParam::new(6, 8, 5320),
            9
        ))
    );
    assert_eq!(it.next(), None);
}

#[test]
fn test_common_prefix_iterator_2() {
    let lexicon = make_lexicon();
    let mut it = lexicon.common_prefix_iterator("X".as_bytes());
    for word_id in 40..46 {
        assert_eq!(
            it.next(),
            Some(LexMatch::new(
                WordIdx::new(LexType::System, word_id),
                WordParam::new(8, 8, -20000),
                1
            ))
        );
    }
    assert_eq!(it.next(), None);
}

#[test]
fn test_get_word_feature() {
    let lexicon = make_lexicon();
    assert_eq!(
        lexicon.word_feature(WordIdx::new(LexType::System, 0)),
        "た,助動詞,*,*,*,助動詞-タ,終止形-一般,タ,た,*,A,*,*,*,*"
    );
    assert_eq!(
        lexicon.word_feature(WordIdx::new(LexType::System, 2)),
        "に,助詞,格助詞,*,*,*,*,ニ,に,*,A,*,*,*,*"
    );
    assert_eq!(
        lexicon.word_feature(WordIdx::new(LexType::System, 39)),
        " ,空白,*,*,*,*,*, , ,*,A,*,*,*,*"
    );
    assert_eq!(
        lexicon.word_feature(WordIdx::new(LexType::System, 45)),
        "X,名詞,固有名詞,地名,一般,*,*,X,X,*,A,*,*,*,*"
    );
}
