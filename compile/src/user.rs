use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};

use vibrato::dictionary::{Dictionary, LexType, Lexicon};

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "main", about = "A program.")]
struct Args {
    #[clap(short = 'i', long)]
    sysdic_filename: String,

    #[clap(short = 'u', long)]
    userlex_filename: String,

    #[clap(short = 'o', long)]
    output_filename: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    eprintln!("Loading the system dictionary...");
    let mut reader = BufReader::new(File::open(args.sysdic_filename)?);
    let dict: Dictionary =
        bincode::decode_from_std_read(&mut reader, vibrato::common::bincode_config())?;

    eprintln!("Compiling the user lexicon...");
    let mut user_lexicon = Lexicon::from_reader(File::open(args.userlex_filename)?, LexType::User)?;
    if let Some(mapper) = dict.mapper() {
        user_lexicon.do_mapping(mapper);
    }

    eprintln!("Writting the user dictionary...: {}", &args.output_filename);
    let mut writer = BufWriter::new(File::create(args.output_filename)?);
    let num_bytes = bincode::encode_into_std_write(
        user_lexicon,
        &mut writer,
        vibrato::common::bincode_config(),
    )?;
    eprintln!("{} MiB", num_bytes as f64 / (1024. * 1024.));

    Ok(())
}
