use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

use vibrato::dictionary::Dictionary;
use vibrato::Tokenizer;

use clap::Parser;

#[derive(Clone, Debug)]
enum OutputMode {
    Mecab,
    Wakati,
    Detail,
}

impl FromStr for OutputMode {
    type Err = &'static str;
    fn from_str(mode: &str) -> Result<Self, Self::Err> {
        match mode {
            "mecab" => Ok(Self::Mecab),
            "wakati" => Ok(Self::Wakati),
            "detail" => Ok(Self::Detail),
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
    let reader = BufReader::new(File::open(args.sysdic_filename)?);
    let mut dict = Dictionary::read(reader)?;

    if let Some(userlex_csv_filename) = args.userlex_csv_filename {
        dict = dict.user_lexicon_from_reader(Some(File::open(userlex_csv_filename)?))?;
    }

    let tokenizer = Tokenizer::new(dict)
        .ignore_space(args.ignore_space)?
        .max_grouping_len(args.max_grouping_len.unwrap_or(0));
    let mut worker = tokenizer.new_worker();

    eprintln!("Ready to tokenize");

    #[allow(clippy::significant_drop_in_scrutinee)]
    for line in std::io::stdin().lock().lines() {
        let line = line?;
        worker.reset_sentence(line)?;
        worker.tokenize();
        match args.output_mode {
            OutputMode::Mecab => {
                for i in 0..worker.num_tokens() {
                    let t = worker.token(i);
                    println!("{}\t{}", t.surface(), t.feature());
                }
                println!("EOS");
            }
            OutputMode::Wakati => {
                for i in 0..worker.num_tokens() {
                    if i != 0 {
                        print!(" ");
                    }
                    print!("{}", worker.token(i).surface());
                }
                println!();
            }
            OutputMode::Detail => {
                for i in 0..worker.num_tokens() {
                    let t = worker.token(i);
                    println!(
                        "{}\t{}\tlex_type={:?}\tleft_id={}\tright_id={}\tword_cost={}\ttotal_cost={}",
                        t.surface(),
                        t.feature(),
                        t.lex_type(),
                        t.left_id(),
                        t.right_id(),
                        t.word_cost(),
                        t.total_cost(),
                    );
                }
                println!("EOS");
            }
        }
    }

    Ok(())
}
