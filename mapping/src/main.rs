use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufWriter, Write};

use tinylattice::dictionary::{
    CharProperty, ConnIdCounter, Connector, Dictionary, LexType, Lexicon, UnkHandler,
};
use tinylattice::Tokenizer;

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

    let output_filename = &args.output_filename;
    let sysdic_filename = format!("{}/lex.csv", &args.resource_dirname);
    let matrix_filename = format!("{}/matrix.def", &args.resource_dirname);
    let chardef_filename = format!("{}/char.def", &args.resource_dirname);
    let unkdef_filename = format!("{}/unk.def", &args.resource_dirname);

    let dict = Dictionary::new(
        Lexicon::from_reader(File::open(sysdic_filename)?, LexType::System)?,
        Connector::from_reader(File::open(matrix_filename)?)?,
        CharProperty::from_reader(File::open(chardef_filename)?)?,
        UnkHandler::from_reader(File::open(unkdef_filename)?)?,
    );
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
    {
        let mut w = BufWriter::new(File::create(format!("{}.lmap", output_filename)).unwrap());
        for (i, p) in lid_probs {
            w.write_all(format!("{}\t{}\n", i, p).as_bytes())?;
        }
    }
    {
        let mut w = BufWriter::new(File::create(format!("{}.rmap", output_filename)).unwrap());
        for (i, p) in rid_probs {
            w.write_all(format!("{}\t{}\n", i, p).as_bytes())?;
        }
    }

    Ok(())
}
