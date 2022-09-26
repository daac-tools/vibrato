use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

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
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    eprintln!("Loading the dictionary...");
    let reader = BufReader::new(File::open(args.sysdic_in)?);
    let mut dict = Dictionary::read(reader)?;

    if let Some(userlex_csv_in) = args.userlex_csv_in {
        dict = dict.user_lexicon_from_reader(Some(File::open(userlex_csv_in)?))?;
    }

    let tokenizer = Tokenizer::new(dict).max_grouping_len(args.max_grouping_len.unwrap_or(0));
    let mut worker = tokenizer.new_worker();

    eprintln!("Ready to tokenize");

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
            refs.insert((start..start + len, token.feature().to_string()));
            start += len;
        }
        worker.reset_sentence(input_str)?;
        worker.tokenize();
        for token in worker.token_iter() {
            syss.insert((token.range_char(), token.feature().to_string()));
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
