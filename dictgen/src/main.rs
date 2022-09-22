use std::ffi::OsStr;
use std::fs::File;
use std::path::PathBuf;

use clap::Parser;
use vibrato::trainer::Model;

#[derive(Parser, Debug)]
#[clap(name = "dictgen", about = "Dictionary generator")]
struct Args {
    /// Model file generated by the train command.
    #[clap(short = 'i', long)]
    model_in: PathBuf,

    /// A file to which the system lexicon is output (lex.csv).
    #[clap(short = 'l', long)]
    lexicon_out: PathBuf,

    /// A file to which the unknown word definition is output (unk.def).
    #[clap(short = 'u', long)]
    unk_out: PathBuf,

    /// A file to which the matrix is output (matrix.def).
    #[clap(short = 'm', long)]
    matrix_out: PathBuf,

    /// User-defined lexicon file. For which you want to give weights automatically,
    /// set 0 for the connection ID and the weight of such entries.
    #[clap(long)]
    user_lexicon_in: Option<PathBuf>,

    /// A file to which the user-defined lexicon is output.
    #[clap(long)]
    user_lexicon_out: Option<PathBuf>,

    /// Outputs a list of features associated with each left connection ID.
    ///
    /// The file name is suffixed with `.left` and `.right`.
    #[clap(long)]
    conn_id_info_out: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let model_rdr = ruzstd::StreamingDecoder::new(File::open(args.model_in)?)?;

    let mut model = Model::read_model(model_rdr)?;

    if let Some(path) = args.user_lexicon_in {
        let rdr = File::open(path)?;
        model.read_user_lexicon(rdr)?;
    }

    let lexicon_wtr = File::create(args.lexicon_out)?;
    let connector_wtr = File::create(args.matrix_out)?;
    let unk_handler_wtr = File::create(args.unk_out)?;

    if let Some(path) = args.user_lexicon_out {
        let user_lexicon_wtr = File::create(path)?;
        model.write_dictionary(
            lexicon_wtr,
            connector_wtr,
            unk_handler_wtr,
            user_lexicon_wtr,
        )?;
    } else {
        model.write_dictionary(lexicon_wtr, connector_wtr, unk_handler_wtr, vec![])?;
    }

    if let Some(path) = args.conn_id_info_out {
        let ext = path
            .extension()
            .unwrap_or_else(|| OsStr::new(""))
            .to_os_string();
        let mut left_ext = ext.clone();
        let mut right_ext = ext;
        left_ext.push(".left");
        right_ext.push(".right");
        let mut left_path = path.clone();
        let mut right_path = path;
        left_path.set_extension(left_ext);
        right_path.set_extension(right_ext);
        let left_wtr = File::create(left_path)?;
        let right_wtr = File::create(right_path)?;
        model.write_used_features(left_wtr, right_wtr)?;
    }

    Ok(())
}
