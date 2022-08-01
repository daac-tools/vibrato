use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter};
use std::time::Instant;

use vibrato::dictionary::{
    CharProperty, ConnIdMapper, Connector, Dictionary, LexType, Lexicon, UnkHandler,
};

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "main", about = "A program.")]
struct Args {
    #[clap(short = 'r', long)]
    resource_dirname: String,

    #[clap(short = 'm', long)]
    mapping_basename: Option<String>,

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
    let start = Instant::now();
    let mut dict = Dictionary::new(
        Lexicon::from_reader(File::open(sysdic_filename)?, LexType::System)?,
        None,
        Connector::from_reader(File::open(matrix_filename)?)?,
        CharProperty::from_reader(File::open(chardef_filename)?)?,
        UnkHandler::from_reader(File::open(unkdef_filename)?)?,
    );

    if let Some(mapping_basename) = args.mapping_basename {
        let l_ranks = read_mapping(&format!("{}.lmap", &mapping_basename))?;
        let r_ranks = read_mapping(&format!("{}.rmap", &mapping_basename))?;
        let mapper = ConnIdMapper::from_ranks(l_ranks, r_ranks)?;
        dict.do_mapping(&mapper);
    }
    println!("{} seconds", start.elapsed().as_secs_f64());

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

fn read_mapping(filepath: &str) -> Result<Vec<u16>, Box<dyn Error>> {
    let mut mapping = vec![];
    let reader = BufReader::new(File::open(filepath)?);
    for line in reader.lines() {
        let line = line?;
        let cols: Vec<_> = line.split("\t").collect();
        mapping.push(cols[0].parse()?)
    }
    Ok(mapping)
}
