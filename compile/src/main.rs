use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter};
use std::time::Instant;

use tinylattice::dictionary::{
    CharProperty, ConnIdCounter, ConnIdMapper, Connector, Dictionary, LexType, Lexicon, UnkHandler,
};
use tinylattice::Tokenizer;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "main", about = "A program.")]
struct Args {
    #[clap(short = 'r', long)]
    resource_dirname: String,

    #[clap(short = 't', long)]
    train_filename: Option<String>,

    #[clap(short = 'o', long)]
    output_filename: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let sysdic_filename = format!("{}/lex.csv", &args.resource_dirname);
    let matrix_filename = format!("{}/matrix.def", &args.resource_dirname);
    let chardef_filename = format!("{}/char.def", &args.resource_dirname);
    let unkdef_filename = format!("{}/unk.def", &args.resource_dirname);

    println!("Compiling the system dictionary...");
    let mut start = Instant::now();
    let mut dict = Dictionary::new(
        Lexicon::from_reader(File::open(sysdic_filename)?, LexType::System)?,
        None,
        Connector::from_reader(File::open(matrix_filename)?)?,
        CharProperty::from_reader(File::open(chardef_filename)?)?,
        UnkHandler::from_reader(File::open(unkdef_filename)?)?,
    );
    println!("{} seconds", start.elapsed().as_secs_f64());

    if let Some(train_filename) = args.train_filename {
        println!("Training connection id mappings...");
        start = Instant::now();

        let connector = dict.connector();
        let mut tokenizer = Tokenizer::new(&dict);
        let mut counter = ConnIdCounter::new(connector.num_left(), connector.num_right());

        let reader = BufReader::new(File::open(train_filename)?);
        for line in reader.lines() {
            let line = line?;
            tokenizer.tokenize(line);
            tokenizer.add_connid_counts(&mut counter);
        }

        let (lid_probs, rid_probs) = counter.compute_probs();
        let l_ranks = lid_probs.into_iter().map(|p| u16::try_from(p.0).unwrap());
        let r_ranks = rid_probs.into_iter().map(|p| u16::try_from(p.0).unwrap());
        let mapper = ConnIdMapper::from_ranks(l_ranks, r_ranks)?;
        dict.do_mapping(&mapper);

        println!("{} seconds", start.elapsed().as_secs_f64());
    }

    println!("Writting the system dictionary...");
    let mut writer = BufWriter::new(File::create(args.output_filename)?);
    let config = bincode::config::standard()
        .with_little_endian()
        .with_fixed_int_encoding()
        .write_fixed_array_length();
    let num_bytes = bincode::encode_into_std_write(dict, &mut writer, config)?;
    println!("{} MiB", num_bytes as f64 / (1024. * 1024.));

    Ok(())
}
