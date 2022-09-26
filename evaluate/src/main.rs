use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use csv_core::ReadFieldResult;
use vibrato::dictionary::Dictionary;
use vibrato::trainer::Corpus;
use vibrato::Tokenizer;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "evaluate", about = "Evaluate the model accuracy")]
struct Args {
    /// Test corpus.
    #[clap(short = 't', long)]
    test_in: PathBuf,

    /// System dictionary.
    #[clap(short = 'i', long)]
    sysdic_in: PathBuf,

    /// User dictionary.
    #[clap(short = 'u', long)]
    userlex_csv_in: Option<String>,

    /// Maximum length of unknown words.
    #[clap(short = 'M', long)]
    max_grouping_len: Option<usize>,

    /// Index of features used to determine the correctness.
    ///
    /// Specify comma-separated indices starting from 0.
    /// If empty, all features are used.
    #[clap(long, default_value = "")]
    feature_indices: String,
}

fn parse_csv_row(row: &str) -> Vec<String> {
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

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mut feature_indices: Vec<usize> = vec![];
    if !args.feature_indices.is_empty() {
        for i in args.feature_indices.split(',') {
            feature_indices.push(i.parse()?);
        }
    }

    eprintln!("Loading the dictionary...");
    let reader = BufReader::new(File::open(args.sysdic_in)?);
    let mut dict = Dictionary::read(reader)?;

    if let Some(userlex_csv_in) = args.userlex_csv_in {
        dict = dict.user_lexicon_from_reader(Some(File::open(userlex_csv_in)?))?;
    }

    let tokenizer = Tokenizer::new(dict).max_grouping_len(args.max_grouping_len.unwrap_or(0));
    let mut worker = tokenizer.new_worker();

    eprintln!("Tokenizing...");

    let rdr = File::open(args.test_in)?;
    let corpus = Corpus::from_reader(rdr)?;

    let mut num_ref = 0;
    let mut num_sys = 0;
    let mut num_cor = 0;
    for example in corpus.iter() {
        let mut input_str = String::new();
        let mut refs = HashSet::new();
        let mut syss = HashSet::new();
        let mut start = 0;
        for token in example.tokens() {
            input_str.push_str(token.surface());
            let len = token.surface().chars().count();
            let features = parse_csv_row(token.feature());
            if feature_indices.is_empty() {
                refs.insert((start..start + len, features));
            } else {
                let mut features_chose = vec![];
                for &i in &feature_indices {
                    features_chose.push(
                        features
                            .get(i)
                            .map_or_else(|| "*".to_string(), |x| x.to_string()),
                    );
                }
                refs.insert((start..start + len, features_chose));
            }
            start += len;
        }
        worker.reset_sentence(input_str)?;
        worker.tokenize();
        for token in worker.token_iter() {
            let features = parse_csv_row(token.feature());
            if feature_indices.is_empty() {
                syss.insert((token.range_char(), features));
            } else {
                let mut features_chose = vec![];
                for &i in &feature_indices {
                    features_chose.push(
                        features
                            .get(i)
                            .map_or_else(|| "*".to_string(), |x| x.to_string()),
                    );
                }
                syss.insert((token.range_char(), features_chose));
            }
        }
        num_ref += refs.len();
        num_sys += syss.len();
        num_cor += refs.intersection(&syss).count();
    }

    let precision = num_cor as f64 / num_sys as f64;
    let recall = num_cor as f64 / num_ref as f64;
    let f1 = 2.0 * precision * recall / (precision + recall);
    println!("Precision = {precision}");
    println!("Recall = {recall}");
    println!("F1 = {f1}");

    Ok(())
}
