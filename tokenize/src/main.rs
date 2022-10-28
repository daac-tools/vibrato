use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
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
#[clap(name = "tokenize", about = "Predicts morphemes")]
struct Args {
    /// System dictionary.
    #[clap(short = 'i', long)]
    sysdic: PathBuf,

    /// User lexicon file.
    #[clap(short = 'u', long)]
    userlex_csv: Option<PathBuf>,

    /// Output mode. Choices are mecab, wakati, and detail.
    #[clap(short = 'O', long, default_value = "mecab")]
    output_mode: OutputMode,

    /// Ignores white spaces in input strings.
    #[clap(short = 'S', long)]
    ignore_space: bool,

    /// Maximum length of unknown words.
    #[clap(short = 'M', long)]
    max_grouping_len: Option<usize>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    eprintln!("Loading the dictionary...");
    let reader = BufReader::new(File::open(args.sysdic)?);
    let mut dict = Dictionary::read(reader)?;

    if let Some(userlex_csv) = args.userlex_csv {
        dict = dict.reset_user_lexicon_from_reader(Some(File::open(userlex_csv)?))?;
    }

    let tokenizer = Tokenizer::new(dict)
        .ignore_space(args.ignore_space)?
        .max_grouping_len(args.max_grouping_len.unwrap_or(0));
    let mut worker = tokenizer.new_worker();

    eprintln!("Ready to tokenize");

    let is_tty = atty::is(atty::Stream::Stdout);

    let out = std::io::stdout();
    let mut out = BufWriter::new(out.lock());
    let lines = std::io::stdin().lock().lines();
    for line in lines {
        let line = line?;
        worker.reset_sentence(line);
        worker.tokenize();
        match args.output_mode {
            OutputMode::Mecab => {
                for i in 0..worker.num_tokens() {
                    let t = worker.token(i);
                    out.write_all(t.surface().as_bytes())?;
                    out.write_all(b"\t")?;
                    out.write_all(t.feature().as_bytes())?;
                    out.write_all(b"\n")?;
                }
                out.write_all(b"EOS\n")?;
                if is_tty {
                    out.flush()?;
                }
            }
            OutputMode::Wakati => {
                for i in 0..worker.num_tokens() {
                    if i != 0 {
                        out.write_all(b" ")?;
                    }
                    out.write_all(worker.token(i).surface().as_bytes())?;
                }
                out.write_all(b"\n")?;
                if is_tty {
                    out.flush()?;
                }
            }
            OutputMode::Detail => {
                for i in 0..worker.num_tokens() {
                    let t = worker.token(i);
                    writeln!(
                        &mut out,
                        "{}\t{}\tlex_type={:?}\tleft_id={}\tright_id={}\tword_cost={}\ttotal_cost={}",
                        t.surface(),
                        t.feature(),
                        t.lex_type(),
                        t.left_id(),
                        t.right_id(),
                        t.word_cost(),
                        t.total_cost(),
                    )?;
                }
                out.write_all(b"EOS\n")?;
                if is_tty {
                    out.flush()?;
                }
            }
        }
    }

    Ok(())
}
