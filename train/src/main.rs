use std::fs::File;
use std::path::PathBuf;

use clap::Parser;
use vibrato::trainer::{Corpus, Trainer, TrainerConfig};

#[derive(Parser, Debug)]
#[clap(name = "train", about = "Model trainer")]
struct Args {
    /// Lexicon file (lex.csv) to be weighted.
    ///
    /// All connection IDs and weights must be set to 0.
    #[clap(short = 'l', long)]
    seed_lexicon: PathBuf,

    /// Unknown word file (unk.def) to be weighted.
    ///
    /// All connection IDs and weights must be set to 0.
    #[clap(short = 'u', long)]
    seed_unk: PathBuf,

    /// Corpus file to be trained. The format is the same as the output of the tokenize command of
    /// Vibrato.
    #[clap(short = 't', long)]
    corpus: PathBuf,

    /// Character definition file (char.def).
    #[clap(short = 'c', long)]
    char_def: PathBuf,

    /// Feature definition file (feature.def).
    #[clap(short = 'f', long)]
    feature_def: PathBuf,

    /// Rewrite rule definition file (rewrite.def).
    #[clap(short = 'r', long)]
    rewrite_def: PathBuf,

    /// A file to which the model is output. The file is compressed by zstd.
    #[clap(short = 'o', long)]
    model_out: PathBuf,

    /// Regularization coefficient. The larger the value, the stronger the L1-regularization.
    #[clap(long, default_value = "0.01")]
    lambda: f64,

    /// Maximum number of iterations.
    #[clap(long, default_value = "100")]
    max_iter: u64,

    /// Number of threads.
    #[clap(long, default_value = "1")]
    num_threads: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let lexicon_rdr = File::open(args.seed_lexicon)?;
    let char_prop_rdr = File::open(args.char_def)?;
    let unk_handler_rdr = File::open(args.seed_unk)?;
    let feature_templates_rdr = File::open(args.feature_def)?;
    let rewrite_rules_rdr = File::open(args.rewrite_def)?;
    let config = TrainerConfig::from_readers(
        lexicon_rdr,
        char_prop_rdr,
        unk_handler_rdr,
        feature_templates_rdr,
        rewrite_rules_rdr,
    )?;

    let trainer = Trainer::new(config)?
        .regularization_cost(args.lambda)
        .max_iter(args.max_iter)
        .num_threads(args.num_threads);

    let corpus_rdr = File::open(args.corpus)?;
    let corpus = Corpus::from_reader(corpus_rdr)?;

    let model = trainer.train(corpus)?;

    let mut encoder = zstd::stream::Encoder::new(File::create(args.model_out)?, 19)?;
    model.write_model(&mut encoder)?;
    encoder.finish().unwrap();

    Ok(())
}
