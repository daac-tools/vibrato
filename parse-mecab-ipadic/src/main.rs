use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::io::{Read, Write};

use encoding_rs::EUC_JP;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "main", about = "A program.")]
struct Args {
    #[clap(short = 'i', long)]
    input_directory: String,

    #[clap(short = 'o', long)]
    output_directory: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let lex_files = vec![
        "Adj.csv",
        "Adnominal.csv",
        "Adverb.csv",
        "Auxil.csv",
        "Conjunction.csv",
        "Filler.csv",
        "Interjection.csv",
        "Noun.adjv.csv",
        "Noun.adverbal.csv",
        "Noun.csv",
        "Noun.demonst.csv",
        "Noun.nai.csv",
        "Noun.name.csv",
        "Noun.number.csv",
        "Noun.org.csv",
        "Noun.others.csv",
        "Noun.place.csv",
        "Noun.proper.csv",
        "Noun.verbal.csv",
        "Others.csv",
        "Postp-col.csv",
        "Postp.csv",
        "Prefix.csv",
        "Suffix.csv",
        "Symbol.csv",
        "Verb.csv",
    ];

    let mut entries = vec![];
    let mut buffer = vec![];

    for lex_file in lex_files {
        let lex_path = format!("{}/{}", args.input_directory, lex_file);

        buffer.clear();
        BufReader::new(File::open(lex_path).unwrap()).read_to_end(&mut buffer)?;

        let (cow, encoding_used, had_errors) = EUC_JP.decode(&buffer);
        assert_eq!(encoding_used, EUC_JP);
        assert!(!had_errors);

        cow.split('\n').for_each(|l| entries.push(l.to_owned()));
    }

    entries.sort();

    {
        let lex_path = format!("{}/lex.csv", args.output_directory);
        let mut w = BufWriter::new(File::create(lex_path).unwrap());
        for e in &entries {
            if !e.is_empty() {
                w.write(e.as_bytes())?;
                w.write(b"\n")?;
            }
        }
    }

    println!("# = {}", entries.len());

    Ok(())
}
