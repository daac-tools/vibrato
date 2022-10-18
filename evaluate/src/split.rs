use std::path::PathBuf;

use clap::{error::ErrorKind, CommandFactory, Parser};
use rand::seq::SliceRandom;
use vibrato::trainer::Corpus;

fn parse_ratio(val: &str) -> Result<f64, String> {
    let val = val.parse::<f64>().map_err(|e| e.to_string())?;
    if (0.0..=1.0).contains(&val) {
        Ok(val)
    } else {
        Err(format!("{val} is not in 0.0..=1.0"))
    }
}

#[derive(Parser, Debug)]
#[clap(
    name = "split",
    about = "Shuffle and split corpus into train/valid/test"
)]
struct Args {
    /// Corpus file to be split.
    #[clap(short = 'i', long)]
    corpus_in: PathBuf,

    /// Destination for training data.
    #[clap(short = 't', long)]
    train_out: PathBuf,

    /// Destination for validation data.
    #[clap(short = 'v', long)]
    valid_out: PathBuf,

    /// Destination for testing data.
    #[clap(short = 'e', long)]
    test_out: PathBuf,

    /// Ratio of validation data. (0.0 to 1.0)
    #[clap(long, default_value = "0.1", value_parser = parse_ratio)]
    valid_ratio: f64,

    /// Ratio of testing data. (0.0 to 1.0)
    #[clap(long, default_value = "0.1", value_parser = parse_ratio)]
    test_ratio: f64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let rdr = std::fs::File::open(args.corpus_in)?;
    let mut corpus = Corpus::from_reader(rdr)?;

    let valid_len = (corpus.len() as f64 * args.valid_ratio) as usize;
    let test_len = (corpus.len() as f64 * args.test_ratio) as usize;
    if valid_len + test_len > corpus.len() {
        Args::command().error(
            ErrorKind::InvalidValue,
            "the total size of the validation and the test set must be less than or equal to the corpus size",
        )
        .exit();
    }

    let mut rng = rand::thread_rng();
    corpus.shuffle(&mut rng);

    let mut train_wtr = std::fs::File::create(args.train_out)?;
    let mut valid_wtr = std::fs::File::create(args.valid_out)?;
    let mut test_wtr = std::fs::File::create(args.test_out)?;

    let mut it = corpus.iter();
    for (_, example) in (0..valid_len).zip(&mut it) {
        example.write(&mut valid_wtr)?;
    }
    for (_, example) in (0..test_len).zip(&mut it) {
        example.write(&mut test_wtr)?;
    }
    for example in it {
        example.write(&mut train_wtr)?;
    }

    Ok(())
}
