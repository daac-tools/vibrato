use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::time::Instant;

use vibrato::dictionary::Dictionary;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "main", about = "A program to compile the system dictionary.")]
struct Args {
    /// System lexicon file (lex.csv).
    #[clap(short = 'l', long)]
    lexicon_in: PathBuf,

    /// Unknown word definition file (unk.def).
    #[clap(short = 'u', long)]
    unk_in: PathBuf,

    /// Character definition file (char.def).
    #[clap(short = 'c', long)]
    char_def: PathBuf,

    /// A file to which the matrix is input (matrix.def).
    #[clap(short = 'm', long)]
    matrix_in: PathBuf,

    /// A file to which the binary dictionary is output.
    #[clap(short = 'o', long)]
    sysdic_out: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    eprintln!("Compiling the system dictionary...");
    let start = Instant::now();
    let dict = Dictionary::from_readers(
        File::open(args.lexicon_in)?,
        File::open(args.matrix_in)?,
        File::open(args.char_def)?,
        File::open(args.unk_in)?,
    )?;
    eprintln!("{} seconds", start.elapsed().as_secs_f64());

    eprintln!("Writting the system dictionary...: {:?}", &args.sysdic_out);
    let num_bytes = dict.write(BufWriter::new(File::create(args.sysdic_out)?))?;
    eprintln!("{} MiB", num_bytes as f64 / (1024. * 1024.));

    Ok(())
}
