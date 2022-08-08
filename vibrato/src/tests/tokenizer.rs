use std::ops::Deref;

use crate::dictionary::Dictionary;
use crate::Tokenizer;

const LEX_CSV: &str = include_str!("./resources/lex.csv");
const USER_CSV: &str = include_str!("./resources/user.csv");
const MATRIX_DEF: &str = include_str!("./resources/matrix.def");
const CHAR_DEF: &str = include_str!("./resources/char.def");
const UNK_DEF: &str = include_str!("./resources/unk.def");

#[test]
fn test_tokenize_tokyo() {
    let dict = Dictionary::from_reader(
        LEX_CSV.as_bytes(),
        MATRIX_DEF.as_bytes(),
        CHAR_DEF.as_bytes(),
        UNK_DEF.as_bytes(),
    )
    .unwrap();

    let mut tokenizer = Tokenizer::new(&dict);
    let tokens = tokenizer.tokenize("東京都").unwrap();

    assert_eq!(tokens.len(), 1);
    {
        let t = tokens.get(0);
        assert_eq!(t.surface().deref(), "東京都");
        assert_eq!(t.range_char(), 0..3);
        assert_eq!(t.range_byte(), 0..9);
        assert_eq!(
            t.feature(),
            "東京都,名詞,固有名詞,地名,一般,*,*,トウキョウト,東京都,*,B,5/9,*,5/9,*"
        );
    }

    //   c=0      c=5320       c=0
    //  [BOS] -- [東京都] -- [EOS]
    //     r=0  l=6   r=8  l=0
    //      c=-79
    assert_eq!(tokens.get(0).total_cost(), -79 + 5320);
}

#[test]
fn test_tokenize_kyotokyo() {
    let dict = Dictionary::from_reader(
        LEX_CSV.as_bytes(),
        MATRIX_DEF.as_bytes(),
        CHAR_DEF.as_bytes(),
        UNK_DEF.as_bytes(),
    )
    .unwrap();

    let mut tokenizer = Tokenizer::new(&dict);
    let tokens = tokenizer.tokenize("京都東京都京都").unwrap();

    assert_eq!(tokens.len(), 3);
    {
        let t = tokens.get(0);
        assert_eq!(t.surface().deref(), "京都");
        assert_eq!(t.range_char(), 0..2);
        assert_eq!(t.range_byte(), 0..6);
        assert_eq!(
            t.feature(),
            "京都,名詞,固有名詞,地名,一般,*,*,キョウト,京都,*,A,*,*,*,1/5"
        );
    }
    {
        let t = tokens.get(1);
        assert_eq!(t.surface().deref(), "東京都");
        assert_eq!(t.range_char(), 2..5);
        assert_eq!(t.range_byte(), 6..15);
        assert_eq!(
            t.feature(),
            "東京都,名詞,固有名詞,地名,一般,*,*,トウキョウト,東京都,*,B,5/9,*,5/9,*"
        );
    }
    {
        let t = tokens.get(2);
        assert_eq!(t.surface().deref(), "京都");
        assert_eq!(t.range_char(), 5..7);
        assert_eq!(t.range_byte(), 15..21);
        assert_eq!(
            t.feature(),
            "京都,名詞,固有名詞,地名,一般,*,*,キョウト,京都,*,A,*,*,*,1/5"
        );
    }

    //   c=0     c=5293    c=5320    c=5293    c=0
    //  [BOS] -- [京都] -- [東京都] -- [京都] -- [EOS]
    //     r=0  l=6  r=6  l=6  r=8  l=6  r=6  l=0
    //      c=-79     c=569     c=-352
    assert_eq!(tokens.get(0).total_cost(), -79 + 5293);
    assert_eq!(
        tokens.get(1).total_cost(),
        tokens.get(0).total_cost() + 569 + 5320
    );
    assert_eq!(
        tokens.get(2).total_cost(),
        tokens.get(1).total_cost() - 352 + 5293
    );
}

#[test]
fn test_tokenize_kyotokyo_with_user() {
    let dict = Dictionary::from_reader(
        LEX_CSV.as_bytes(),
        MATRIX_DEF.as_bytes(),
        CHAR_DEF.as_bytes(),
        UNK_DEF.as_bytes(),
    )
    .unwrap()
    .user_lexicon_from_reader(Some(USER_CSV.as_bytes()))
    .unwrap();

    let mut tokenizer = Tokenizer::new(&dict);
    let tokens = tokenizer.tokenize("京都東京都京都").unwrap();

    assert_eq!(tokens.len(), 2);
    {
        let t = tokens.get(0);
        assert_eq!(t.surface().deref(), "京都東京都");
        assert_eq!(t.range_char(), 0..5);
        assert_eq!(t.range_byte(), 0..15);
        assert_eq!(t.feature(), "カスタム名詞");
    }
    {
        let t = tokens.get(1);
        assert_eq!(t.surface().deref(), "京都");
        assert_eq!(t.range_char(), 5..7);
        assert_eq!(t.range_byte(), 15..21);
        assert_eq!(
            t.feature(),
            "京都,名詞,固有名詞,地名,一般,*,*,キョウト,京都,*,A,*,*,*,1/5"
        );
    }

    //   c=0      c=-1000      c=5293    c=0
    //  [BOS] -- [京都東京都] -- [京都] -- [EOS]
    //     r=0  l=6      r=8  l=6  r=6  l=0
    //      c=-79         c=-352
    assert_eq!(tokens.get(0).total_cost(), -79 - 1000);
    assert_eq!(
        tokens.get(1).total_cost(),
        tokens.get(0).total_cost() - 352 + 5293
    );
}

