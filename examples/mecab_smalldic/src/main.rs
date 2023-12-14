use std::error::Error;
use std::fs::File;
use std::path::PathBuf;
use std::time::Instant;

use vibrato::{dictionary::SystemDictionaryBuilder, mecab};

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(
    name = "mecab_smalldic",
    about = "A program to compile a small dictionary from the given MeCab model."
)]
struct Args {
    /// System lexicon file (lex.csv).
    #[clap(short = 'l', long)]
    lexicon_in: PathBuf,

    /// Unknown word definition file (unk.def).
    #[clap(short = 'u', long)]
    unk_in: PathBuf,

    /// Character definition file (char.def).
    #[clap(short = 'c', long)]
    char_in: PathBuf,

    /// Feature definition (feature.def).
    #[clap(short = 'f', long)]
    feature_in: PathBuf,

    /// Feature information associated with right connection IDs (right-id.def).
    #[clap(short = 'a', long)]
    right_id_in: PathBuf,

    /// Feature information associated with left connection IDs (left-id.def).
    #[clap(short = 'b', long)]
    left_id_in: PathBuf,

    /// Model file (model.def).
    #[clap(short = 'm', long)]
    text_model_in: PathBuf,

    /// Cost factor multiplied when costs are casted to integers. This value may be defined in
    /// your dicrc.
    #[clap(short = 'r', long)]
    cost_factor: f64,

    /// Option to control trade-off between speed and memory.
    /// When setting it, the resulting model will be faster but larger.
    /// This option is enabled when bi-gram information is specified.
    #[clap(long)]
    dual_connector: bool,

    /// File to which the binary dictionary is output (in zstd).
    #[clap(short = 'o', long)]
    sysdic_out: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    eprintln!("Compiling the system dictionary...");
    let start = Instant::now();

    let mut bigram_right = vec![];
    let mut bigram_left = vec![];
    let mut bigram_cost = vec![];

    mecab::generate_bigram_info(
        File::open(args.feature_in)?,
        File::open(args.right_id_in)?,
        File::open(args.left_id_in)?,
        File::open(args.text_model_in)?,
        args.cost_factor,
        &mut bigram_right,
        &mut bigram_left,
        &mut bigram_cost,
    )?;

    let dict = SystemDictionaryBuilder::from_readers_with_bigram_info(
        File::open(args.lexicon_in)?,
        bigram_right.as_slice(),
        bigram_left.as_slice(),
        bigram_cost.as_slice(),
        File::open(args.char_in)?,
        File::open(args.unk_in)?,
        args.dual_connector,
    )?;
    eprintln!("{} seconds", start.elapsed().as_secs_f64());

    eprintln!(
        "Writing the system dictionary in zstd...: {:?}",
        &args.sysdic_out
    );
    let mut f = zstd::Encoder::new(File::create(args.sysdic_out)?, 19)?;
    dict.write(&mut f)?;
    f.finish()?;

    Ok(())
}
