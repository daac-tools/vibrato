mod timer;

use std::error::Error;
use std::fs::File;
use std::io::BufRead;
use std::path::PathBuf;

use vibrato::{Dictionary, Tokenizer};

use timer::Timer;

use clap::Parser;

const RUNS: usize = 10;
const TRIALS: usize = 10;

#[derive(Parser, Debug)]
#[clap(
    name = "benchmark",
    about = "A program to benchmark tokenization speed."
)]
struct Args {
    /// System dictionary (in zstd).
    #[clap(short = 'i', long)]
    sysdic: PathBuf,

    /// Ignores white spaces in input strings.
    #[clap(short = 'S', long)]
    ignore_space: bool,

    /// Maximum length of unknown words.
    #[clap(short = 'M', long)]
    max_grouping_len: Option<usize>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let reader = zstd::Decoder::new(File::open(args.sysdic)?)?;
    let dict = Dictionary::read(reader)?;

    let tokenizer = Tokenizer::new(dict)
        .ignore_space(args.ignore_space)?
        .max_grouping_len(args.max_grouping_len.unwrap_or(0));
    let mut worker = tokenizer.new_worker();

    let lines: Vec<_> = std::io::stdin()
        .lock()
        .lines()
        .map(|l| l.unwrap())
        .collect();

    let mut measure = |t: &mut Timer| {
        let mut n_words = 0;
        for _ in 0..RUNS {
            t.start();
            for line in &lines {
                worker.reset_sentence(line);
                worker.tokenize();
                n_words += worker.num_tokens();
            }
            t.stop();
        }
        dbg!(n_words);
    };

    let mut t = Timer::new();

    // Warmup
    t.reset();
    measure(&mut t);
    println!("Warmup: {}", t.average());

    let (mut min, mut max, mut avg) = (0.0, 0.0, 0.0);

    for _ in 0..TRIALS {
        t.reset();
        measure(&mut t);
        t.discard_min();
        t.discard_max();
        min += t.min();
        avg += t.average();
        max += t.max();
    }

    min /= TRIALS as f64;
    avg /= TRIALS as f64;
    max /= TRIALS as f64;

    println!("Number_of_sentences: {}", lines.len());
    println!("Elapsed_seconds_to_tokenize_all_sentences: [{min},{avg},{max}]");

    Ok(())
}