#[test]
fn test_tokenize_tokyoto_with_space() {
    let dict = Dictionary::from_reader(
        LEX_CSV.as_bytes(),
        MATRIX_DEF.as_bytes(),
        CHAR_DEF.as_bytes(),
        UNK_DEF.as_bytes(),
    )
    .unwrap();

    let mut tokenizer = Tokenizer::new(&dict);
    let tokens = tokenizer.tokenize("東京 都").unwrap();

    assert_eq!(tokens.len(), 3);
    {
        let t = tokens.get(0);
        assert_eq!(t.surface().deref(), "東京");
        assert_eq!(t.range_char(), 0..2);
        assert_eq!(t.range_byte(), 0..6);
        assert_eq!(
            t.feature(),
            "東京,名詞,固有名詞,地名,一般,*,*,トウキョウ,東京,*,A,*,*,*,*"
        );
    }
    {
        let t = tokens.get(1);
        assert_eq!(t.surface().deref(), " ");
        assert_eq!(t.range_char(), 2..3);
        assert_eq!(t.range_byte(), 6..7);
        assert_eq!(t.feature(), " ,空白,*,*,*,*,*, , ,*,A,*,*,*,*");
    }
    {
        let t = tokens.get(2);
        assert_eq!(t.surface().deref(), "都");
        assert_eq!(t.range_char(), 3..4);
        assert_eq!(t.range_byte(), 7..10);
        assert_eq!(t.feature(), "都,名詞,普通名詞,一般,*,*,*,ト,都,*,A,*,*,*,*");
    }

    //   c=0     c=2816 c=-20000 c=2914   c=0
    //  [BOS] -- [東京] -- [ ] -- [都] -- [EOS]
    //     r=0  l=6 r=6 l=8 r=8 l=8 r=8 l=0
    //      c=-79    c=-390  c=1134  c=-522
    assert_eq!(tokens.get(0).total_cost(), -79 + 2816);
    assert_eq!(
        tokens.get(1).total_cost(),
        tokens.get(0).total_cost() - 390 - 20000
    );
    assert_eq!(
        tokens.get(2).total_cost(),
        tokens.get(1).total_cost() + 1134 + 2914
    );
}

#[test]
fn test_tokenize_tokyoto_with_space_ignored() {
    let dict = Dictionary::from_reader(
        LEX_CSV.as_bytes(),
        MATRIX_DEF.as_bytes(),
        CHAR_DEF.as_bytes(),
        UNK_DEF.as_bytes(),
    )
    .unwrap();

    let mut tokenizer = Tokenizer::new(&dict).ignore_space(true);
    let tokens = tokenizer.tokenize("東京 都").unwrap();

    assert_eq!(tokens.len(), 2);
    {
        let t = tokens.get(0);
        assert_eq!(t.surface().deref(), "東京");
        assert_eq!(t.range_char(), 0..2);
        assert_eq!(t.range_byte(), 0..6);
        assert_eq!(
            t.feature(),
            "東京,名詞,固有名詞,地名,一般,*,*,トウキョウ,東京,*,A,*,*,*,*"
        );
    }
    {
        let t = tokens.get(1);
        assert_eq!(t.surface().deref(), "都");
        assert_eq!(t.range_char(), 3..4);
        assert_eq!(t.range_byte(), 7..10);
        assert_eq!(t.feature(), "都,名詞,普通名詞,一般,*,*,*,ト,都,*,A,*,*,*,*");
    }

    //   c=0     c=2816   c=2914   c=0
    //  [BOS] -- [東京] -- [都] -- [EOS]
    //     r=0  l=6 r=6  l=8 r=8 l=0
    //      c=-79    c=-390  c=-522
    assert_eq!(tokens.get(0).total_cost(), -79 + 2816);
    assert_eq!(
        tokens.get(1).total_cost(),
        tokens.get(0).total_cost() - 390 + 2914
    );
}

