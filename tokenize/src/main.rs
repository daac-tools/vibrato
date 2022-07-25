use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

use tinylattice::Tokenizer;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "main", about = "A program.")]
struct Args {
    #[clap(short = 'i', long)]
    sysdic_filename: String,

    #[clap(short = 'w', long)]
    wakachi: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    eprintln!("Loading the dictionary...");
    let mut reader = BufReader::new(File::open(args.sysdic_filename)?);
    let config = bincode::config::standard()
        .with_little_endian()
        .with_fixed_int_encoding()
        .write_fixed_array_length();
    let dict = bincode::decode_from_std_read(&mut reader, config)?;
    let mut tokenizer = Tokenizer::new(&dict);
    eprintln!("Ready to tokenize :)");

    #[allow(clippy::significant_drop_in_scrutinee)]
    for line in std::io::stdin().lock().lines() {
        let line = line?;
        let tokens = tokenizer.tokenize(line);
        if args.wakachi {
            for i in 0..tokens.len() {
                print!(
                    "{}{}",
                    tokens.surface(i),
                    if i != tokens.len() - 1 { ' ' } else { '\n' }
                );
            }
        } else {
            for i in 0..tokens.len() {
                println!("{}\t{}", tokens.surface(i), tokens.feature(i))
            }
            println!("EOS");
        }
    }

    Ok(())
}
