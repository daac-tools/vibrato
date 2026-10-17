use vibrato::{Dictionary, SystemDictionaryBuilder, Tokenizer};

fn dictionary(newline: &str) -> Dictionary {
    SystemDictionaryBuilder::from_readers(
        "猫,0,0,1,cat\n犬,0,0,1,dog\n"
            .replace('\n', newline)
            .as_bytes(),
        b"1 1\n0 0 0\n".as_slice(),
        b"DEFAULT 0 1 0\n".as_slice(),
        "DEFAULT,0,0,100,unknown\n"
            .replace('\n', newline)
            .as_bytes(),
    )
    .unwrap()
}

#[test]
fn compile_crlf_dictionary() {
    let tokenizer = Tokenizer::new(dictionary("\r\n"));
    let mut worker = tokenizer.new_worker();
    worker.reset_sentence("猫犬");
    worker.tokenize();
    assert_eq!(worker.num_tokens(), 2);
    assert_eq!(worker.token(0).feature(), "cat");
    assert_eq!(worker.token(1).feature(), "dog");
}

fn assert_user_dictionary_without_optional_features(csv: &str) {
    let dict = dictionary("\n")
        .reset_user_lexicon_from_reader(Some(csv.as_bytes()))
        .unwrap();
    let tokenizer = Tokenizer::new(dict);
    let mut worker = tokenizer.new_worker();
    worker.reset_sentence("猫犬");
    worker.tokenize();
    assert_eq!(worker.num_tokens(), 1);
    assert_eq!(worker.token(0).surface(), "猫犬");
    assert_eq!(worker.token(0).feature(), "");
    assert_eq!(worker.token(0).word_cost(), -10);
}

#[test]
fn user_dictionary_without_optional_features_at_eof() {
    assert_user_dictionary_without_optional_features("猫犬,0,0,-10");
}

#[test]
fn user_dictionary_without_optional_features_with_lf() {
    assert_user_dictionary_without_optional_features("猫犬,0,0,-10\n");
}

#[test]
fn user_dictionary_without_optional_features_with_crlf() {
    assert_user_dictionary_without_optional_features("猫犬,0,0,-10\r\n");
}

fn assert_rejects_quoted_empty_record(csv: &str) {
    assert!(dictionary("\n")
        .reset_user_lexicon_from_reader(Some(csv.as_bytes()))
        .is_err());
}

#[test]
fn reject_quoted_empty_record_at_eof() {
    assert_rejects_quoted_empty_record("\"\"");
}

#[test]
fn reject_quoted_empty_record_with_lf() {
    assert_rejects_quoted_empty_record("\"\"\n");
}

#[test]
fn reject_quoted_empty_record_with_crlf() {
    assert_rejects_quoted_empty_record("\"\"\r\n");
}

#[test]
fn load_crlf_user_dictionary() {
    let dict = dictionary("\n")
        .reset_user_lexicon_from_reader(Some("猫犬,0,0,-10,custom,pair\r\n".as_bytes()))
        .unwrap();
    let tokenizer = Tokenizer::new(dict);
    let mut worker = tokenizer.new_worker();
    worker.reset_sentence("猫犬");
    worker.tokenize();
    assert_eq!(worker.num_tokens(), 1);
    assert_eq!(worker.token(0).surface(), "猫犬");
    assert_eq!(worker.token(0).feature(), "custom,pair");
}