#[test]
fn test_tokenize_tokyoto_with_spaces_ignored() {
    let dict = Dictionary::from_reader(
        LEX_CSV.as_bytes(),
        MATRIX_DEF.as_bytes(),
        CHAR_DEF.as_bytes(),
        UNK_DEF.as_bytes(),
    )
    .unwrap();

    let mut tokenizer = Tokenizer::new(&dict).ignore_space(true);
    let tokens = tokenizer.tokenize("東京   都").unwrap();

    assert_eq!(tokens.len(), 2);
    {
        let t = tokens.get(0);
        assert_eq!(t.surface().deref(), "東京");
        assert_eq!(t.range_char(), 0..2);
        assert_eq!(t.range_byte(), 0..6);
        assert_eq!(
            t.feature(),
            "東京,名詞,固有名詞,地名,一般,*,*,トウキョウ,東京,*,A,*,*,*,*"
        );
    }
    {
        let t = tokens.get(1);
        assert_eq!(t.surface().deref(), "都");
        assert_eq!(t.range_char(), 5..6);
        assert_eq!(t.range_byte(), 9..12);
        assert_eq!(t.feature(), "都,名詞,普通名詞,一般,*,*,*,ト,都,*,A,*,*,*,*");
    }

    //   c=0     c=2816   c=2914   c=0
    //  [BOS] -- [東京] -- [都] -- [EOS]
    //     r=0  l=6 r=6  l=8 r=8 l=0
    //      c=-79    c=-390  c=-522
    assert_eq!(tokens.get(0).total_cost(), -79 + 2816);
    assert_eq!(
        tokens.get(1).total_cost(),
        tokens.get(0).total_cost() - 390 + 2914
    );
}

#[test]
fn test_tokenize_tokyoto_startswith_spaces_ignored() {
    let dict = Dictionary::from_reader(
        LEX_CSV.as_bytes(),
        MATRIX_DEF.as_bytes(),
        CHAR_DEF.as_bytes(),
        UNK_DEF.as_bytes(),
    )
    .unwrap();

    let mut tokenizer = Tokenizer::new(&dict).ignore_space(true);
    let tokens = tokenizer.tokenize("   東京都").unwrap();

    assert_eq!(tokens.len(), 1);
    {
        let t = tokens.get(0);
        assert_eq!(t.surface().deref(), "東京都");
        assert_eq!(t.range_char(), 3..6);
        assert_eq!(t.range_byte(), 3..12);
        assert_eq!(
            t.feature(),
            "東京都,名詞,固有名詞,地名,一般,*,*,トウキョウト,東京都,*,B,5/9,*,5/9,*"
        );
    }

    //   c=0      c=5320       c=0
    //  [BOS] -- [東京都] -- [EOS]
    //     r=0  l=6   r=8  l=0
    //      c=-79
    assert_eq!(tokens.get(0).total_cost(), -79 + 5320);
}

#[test]
fn test_tokenize_tokyoto_endswith_spaces_ignored() {
    let dict = Dictionary::from_reader(
        LEX_CSV.as_bytes(),
        MATRIX_DEF.as_bytes(),
        CHAR_DEF.as_bytes(),
        UNK_DEF.as_bytes(),
    )
    .unwrap();

    let mut tokenizer = Tokenizer::new(&dict).ignore_space(true);
    let tokens = tokenizer.tokenize("東京都   ").unwrap();

    assert_eq!(tokens.len(), 1);
    {
        let t = tokens.get(0);
        assert_eq!(t.surface().deref(), "東京都");
        assert_eq!(t.range_char(), 0..3);
        assert_eq!(t.range_byte(), 0..9);
        assert_eq!(
            t.feature(),
            "東京都,名詞,固有名詞,地名,一般,*,*,トウキョウト,東京都,*,B,5/9,*,5/9,*"
        );
    }

    //   c=0      c=5320       c=0
    //  [BOS] -- [東京都] -- [EOS]
    //     r=0  l=6   r=8  l=0
    //      c=-79
    assert_eq!(tokens.get(0).total_cost(), -79 + 5320);
}

#[test]
fn test_tokenize_kampersanda() {
    let dict = Dictionary::from_reader(
        LEX_CSV.as_bytes(),
        MATRIX_DEF.as_bytes(),
        CHAR_DEF.as_bytes(),
        UNK_DEF.as_bytes(),
    )
    .unwrap();

    let mut tokenizer = Tokenizer::new(&dict);
    let tokens = tokenizer.tokenize("kampersanda").unwrap();

    assert_eq!(tokens.len(), 1);
    {
        let t = tokens.get(0);
        assert_eq!(t.surface().deref(), "kampersanda");
        assert_eq!(t.range_char(), 0..11);
        assert_eq!(t.range_byte(), 0..11);
        assert_eq!(t.feature(), "名詞,普通名詞,一般,*,*,*");
    }

    //   c=0        c=11633         c=0
    //  [BOS] -- [kampersanda] -- [EOS]
    //     r=0  l=7         r=7  l=0
    //      c=887
    assert_eq!(tokens.get(0).total_cost(), 887 + 11633);
}

