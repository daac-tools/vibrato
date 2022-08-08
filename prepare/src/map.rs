use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};

use vibrato::dictionary::{ConnIdMapper, Dictionary};

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "main", about = "A program.")]
struct Args {
    #[clap(short = 'i', long)]
    sysdic_filename: String,

    #[clap(short = 'm', long)]
    mapping_basename: String,

    #[clap(short = 'o', long)]
    output_filename: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    eprintln!("Loading the dictionary...");
    let mut dict = Dictionary::read(BufReader::new(File::open(args.sysdic_filename)?))?;

    eprintln!("Loading the mapping...");
    let mapper = ConnIdMapper::from_reader(
        File::open(format!("{}.lmap", &args.mapping_basename))?,
        File::open(format!("{}.rmap", &args.mapping_basename))?,
    )?;

    eprintln!("Do mapping...");
    dict.do_mapping(mapper);

    eprintln!(
        "Writting the mapped system dictionary...: {}",
        &args.output_filename
    );
    let num_bytes = dict.write(BufWriter::new(File::create(args.output_filename)?))?;
    eprintln!("{} MiB", num_bytes as f64 / (1024. * 1024.));

    Ok(())
}
