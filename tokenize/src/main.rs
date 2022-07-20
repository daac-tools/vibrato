use std::error::Error;
use std::fs::File;
use std::io::BufRead;

use tinylattice::dictionary::{CharProperty, Connector, Dictionary, LexType, Lexicon, UnkHandler};
use tinylattice::{Sentence, Tokenizer};

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "main", about = "A program.")]
struct Args {
    #[clap(short = 'r', long)]
    resource_dirname: String,

    #[clap(short = 'w', long)]
    wakachi: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let wakachi = args.wakachi;

    let sysdic_filename = format!("{}/lex.csv", &args.resource_dirname);
    let matrix_filename = format!("{}/matrix.def", &args.resource_dirname);
    let chardef_filename = format!("{}/char.def", &args.resource_dirname);
    let unkdef_filename = format!("{}/unk.def", &args.resource_dirname);

    let mut tokenizer = Tokenizer::new(Dictionary::new(
        Lexicon::from_reader(File::open(sysdic_filename)?, LexType::System)?,
        Connector::from_reader(File::open(matrix_filename)?)?,
        CharProperty::from_reader(File::open(chardef_filename)?)?,
        UnkHandler::from_reader(File::open(unkdef_filename)?)?,
    ));

    let mut sentence = Sentence::new();

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
                        println!("{}\t{}", sentence.surface(m), tokenizer.feature(m))
                    }
                    LexType::Unknown => {
                        println!("{}\t{} (UNK)", sentence.surface(m), tokenizer.feature(m))
                    }
                }
            }
            println!("EOS");
        }
    }

    Ok(())
}
