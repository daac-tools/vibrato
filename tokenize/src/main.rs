use std::error::Error;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;

use tinylattice::dictionary::{CategoryMap, Connector, LexType, Lexicon, SimpleUnkHandler};
use tinylattice::{Dictionary, Sentence, Tokenizer};

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "main", about = "A program.")]
struct Args {
    #[clap(short = 'd', long)]
    sysdic_filename: String,

    #[clap(short = 'm', long)]
    matrix_filename: String,

    #[clap(short = 'c', long)]
    chardef_filename: String,

    #[clap(short = 'u', long)]
    unkdef_filename: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mut tokenizer = Tokenizer::new(Dictionary::new(
        Lexicon::from_lines(to_lines(args.sysdic_filename), LexType::System)?,
        Connector::from_lines(to_lines(args.matrix_filename))?,
        CategoryMap::from_lines(to_lines(args.chardef_filename))?,
        SimpleUnkHandler::from_lines(to_lines(args.unkdef_filename))?,
    ));

    let mut sentence = Sentence::new();
    for line in std::io::stdin().lock().lines() {
        sentence.set_sentence(line?);
        tokenizer.tokenize(&mut sentence);
        let surfaces = sentence.surfaces();
        println!("{}", surfaces.join(" "));
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
