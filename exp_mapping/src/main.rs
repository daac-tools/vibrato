use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use tinylattice::dictionary::{CharProperty, Connector, Dictionary, LexType, Lexicon, UnkHandler};
use tinylattice::{Sentence, Tokenizer};

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "main", about = "A program.")]
struct Args {
    #[clap(short = 'r', long)]
    resource_dirname: String,

    #[clap(short = 's', long)]
    sentence_filename: String,

    #[clap(short = 'o', long)]
    output_filename: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let lines: Vec<_> = to_lines(&args.sentence_filename).collect();
    let output_filename = &args.output_filename;

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

    let mut sentence = Sentence::new();
    let mut lid_counts = vec![];
    let mut rid_counts = vec![];

    for line in lines {
        sentence.set_sentence(line);
        tokenizer.tokenize(&mut sentence);
        tokenizer.count_connid_occurrences(&mut lid_counts, &mut rid_counts);
    }
    assert_eq!(lid_counts[0], 0);
    assert_eq!(rid_counts[0], 0);

    let lid_sum = lid_counts.iter().fold(0, |acc, &x| acc + x) as f64;
    let rid_sum = rid_counts.iter().fold(0, |acc, &x| acc + x) as f64;

    let mut lid_probs: Vec<_> = lid_counts[1..]
        .iter()
        .enumerate()
        .map(|(i, &x)| (i + 1, x as f64 / lid_sum))
        .collect();
    let mut rid_probs: Vec<_> = rid_counts[1..]
        .iter()
        .enumerate()
        .map(|(i, &x)| (i + 1, x as f64 / rid_sum))
        .collect();

    lid_probs.sort_unstable_by(|(i1, f1), (i2, f2)| {
        f2.partial_cmp(f1).unwrap().then_with(|| i1.cmp(i2))
    });
    rid_probs.sort_unstable_by(|(i1, f1), (i2, f2)| {
        f2.partial_cmp(f1).unwrap().then_with(|| i1.cmp(i2))
    });

    {
        let mut w = BufWriter::new(File::create(format!("{}.lprobs", output_filename)).unwrap());
        for (i, p) in lid_probs {
            w.write(format!("{}\t{}\n", i, p).as_bytes())?;
        }
    }
    {
        let mut w = BufWriter::new(File::create(format!("{}.rprobs", output_filename)).unwrap());
        for (i, p) in rid_probs {
            w.write(format!("{}\t{}\n", i, p).as_bytes())?;
        }
    }

    Ok(())
}

fn to_lines<P>(path: P) -> impl Iterator<Item = String>
where
    P: AsRef<Path>,
{
    let buf = BufReader::new(File::open(path).unwrap());
    buf.lines().map(|line| line.unwrap())
}
