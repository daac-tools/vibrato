use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::time::Instant;

use vibrato::dictionary::SystemDictionaryBuilder;

use clap::{error::ErrorKind, CommandFactory, Parser};

#[derive(Parser, Debug)]
#[clap(
    name = "compile",
    about = "A program to compile the system dictionary."
)]
struct Args {
    /// System lexicon file (lex.csv).
    #[clap(short = 'l', long)]
    lexicon_in: PathBuf,

    /// Matrix definition file (matrix.def).
    ///
    /// If this argument is not specified, the compiler considers `--bigram-right-in`,
    /// `--bigram-left-in`, and `--bigram-cost-in` arguments.
    #[clap(short = 'm', long)]
    matrix_in: Option<PathBuf>,

    /// Unknown word definition file (unk.def).
    #[clap(short = 'u', long)]
    unk_in: PathBuf,

    /// Character definition file (char.def).
    #[clap(short = 'c', long)]
    char_in: PathBuf,

    /// File to which the binary dictionary is output.
    #[clap(short = 'o', long)]
    sysdic_out: PathBuf,

    /// Bigram information associated with right connection IDs (bigram.right).
    #[clap(long)]
    bigram_right_in: Option<PathBuf>,

    /// Bigram information associated with left connection IDs (bigram.right).
    #[clap(long)]
    bigram_left_in: Option<PathBuf>,

    /// Bigram cost file (bigram.cost).
    #[clap(long)]
    bigram_cost_in: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    eprintln!("Compiling the system dictionary...");
    let start = Instant::now();
    let dict = if let Some(matrix_in) = args.matrix_in {
        SystemDictionaryBuilder::from_readers(
            File::open(args.lexicon_in)?,
            File::open(matrix_in)?,
            File::open(args.char_in)?,
            File::open(args.unk_in)?,
        )?
    } else if let (Some(bigram_right_in), Some(bigram_left_in), Some(bigram_cost_in)) = (
        args.bigram_right_in,
        args.bigram_left_in,
        args.bigram_cost_in,
    ) {
        SystemDictionaryBuilder::raw_connector_from_readers(
            File::open(args.lexicon_in)?,
            File::open(bigram_right_in)?,
            File::open(bigram_left_in)?,
            File::open(bigram_cost_in)?,
            File::open(args.char_in)?,
            File::open(args.unk_in)?,
        )?
    } else {
        Args::command()
            .error(
                ErrorKind::InvalidValue,
                "At least one of --matrin-in or --bigram-{right,left,cost}-in must be specified.",
            )
            .exit();
    };
    eprintln!("{} seconds", start.elapsed().as_secs_f64());

    eprintln!("Writting the system dictionary...: {:?}", &args.sysdic_out);
    let num_bytes = dict.write(BufWriter::new(File::create(args.sysdic_out)?))?;
    eprintln!("{} MiB", num_bytes as f64 / (1024. * 1024.));

    Ok(())
}
