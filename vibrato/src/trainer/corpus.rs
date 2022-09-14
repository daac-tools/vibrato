use std::io::{BufRead, BufReader, Read};

use csv_core::ReadFieldResult;

use crate::errors::{Result, VibratoError};
use crate::sentence::Sentence;

pub fn parse_csv_row(row: &str) -> Vec<String> {
    let mut features = vec![];
    let mut rdr = csv_core::Reader::new();
    let mut bytes = row.as_bytes();
    let mut output = [0; 4096];
    loop {
        let (result, nin, nout) = rdr.read_field(bytes, &mut output);
        let end = match result {
            ReadFieldResult::InputEmpty => true,
            ReadFieldResult::Field { .. } => false,
            _ => unreachable!(),
        };
        features.push(std::str::from_utf8(&output[..nout]).unwrap().to_string());
        if end {
            break;
        }
        bytes = &bytes[nin..];
    }
    features
}

/// Representation of a pair of a surface and features.
pub struct Word {
    surface: String,

    // Since a vector of strings consumes massive memory, a single string is stored and divided as
    // needed.
    feature: String,
}

impl Word {
    /// Returns a surface string.
    pub fn surface(&self) -> &str {
        &self.surface
    }

    /// Returns a concatenated feature string.
    pub fn feature(&self) -> &str {
        &self.feature
    }
}

/// Representation of a sentence.
pub struct Example {
    pub(crate) sentence: Sentence,
    pub(crate) tokens: Vec<Word>,
}

/// Representation of a corpus.
pub struct Corpus {
    pub(crate) examples: Vec<Example>,
}

impl Corpus {
    /// Loads a corpus from the given sink.
    ///
    /// # Arguments
    ///
    /// * `rdr` - A reader of the corpus.
    ///
    /// # Errors
    ///
    /// [`VibratoError`] is returned when an input format is invalid.
    pub fn from_reader<R>(rdr: R) -> Result<Self>
    where
        R: Read,
    {
        let buf = BufReader::new(rdr);

        let mut examples = vec![];
        let mut tokens = vec![];
        for line in buf.lines() {
            let line = line?;
            let mut spl = line.split('\t');
            let surface = spl.next();
            let feature = spl.next();
            let rest = spl.next();
            match (surface, feature, rest) {
                (Some(surface), Some(feature), None) => {
                    tokens.push(Word {
                        surface: surface.to_string(),
                        feature: feature.to_string(),
                    });
                }
                (Some("EOS"), None, None) => {
                    let mut sentence = Sentence::new();
                    let mut input = String::new();
                    for token in &tokens {
                        input.push_str(token.surface());
                    }
                    sentence.set_sentence(input);
                    examples.push(Example { sentence, tokens });
                    tokens = vec![];
                }
                _ => {
                    return Err(VibratoError::invalid_format(
                        "rdr",
                        "Each line must be a pair of a surface and features or `EOS`",
                    ))
                }
            }
        }

        Ok(Self { examples })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_corpus() {
        let corpus_data = "\
トスカーナ\t名詞,トスカーナ
地方\t名詞,チホー
に\t助詞,ニ
行く\t動詞,イク
EOS
火星\t名詞,カセー
猫\t名詞,ネコ
EOS
";

        let corpus = Corpus::from_reader(corpus_data.as_bytes()).unwrap();

        assert_eq!(2, corpus.examples.len());

        let sentence1 = &corpus.examples[0];
        assert_eq!(4, sentence1.tokens.len());

        assert_eq!("トスカーナ", sentence1.tokens[0].surface());
        assert_eq!("名詞,トスカーナ", sentence1.tokens[0].feature());
        assert_eq!("地方", sentence1.tokens[1].surface());
        assert_eq!("名詞,チホー", sentence1.tokens[1].feature());
        assert_eq!("に", sentence1.tokens[2].surface());
        assert_eq!("助詞,ニ", sentence1.tokens[2].feature());
        assert_eq!("行く", sentence1.tokens[3].surface());
        assert_eq!("動詞,イク", sentence1.tokens[3].feature());

        let sentence2 = &corpus.examples[1];
        assert_eq!(2, sentence2.tokens.len());

        assert_eq!("火星", sentence2.tokens[0].surface());
        assert_eq!("名詞,カセー", sentence2.tokens[0].feature());
        assert_eq!("猫", sentence2.tokens[1].surface());
        assert_eq!("名詞,ネコ", sentence2.tokens[1].feature());
    }

    #[test]
    fn test_parse_csv_row() {
        let corpus_data = "\
トスカーナ\t名詞,トスカーナ
EOS
";

        let corpus = Corpus::from_reader(corpus_data.as_bytes()).unwrap();

        assert_eq!(
            &["名詞", "トスカーナ"],
            parse_csv_row(corpus.examples[0].tokens[0].feature()).as_slice()
        );
    }

    #[test]
    fn test_parse_csv_row_with_quote() {
        let corpus_data = "\
1,2-ジクロロエタン\t名詞,\"1,2-ジクロロエタン\"
EOS
";

        let corpus = Corpus::from_reader(corpus_data.as_bytes()).unwrap();

        assert_eq!(
            &["名詞", "1,2-ジクロロエタン"],
            parse_csv_row(corpus.examples[0].tokens[0].feature()).as_slice()
        );
    }

    #[test]
    fn test_load_dictionary() {
        let dictionary_data = "\
トスカーナ,1,2,3,名詞,トスカーナ
地方,4,5,6,名詞,チホー
に,7,8,9,助詞,ニ
";

        let dict = Dictionary::from_reader(dictionary_data.as_bytes()).unwrap();

        assert_eq!(3, dict.words().len());

        assert_eq!("トスカーナ", dict.words()[0].surface());
        assert_eq!("名詞,トスカーナ", dict.words()[0].feature());
        assert_eq!("地方", dict.words()[1].surface());
        assert_eq!("名詞,チホー", dict.words()[1].feature());
        assert_eq!("に", dict.words()[2].surface());
        assert_eq!("助詞,ニ", dict.words()[2].feature());
    }

    #[test]
    fn test_load_dictionary_with_quote() {
        let dictionary_data = "\
\"1,2-ジクロロエタン\",1,2,3,名詞,\"1,2-ジクロロエタン\"
\"\"\"\",4,5,6,名詞,*
";

        let dict = Dictionary::from_reader(dictionary_data.as_bytes()).unwrap();

        assert_eq!(2, dict.words().len());

        assert_eq!("1,2-ジクロロエタン", dict.words()[0].surface());
        assert_eq!("名詞,\"1,2-ジクロロエタン\"", dict.words()[0].feature());
        assert_eq!("\"", dict.words()[1].surface());
        assert_eq!("名詞,*", dict.words()[1].feature());
    }

    #[test]
    fn test_load_dictionary_few_cols() {
        let dictionary_data = "\
\"1,2-ジクロロエタン\",1,2,3
";

        let dict = Dictionary::from_reader(dictionary_data.as_bytes());

        assert!(dict.is_err());
    }
}
