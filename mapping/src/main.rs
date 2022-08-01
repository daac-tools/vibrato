use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::time::Instant;

use vibrato::dictionary::{
    CharProperty, ConnIdCounter, Connector, Dictionary, LexType, Lexicon, UnkHandler,
};
use vibrato::Tokenizer;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "main", about = "A program.")]
struct Args {
    #[clap(short = 'r', long)]
    resource_dirname: String,

    #[clap(short = 't', long)]
    train_filename: String,

    #[clap(short = 'o', long)]
    output_basename: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let sysdic_filename = format!("{}/lex.csv", &args.resource_dirname);
    let matrix_filename = format!("{}/matrix.def", &args.resource_dirname);
    let chardef_filename = format!("{}/char.def", &args.resource_dirname);
    let unkdef_filename = format!("{}/unk.def", &args.resource_dirname);

    println!("Compiling the system dictionary...");
    let mut start = Instant::now();
    let dict = Dictionary::new(
        Lexicon::from_reader(File::open(sysdic_filename)?, LexType::System)?,
        None,
        Connector::from_reader(File::open(matrix_filename)?)?,
        CharProperty::from_reader(File::open(chardef_filename)?)?,
        UnkHandler::from_reader(File::open(unkdef_filename)?)?,
    );
    println!("{} seconds", start.elapsed().as_secs_f64());

    println!("Training connection id mappings...");
    start = Instant::now();

    let connector = dict.connector();
    let mut tokenizer = Tokenizer::new(&dict);
    let mut counter = ConnIdCounter::new(connector.num_left(), connector.num_right());

    let reader = BufReader::new(File::open(args.train_filename)?);
    for line in reader.lines() {
        let line = line?;
        tokenizer.tokenize(line);
        tokenizer.add_connid_counts(&mut counter);
    }

    let (lid_probs, rid_probs) = counter.compute_probs();
    println!("{} seconds", start.elapsed().as_secs_f64());

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
