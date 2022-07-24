use std::error::Error;
use std::fs::File;
use std::io::BufRead;

use tinylattice::dictionary::{
    CharProperty, ConnIdMapper, Connector, Dictionary, LexType, Lexicon, UnkHandler,
};
use tinylattice::{Sentence, Tokenizer};

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "main", about = "A program.")]
struct Args {
    #[clap(short = 'r', long)]
    resource_dirname: String,

    #[clap(short = 'm', long)]
    mapping_basename: Option<String>,

    #[clap(short = 'w', long)]
    wakachi: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let wakachi = args.wakachi;

    let sysdic_filename = format!("{}/lex.csv", &args.resource_dirname);
    let matrix_filename = format!("{}/matrix.def", &args.resource_dirname);
    let chardef_filename = format!("{}/char.def", &args.resource_dirname);
    let unkdef_filename = format!("{}/unk.def", &args.resource_dirname);

    let mut dict = Dictionary::new(
        Lexicon::from_reader(File::open(sysdic_filename)?, LexType::System)?,
        Connector::from_reader(File::open(matrix_filename)?)?,
        CharProperty::from_reader(File::open(chardef_filename)?)?,
        UnkHandler::from_reader(File::open(unkdef_filename)?)?,
    );

    if let Some(mapping_basename) = args.mapping_basename {
        let l_filename = format!("{}.lmap", mapping_basename);
        let r_filename = format!("{}.rmap", mapping_basename);
        let mapper = ConnIdMapper::from_reader(File::open(l_filename)?, File::open(r_filename)?)?;
        dict.map_ids(&mapper);
    }

    let mut tokenizer = Tokenizer::new(&dict);
    let mut sentence = Sentence::new();

    for line in std::io::stdin().lock().lines() {
        let line = line?;

        sentence.set_sentence(line);
        tokenizer.tokenize(&mut sentence);
        let morphs = sentence.morphs();

        if wakachi {
            let surfaces = sentence.surfaces();
            println!("{}", surfaces.join(" "));
        } else {
            for m in morphs {
                match m.word_idx().lex_type() {
                    LexType::System => {
                        println!(
                            "{}\t{}\t{}\t{}",
                            sentence.surface(m),
                            tokenizer.feature(m),
                            m.word_idx().word_id(),
                            m.total_cost()
                        )
                    }
                    LexType::Unknown => {
                        println!(
                            "{}\t{}\t{}\t{} (UNK)",
                            sentence.surface(m),
                            tokenizer.feature(m),
                            m.word_idx().word_id(),
                            m.total_cost()
                        )
                    }
                }
            }
            println!("EOS");
        }
    }

    Ok(())
}
