use std::path::PathBuf;

use clap::Parser;
use rand::seq::SliceRandom;
use vibrato::trainer::Corpus;

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
    #[clap(long, default_value = "0.1")]
    valid_ratio: f64,

    /// Ratio of testing data. (0.0 to 1.0)
    #[clap(long, default_value = "0.1")]
    test_ratio: f64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let rdr = std::fs::File::open(args.corpus_in)?;
    let mut corpus = Corpus::from_reader(rdr)?;
    let mut rng = rand::thread_rng();

    let mut train_wtr = std::fs::File::create(args.train_out)?;
    let mut valid_wtr = std::fs::File::create(args.valid_out)?;
    let mut test_wtr = std::fs::File::create(args.test_out)?;

    corpus.shuffle(&mut rng);

    assert!(args.valid_ratio >= 0.0);
    assert!(args.valid_ratio <= 1.0);
    assert!(args.test_ratio >= 0.0);
    assert!(args.test_ratio <= 1.0);
    let valid_len = (corpus.len() as f64 * args.valid_ratio) as usize;
    let test_len = (corpus.len() as f64 * args.test_ratio) as usize;
    assert!(valid_len + test_len <= corpus.len());

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