#[test]
fn test_tokenize_kampersanda_with_user() {
    let dict = Dictionary::from_reader(
        LEX_CSV.as_bytes(),
        MATRIX_DEF.as_bytes(),
        CHAR_DEF.as_bytes(),
        UNK_DEF.as_bytes(),
    )
    .unwrap()
    .user_lexicon_from_reader(Some(USER_CSV.as_bytes()))
    .unwrap();

    let mut tokenizer = Tokenizer::new(&dict);
    let tokens = tokenizer.tokenize("kampersanda").unwrap();

    assert_eq!(tokens.len(), 1);
    {
        let t = tokens.get(0);
        assert_eq!(t.surface().deref(), "kampersanda");
        assert_eq!(t.range_char(), 0..11);
        assert_eq!(t.range_byte(), 0..11);
        assert_eq!(t.feature(), "カスタム名詞");
    }

    //   c=0        c=-2000        c=0
    //  [BOS] -- [kampersanda] -- [EOS]
    //     r=0  l=7         r=7  l=0
    //      c=887
    assert_eq!(tokens.get(0).total_cost(), 887 - 2000);
}

#[test]
fn test_tokenize_kampersanda_with_max_grouping() {
    let dict = Dictionary::from_reader(
        LEX_CSV.as_bytes(),
        MATRIX_DEF.as_bytes(),
        CHAR_DEF.as_bytes(),
        UNK_DEF.as_bytes(),
    )
    .unwrap();

    let mut tokenizer = Tokenizer::new(&dict).ignore_space(true).max_grouping_len(9);
    let tokens = tokenizer.tokenize("kampersanda").unwrap();

    assert_eq!(tokens.len(), 2);
    {
        let t = tokens.get(0);
        assert_eq!(t.surface().deref(), "k");
        assert_eq!(t.range_char(), 0..1);
        assert_eq!(t.range_byte(), 0..1);
        assert_eq!(t.feature(), "名詞,普通名詞,一般,*,*,*");
    }
    {
        let t = tokens.get(1);
        assert_eq!(t.surface().deref(), "ampersanda");
        assert_eq!(t.range_char(), 1..11);
        assert_eq!(t.range_byte(), 1..11);
        assert_eq!(t.feature(), "名詞,普通名詞,一般,*,*,*");
    }

    //   c=0   c=11633    c=11633        c=0
    //  [BOS] -- [k] -- [ampersanda] -- [EOS]
    //     r=0 l=7 r=7 l=7        r=7  l=0
    //      c=887   c=2341
    assert_eq!(tokens.get(0).total_cost(), 887 + 11633);
    assert_eq!(
        tokens.get(1).total_cost(),
        tokens.get(0).total_cost() + 2341 + 11633
    );
}

#[test]
fn test_tokenize_tokyoken() {
    let dict = Dictionary::from_reader(
        LEX_CSV.as_bytes(),
        MATRIX_DEF.as_bytes(),
        CHAR_DEF.as_bytes(),
        UNK_DEF.as_bytes(),
    )
    .unwrap();

    let mut tokenizer = Tokenizer::new(&dict);
    let tokens = tokenizer.tokenize("東京県に行く").unwrap();

    assert_eq!(tokens.len(), 4);
}

/// This test is to check if the category order in char.def is preserved.
#[test]
fn test_tokenize_kanjinumeric() {
    let dict = Dictionary::from_reader(
        LEX_CSV.as_bytes(),
        MATRIX_DEF.as_bytes(),
        CHAR_DEF.as_bytes(),
        UNK_DEF.as_bytes(),
    )
    .unwrap();

    let mut tokenizer = Tokenizer::new(&dict);
    let tokens = tokenizer.tokenize("一橋大学大学院").unwrap();

    assert_eq!(tokens.len(), 1);
    {
        let t = tokens.get(0);
        assert_eq!(t.surface().deref(), "一橋大学大学院");
        assert_eq!(t.range_char(), 0..7);
        assert_eq!(t.range_byte(), 0..21);
        assert_eq!(t.feature(), "名詞,数,*,*,*,*,*");
    }
}

#[test]
fn test_tokenize_empty() {
    let dict = Dictionary::from_reader(
        LEX_CSV.as_bytes(),
        MATRIX_DEF.as_bytes(),
        CHAR_DEF.as_bytes(),
        UNK_DEF.as_bytes(),
    )
    .unwrap();

    let mut tokenizer = Tokenizer::new(&dict);
    let tokens = tokenizer.tokenize("").unwrap();

    assert_eq!(tokens.len(), 0);
}
