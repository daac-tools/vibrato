use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufWriter, Write};

use tinylattice::dictionary::{CharProperty, Connector, Dictionary, LexType, Lexicon, UnkHandler};
use tinylattice::{Sentence, Tokenizer};

use clap::Parser;

type Probs = Vec<(usize, f64)>;

#[derive(Parser, Debug)]
#[clap(name = "main", about = "A program.")]
struct Args {
    #[clap(short = 'r', long)]
    resource_dirname: String,

    #[clap(short = 'o', long)]
    output_filename: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

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
    let mut lid_to_rid_occ = tokenizer.new_connid_occ();

    for line in std::io::stdin().lock().lines() {
        let line = line?;
        sentence.set_sentence(line);
        tokenizer.tokenize(&mut sentence);
        tokenizer.count_connid_occ(&mut lid_to_rid_occ);
    }
    let (lid_probs, rid_probs) = compute_probs(&lid_to_rid_occ);

    {
        let mut w = BufWriter::new(File::create(format!("{}.lmap", output_filename)).unwrap());
        for (i, p) in lid_probs {
            w.write_all(format!("{}\t{}\n", i, p).as_bytes())?;
        }
    }
    {
        let mut w = BufWriter::new(File::create(format!("{}.rmap", output_filename)).unwrap());
        for (i, p) in rid_probs {
            w.write_all(format!("{}\t{}\n", i, p).as_bytes())?;
        }
    }

    Ok(())
}

fn compute_probs<V>(lid_to_rid_count: &[V]) -> (Probs, Probs)
where
    V: AsRef<[usize]>,
{
    let num_left = lid_to_rid_count.len();
    let num_right = lid_to_rid_count[0].as_ref().len();

    // Compute Left-id probs
    let mut lid_probs = Vec::with_capacity(num_left);
    let mut lid_to_rid_probs = Vec::with_capacity(num_left);

    for (lid, rid_count) in lid_to_rid_count.iter().enumerate() {
        let rid_count = rid_count.as_ref();
        assert_eq!(num_right, rid_count.len());

        let acc = rid_count.iter().sum::<usize>() as f64;
        let mut probs = vec![0.0; num_right];
        if acc != 0.0 {
            for (rid, &cnt) in rid_count.iter().enumerate() {
                probs[rid] = cnt as f64 / acc;
            }
        }
        lid_probs.push((lid, acc)); // ittan acc wo push suru
        lid_to_rid_probs.push(probs);
    }

    let acc = lid_probs.iter().fold(0., |acc, &(_, cnt)| acc + cnt);
    for (_, lp) in lid_probs.iter_mut() {
        *lp /= acc;
    }

    // Compute Right-id probs
    let mut rid_probs = vec![(0, 0.0); num_right];
    for (i, (rid, rp)) in rid_probs.iter_mut().enumerate() {
        *rid = i;
        for lid in 0..num_left {
            assert_eq!(lid, lid_probs[lid].0);
            *rp += lid_probs[lid].1 * lid_to_rid_probs[lid][*rid];
        }
    }

    // Pop Id = 0
    lid_probs.drain(..1);
    rid_probs.drain(..1);

    // Sort
    lid_probs.sort_unstable_by(|(i1, p1), (i2, p2)| {
        p2.partial_cmp(p1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| i1.cmp(i2))
    });
    rid_probs.sort_unstable_by(|(i1, p1), (i2, p2)| {
        p2.partial_cmp(p1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| i1.cmp(i2))
    });

    (lid_probs, rid_probs)
}
