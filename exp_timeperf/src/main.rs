use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use exp_timer::Timer;
use tinylattice::dictionary::{CharProperty, Connector, Dictionary, LexType, Lexicon, UnkHandler};
use tinylattice::{Sentence, Tokenizer};

use clap::Parser;

const RUNS: usize = 10;
const TRIALS: usize = 10;

#[derive(Parser, Debug)]
#[clap(name = "main", about = "A program.")]
struct Args {
    #[clap(short = 'r', long)]
    resource_dirname: String,

    #[clap(short = 's', long)]
    sentence_filename: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let lines: Vec<_> = to_lines(&args.sentence_filename).collect();

    let sysdic_filename = format!("{}/lex.csv", &args.resource_dirname);
    let matrix_filename = format!("{}/matrix.def", &args.resource_dirname);
    let chardef_filename = format!("{}/char.def", &args.resource_dirname);
    let unkdef_filename = format!("{}/unk.def", &args.resource_dirname);

    let mut tokenizer = Tokenizer::new(Dictionary::new(
        Lexicon::from_reader(File::open(sysdic_filename)?, LexType::System)?,
        Connector::from_reader(File::open(matrix_filename)?)?,
        CharProperty::from_reader(File::open(chardef_filename)?)?,
        UnkHandler::from_reader(File::open(unkdef_filename)?)?,
    ));

    let mut measure = |t: &mut Timer, s: &mut Sentence| {
        let mut n_words = 0;
        for _ in 0..RUNS {
            t.start();
            for line in &lines {
                s.set_sentence(line);
                tokenizer.tokenize(s);
                n_words += s.morphs().len();
            }
            t.stop();
        }
        dbg!(n_words);
    };

    let mut t = Timer::new();
    let mut s = Sentence::new();

    // Warmup
    t.reset();
    measure(&mut t, &mut s);
    println!("Warmup: {}", t.average());

    let (mut min, mut max, mut avg) = (0.0, 0.0, 0.0);

    for _ in 0..TRIALS {
        t.reset();
        measure(&mut t, &mut s);
        t.discard_min();
        t.discard_max();
        min += t.min();
        avg += t.average();
        max += t.max();
    }

    min = min / TRIALS as f64;
    avg = avg / TRIALS as f64;
    max = max / TRIALS as f64;

    println!("Number_of_sentences: {}", lines.len());
    println!(
        "Elapsed_seconds_to_tokenize_all_sentences: [{},{},{}]",
        min, avg, max
    );

    Ok(())
}

fn to_lines<P>(path: P) -> impl Iterator<Item = String>
where
    P: AsRef<Path>,
{
    let buf = BufReader::new(File::open(path).unwrap());
    buf.lines().map(|line| line.unwrap())
}
