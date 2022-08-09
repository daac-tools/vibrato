use crate::dictionary::lexicon::{LexMatch, Lexicon, WordParam};
use crate::dictionary::word_idx::WordIdx;
use crate::dictionary::LexType;

const LEX_CSV: &str = include_str!("./resources/lex.csv");

#[test]
fn test_common_prefix_iterator_1() {
    let lexicon = Lexicon::from_reader(LEX_CSV.as_bytes(), LexType::System).unwrap();
    let input: Vec<_> = "東京都に行く".chars().collect();
    let mut it = lexicon.common_prefix_iterator(&input);
    // 東
    assert_eq!(
        it.next(),
        Some(LexMatch::new(
            WordIdx::new(LexType::System, 4),
            WordParam::new(7, 7, 4675),
            1
        ))
    );
    // 東京
    assert_eq!(
        it.next(),
        Some(LexMatch::new(
            WordIdx::new(LexType::System, 5),
            WordParam::new(6, 6, 2816),
            2
        ))
    );
    // 東京都
    assert_eq!(
        it.next(),
        Some(LexMatch::new(
            WordIdx::new(LexType::System, 6),
            WordParam::new(6, 8, 5320),
            3
        ))
    );
    assert_eq!(it.next(), None);
}

#[test]
fn test_common_prefix_iterator_2() {
    let lexicon = Lexicon::from_reader(LEX_CSV.as_bytes(), LexType::System).unwrap();
    let mut it = lexicon.common_prefix_iterator(&['X']);
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
    let lexicon = Lexicon::from_reader(LEX_CSV.as_bytes(), LexType::System).unwrap();
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
