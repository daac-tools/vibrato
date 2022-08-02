use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use std::time::Instant;

use vibrato::dictionary::{CharProperty, Connector, Dictionary, LexType, Lexicon, UnkHandler};

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "main", about = "A program.")]
struct Args {
    #[clap(short = 'r', long)]
    resource_dirname: String,

    #[clap(short = 'o', long)]
    output_filename: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let sysdic_filename = format!("{}/lex.csv", &args.resource_dirname);
    let matrix_filename = format!("{}/matrix.def", &args.resource_dirname);
    let chardef_filename = format!("{}/char.def", &args.resource_dirname);
    let unkdef_filename = format!("{}/unk.def", &args.resource_dirname);

    eprintln!("Compiling the system dictionary...");
    let start = Instant::now();
    let dict = Dictionary::new(
        Lexicon::from_reader(File::open(sysdic_filename)?, LexType::System)?,
        None,
        Connector::from_reader(File::open(matrix_filename)?)?,
        None,
        CharProperty::from_reader(File::open(chardef_filename)?)?,
        UnkHandler::from_reader(File::open(unkdef_filename)?)?,
    );
    eprintln!("{} seconds", start.elapsed().as_secs_f64());

    eprintln!(
        "Writting the system dictionary...: {}",
        &args.output_filename
    );
    let mut writer = BufWriter::new(File::create(args.output_filename)?);
    let config = bincode::config::standard()
        .with_little_endian()
        .with_fixed_int_encoding()
        .write_fixed_array_length();
    let num_bytes = bincode::encode_into_std_write(dict, &mut writer, config)?;
    eprintln!("{} MiB", num_bytes as f64 / (1024. * 1024.));

    Ok(())
}
