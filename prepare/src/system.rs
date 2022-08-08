use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use std::time::Instant;

use vibrato::dictionary::Dictionary;

use clap::Parser;

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

    let sysdic_filename = format!("{}/lex.csv", &args.resource_dirname);
    let matrix_filename = format!("{}/matrix.def", &args.resource_dirname);
    let chardef_filename = format!("{}/char.def", &args.resource_dirname);
    let unkdef_filename = format!("{}/unk.def", &args.resource_dirname);

    eprintln!("Compiling the system dictionary...");
    let start = Instant::now();
    let dict = Dictionary::from_reader(
        File::open(sysdic_filename)?,
        File::open(matrix_filename)?,
        File::open(chardef_filename)?,
        File::open(unkdef_filename)?,
    )?;
    eprintln!("{} seconds", start.elapsed().as_secs_f64());

    eprintln!(
        "Writting the system dictionary...: {}",
        &args.output_filename
    );
    let num_bytes = dict.write(BufWriter::new(File::create(args.output_filename)?))?;
    eprintln!("{} MiB", num_bytes as f64 / (1024. * 1024.));

    Ok(())
}
