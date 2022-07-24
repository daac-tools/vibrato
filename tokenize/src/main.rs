use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

use tinylattice::dictionary::LexType;
use tinylattice::{Sentence, Tokenizer};

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

    let mut reader = BufReader::new(File::open(args.sysdic_filename)?);
    let dict = bincode::decode_from_std_read(&mut reader, bincode::config::standard())?;

    let mut tokenizer = Tokenizer::new(&dict);
    let mut sentence = Sentence::new();

    let wakachi = args.wakachi;
    for line in std::io::stdin().lock().lines() {
        let line = line?;

        sentence.set_sentence(line);
        tokenizer.tokenize(&mut sentence);
        let morphs = sentence.morphs();

        if wakachi {
            let surfaces = sentence.surfaces();
            println!("{}", surfaces.join(" "));
        } else {
            for m in morphs {
                match m.word_idx().lex_type() {
                    LexType::System => {
                        println!("{}\t{}", sentence.surface(m), tokenizer.feature(m),)
                    }
                    LexType::Unknown => {
                        println!("{}\t{} (UNK)", sentence.surface(m), tokenizer.feature(m),)
                    }
                }
            }
            println!("EOS");
        }
    }

    Ok(())
}
