use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

use vibrato::dictionary::{ConnIdCounter, ConnIdMapper, Dictionary};
use vibrato::Tokenizer;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "main", about = "A program.")]
struct Args {
    #[clap(short = 'i', long)]
    sysdic_filename: String,

    #[clap(short = 't', long)]
    train_filename: String,

    #[clap(short = 'o', long)]
    output_filename: String,

    #[clap(short = 'm', long)]
    mapping_basename: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    eprintln!("Loading the dictionary...");
    let mut reader = BufReader::new(File::open(args.sysdic_filename)?);
    let config = bincode::config::standard()
        .with_little_endian()
        .with_fixed_int_encoding()
        .write_fixed_array_length();
    let mut dict: Dictionary = bincode::decode_from_std_read(&mut reader, config)?;

    eprintln!("Training connection id mappings...");
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
    let l_ranks = lid_probs.iter().map(|p| u16::try_from(p.0).unwrap());
    let r_ranks = rid_probs.iter().map(|p| u16::try_from(p.0).unwrap());
    let mapper = ConnIdMapper::from_ranks(l_ranks, r_ranks)?;
    dict.do_mapping(&mapper);

    eprintln!(
        "Writting the system dictionary...: {}",
        &args.output_filename
    );
    let mut writer = BufWriter::new(File::create(args.output_filename)?);
    let config = bincode::config::standard()
        .with_little_endian()
        .with_fixed_int_encoding()
        .write_fixed_array_length();
    let num_bytes = bincode::encode_into_std_write(dict, &mut writer, config)?;
    eprintln!("{} MiB", num_bytes as f64 / (1024. * 1024.));

    if let Some(mapping_basename) = args.mapping_basename {
        eprintln!("Writting connection id mappings...");
        {
            let output_filename = format!("{}.lmap", &mapping_basename);
            let mut w = BufWriter::new(File::create(&output_filename).unwrap());
            for (i, p) in lid_probs {
                w.write_all(format!("{}\t{}\n", i, p).as_bytes())?;
            }
            println!("Wrote {}", output_filename);
        }
        {
            let output_filename = format!("{}.rmap", &mapping_basename);
            let mut w = BufWriter::new(File::create(&output_filename).unwrap());
            for (i, p) in rid_probs {
                w.write_all(format!("{}\t{}\n", i, p).as_bytes())?;
            }
            println!("Wrote {}", output_filename);
        }
    }

    Ok(())
}
