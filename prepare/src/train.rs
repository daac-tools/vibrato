use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

use vibrato::dictionary::{ConnIdCounter, Dictionary};
use vibrato::Tokenizer;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "main", about = "A program.")]
struct Args {
    #[clap(short = 'i', long)]
    sysdic_filename: String,

    #[clap(short = 'o', long)]
    output_basename: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    eprintln!("Loading the dictionary...");
    let mut reader = BufReader::new(File::open(args.sysdic_filename)?);
    let dict: Dictionary =
        bincode::decode_from_std_read(&mut reader, vibrato::common::bincode_config())?;

    eprintln!("Training connection id mappings...");
    let connector = dict.connector();
    let mut tokenizer = Tokenizer::new(&dict);
    let mut counter = ConnIdCounter::new(connector.num_left(), connector.num_right());

    #[allow(clippy::significant_drop_in_scrutinee)]
    for line in std::io::stdin().lock().lines() {
        let line = line?;
        tokenizer.tokenize(line);
        tokenizer.add_connid_counts(&mut counter);
    }
    let (lid_probs, rid_probs) = counter.compute_probs();

    eprintln!("Writting connection id mappings...");
    {
        let output_filename = format!("{}.lmap", &args.output_basename);
        let mut w = BufWriter::new(File::create(&output_filename).unwrap());
        for (i, p) in lid_probs {
            w.write_all(format!("{}\t{}\n", i, p).as_bytes())?;
        }
        println!("Wrote {}", output_filename);
    }
    {
        let output_filename = format!("{}.rmap", &args.output_basename);
        let mut w = BufWriter::new(File::create(&output_filename).unwrap());
        for (i, p) in rid_probs {
            w.write_all(format!("{}\t{}\n", i, p).as_bytes())?;
        }
        println!("Wrote {}", output_filename);
    }

    Ok(())
}
