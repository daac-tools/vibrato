use std::error::Error;
use std::fs::File;
use std::io::{prelude::*, BufReader, BufWriter};

use vibrato::dictionary::Dictionary;

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
    let reader = BufReader::new(File::open(args.sysdic_filename)?);
    #[cfg(not(feature = "unchecked"))]
    let dict = Dictionary::read(reader)?;
    #[cfg(feature = "unchecked")]
    let dict = unsafe { Dictionary::read_unchecked(reader)? };

    eprintln!("Loading and doing the mapping...");
    let lmap = load_mapping(File::open(format!("{}.lmap", &args.mapping_basename))?)?;
    let rmap = load_mapping(File::open(format!("{}.rmap", &args.mapping_basename))?)?;
    let dict = dict.mapping_from_iter(lmap, rmap)?;

    eprintln!(
        "Writting the mapped system dictionary...: {}",
        &args.output_filename
    );
    let num_bytes = dict.write(BufWriter::new(File::create(args.output_filename)?))?;
    eprintln!("{} MiB", num_bytes as f64 / (1024. * 1024.));

    Ok(())
}

fn load_mapping<R>(rdr: R) -> Result<Vec<u16>, Box<dyn Error>>
where
    R: Read,
{
    let reader = BufReader::new(rdr);
    let lines = reader.lines();
    let mut ids = vec![];
    for line in lines {
        let line = line?;
        let cols: Vec<_> = line.split('\t').collect();
        ids.push(cols[0].parse()?);
    }
    Ok(ids)
}
