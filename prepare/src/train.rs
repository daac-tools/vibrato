use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

use vibrato::dictionary::Dictionary;
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
    let reader = BufReader::new(File::open(args.sysdic_filename)?);
    #[cfg(not(feature = "unchecked"))]
    let dict = Dictionary::read(reader)?;
    #[cfg(feature = "unchecked")]
    let dict = unsafe { Dictionary::read_unchecked(reader)? };

    eprintln!("Training connection id mappings...");
    let tokenizer = Tokenizer::new(dict);
    let mut worker = tokenizer.new_worker();
    worker.init_connid_counter();

    #[allow(clippy::significant_drop_in_scrutinee)]
    for line in std::io::stdin().lock().lines() {
        let line = line?;
        worker.reset_sentence(line)?;
        worker.tokenize();
        worker.update_connid_counts();
    }
    let (lid_probs, rid_probs) = worker.compute_connid_probs();

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
