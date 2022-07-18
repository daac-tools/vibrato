use std::error::Error;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;

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
        Lexicon::from_lines(to_lines(sysdic_filename), LexType::System)?,
        Connector::from_lines(to_lines(matrix_filename))?,
        CharProperty::from_lines(to_lines(chardef_filename))?,
        UnkHandler::from_lines(to_lines(unkdef_filename))?,
    ));

    let mut sentence = Sentence::new();

    for line in std::io::stdin().lock().lines() {
        let line = line?;

        sentence.set_sentence(line);
        tokenizer.tokenize(&mut sentence);
        let morphs = sentence.morphs();

        if wakachi {
            for m in morphs {
                print!("{} ", sentence.surface(m));
            }
            println!();
        } else {
            for m in morphs {
                println!("{}\t{}", sentence.surface(m), tokenizer.feature(m));
            }
            println!("EOS");
        }
    }

    Ok(())
}

fn to_lines<P>(path: P) -> impl Iterator<Item = String>
where
    P: AsRef<Path>,
{
    let buf = BufReader::new(File::open(path).unwrap());
    buf.lines().map(|line| line.unwrap())
}
