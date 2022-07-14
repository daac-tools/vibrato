use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "main", about = "A program.")]
struct Args {
    #[clap(short = 'd', long)]
    sysdic_filename: String,

    #[clap(short = 'm', long)]
    matrix_filename: String,

    #[clap(short = 'c', long)]
    chardef_filename: String,

    #[clap(short = 'u', long)]
    unkdef_filename: String,
}

fn main() {
    let args = Args::parse();

    // let config = Config::new(
    //     None,
    //     args.resources_filename.map(|s| PathBuf::from(s)),
    //     Some(PathBuf::from(&args.dict_filename)),
    // )
    // .unwrap();
    // let lines = load_file(&args.sentence_filename);

    // let dict = JapaneseDictionary::from_cfg(&config).unwrap();
    // let mut tokenizer = StatefulTokenizer::new(&dict, Mode::C);
    // tokenizer.set_subset(InfoSubset::empty());
    // let mut morphemes = MorphemeList::empty(&dict);

    // for line in &lines {
    //     tokenizer.reset().push_str(line);
    //     tokenizer.do_tokenize().unwrap();
    //     morphemes.collect_results(&mut tokenizer).unwrap();

    //     let tokenized: Vec<_> = morphemes.iter().map(|m| m.surface().to_string()).collect();
    //     println!("{}", tokenized.join(" "));
    // }
}

fn load_file<P>(path: P) -> Vec<String>
where
    P: AsRef<Path>,
{
    let file = File::open(path).unwrap();
    let buf = BufReader::new(file);
    buf.lines().map(|line| line.unwrap()).collect()
}
