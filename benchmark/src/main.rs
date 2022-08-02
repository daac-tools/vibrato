mod timer;

use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

use vibrato::Tokenizer;

use timer::Timer;

use clap::Parser;

const RUNS: usize = 10;
const TRIALS: usize = 10;

#[derive(Parser, Debug)]
#[clap(name = "main", about = "A program.")]
struct Args {
    #[clap(short = 'i', long)]
    sysdic_filename: String,

    #[clap(short = 'S', long)]
    ignore_space: bool,

    #[clap(short = 'M', long)]
    max_grouping_len: Option<usize>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mut reader = BufReader::new(File::open(args.sysdic_filename)?);
    let config = bincode::config::standard()
        .with_little_endian()
        .with_fixed_int_encoding()
        .write_fixed_array_length();
    let dict = bincode::decode_from_std_read(&mut reader, config)?;

    let mut tokenizer = Tokenizer::new(&dict);
    if args.ignore_space {
        tokenizer = tokenizer.ignore_space();
    }
    if let Some(max_grouping_len) = args.max_grouping_len {
        tokenizer = tokenizer.max_grouping_len(max_grouping_len);
    }

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
                let tokens = tokenizer.tokenize(line);
                n_words += tokens.len();
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
    println!(
        "Elapsed_seconds_to_tokenize_all_sentences: [{},{},{}]",
        min, avg, max
    );

    Ok(())
}
