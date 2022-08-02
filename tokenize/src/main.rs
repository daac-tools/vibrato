use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

use vibrato::dictionary::{Dictionary, LexType, Lexicon};
use vibrato::Tokenizer;

use clap::Parser;

#[derive(Clone, Debug)]
enum TokMode {
    Standard,
    Wakati,
}

impl FromStr for TokMode {
    type Err = &'static str;
    fn from_str(mode: &str) -> Result<Self, Self::Err> {
        match mode {
            "standard" => Ok(TokMode::Standard),
            "wakati" => Ok(TokMode::Wakati),
            _ => Err("Could not parse a mode"),
        }
    }
}

#[derive(Parser, Debug)]
#[clap(name = "main", about = "A program.")]
struct Args {
    #[clap(short = 'i', long)]
    sysdic_filename: String,

    #[clap(short = 'u', long)]
    userdic_csv_filename: Option<String>,

    #[clap(short = 'm', long, default_value = "standard")]
    mode: TokMode,

    #[clap(short = 'M', long)]
    mecab_mode: bool,
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
    if let Some(userdic_csv_filename) = args.userdic_csv_filename {
        dict.reset_user_lexicon(Lexicon::from_reader(
            File::open(userdic_csv_filename)?,
            LexType::User,
        )?);
    }

    let mut tokenizer = Tokenizer::new(&dict);
    if args.mecab_mode {
        tokenizer = tokenizer.mecab();
    }

    eprintln!("Ready to tokenize :)");

    #[allow(clippy::significant_drop_in_scrutinee)]
    for line in std::io::stdin().lock().lines() {
        let line = line?;
        let tokens = tokenizer.tokenize(line);
        match args.mode {
            TokMode::Standard => {
                for i in 0..tokens.len() {
                    print!("{}\t{}", tokens.surface(i), tokens.feature(i));
                    if tokens.is_unknown(i) {
                        print!(" (unk)");
                    }
                    println!();
                }
                println!("EOS");
            }
            TokMode::Wakati => {
                for i in 0..tokens.len() {
                    if i != 0 {
                        print!(" ");
                    }
                    print!("{}", tokens.surface(i));
                }
                println!();
            }
        }
    }

    Ok(())
}
