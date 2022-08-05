use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

use vibrato::dictionary::{Dictionary, LexType, Lexicon};
use vibrato::Tokenizer;

use clap::Parser;

#[derive(Clone, Debug)]
enum OutputMode {
    Mecab,
    Wakati,
}

impl FromStr for OutputMode {
    type Err = &'static str;
    fn from_str(mode: &str) -> Result<Self, Self::Err> {
        match mode {
            "mecab" => Ok(Self::Mecab),
            "wakati" => Ok(Self::Wakati),
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
    userlex_csv_filename: Option<String>,

    #[clap(short = 'O', long, default_value = "mecab")]
    output_mode: OutputMode,

    #[clap(short = 'S', long)]
    ignore_space: bool,

    #[clap(short = 'M', long)]
    max_grouping_len: Option<usize>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    eprintln!("Loading the dictionary...");
    let mut dict: Dictionary = {
        let mut reader = BufReader::new(File::open(args.sysdic_filename)?);
        bincode::decode_from_std_read(&mut reader, vibrato::common::bincode_config())?
    };

    if let Some(userlex_csv_filename) = args.userlex_csv_filename {
        let user_lexicon = Lexicon::from_reader(File::open(userlex_csv_filename)?, LexType::User)?;
        dict.reset_user_lexicon(Some(user_lexicon));
    }

    let mut tokenizer = Tokenizer::new(&dict);
    if args.ignore_space {
        tokenizer = tokenizer.ignore_space(true);
    }
    if let Some(max_grouping_len) = args.max_grouping_len {
        tokenizer = tokenizer.max_grouping_len(max_grouping_len);
    }

    eprintln!("Ready to tokenize");

    #[allow(clippy::significant_drop_in_scrutinee)]
    for line in std::io::stdin().lock().lines() {
        let line = line?;
        let tokens = tokenizer.tokenize(line);
        match args.output_mode {
            OutputMode::Mecab => {
                for i in 0..tokens.len() {
                    println!("{}\t{}", tokens.surface(i), tokens.feature(i));
                }
                println!("EOS");
            }
            OutputMode::Wakati => {
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
