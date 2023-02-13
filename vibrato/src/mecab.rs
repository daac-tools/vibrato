//! Utilities to support MeCab models.

use std::io::{BufRead, BufReader, BufWriter, Read, Write};

use hashbrown::HashMap;
use regex::Regex;

use crate::errors::{Result, VibratoError};
use crate::trainer::TrainerConfig;
use crate::utils;

/// Generates bi-gram feature information from MeCab model.
///
/// This function is useful to create a small dictionary from an existing MeCab model.
///
/// # Arguments
///
/// * `feature_def_rdr` - A reader of the feature definition file `feature.def`.
/// * `left_id_def_rdr` - A reader of the left-id and feature mapping file `left-id.def`.
/// * `right_id_def_rdr` - A reader of the right-id and feature mapping file `right-id.def`
/// * `model_def_rdr` - A reader of the model file `model.def`.
/// * `cost_factor` - A factor to be multiplied when casting costs to integers.
/// * `bigram_left_wtr` - A writer of the left-id and feature mapping file `bi-gram.left`.
/// * `bigram_right_wtr` - A writer of the right-id and feature mapping file `bi-gram.right`.
/// * `bigram_cost_wtr` - A writer of the bi-gram cost file `bi-gram.cost`.
///
/// # Errors
///
/// [`VibratoError`] is returned when the convertion failed.
#[allow(clippy::too_many_arguments)]
pub fn generate_bigram_info(
    feature_def_rdr: impl Read,
    right_id_def_rdr: impl Read,
    left_id_def_rdr: impl Read,
    model_def_rdr: impl Read,
    cost_factor: f64,
    bigram_right_wtr: impl Write,
    bigram_left_wtr: impl Write,
    bigram_cost_wtr: impl Write,
) -> Result<()> {
    let mut left_features = HashMap::new();
    let mut right_features = HashMap::new();

    let mut feature_extractor = TrainerConfig::parse_feature_config(feature_def_rdr)?;

    let id_feature_re = Regex::new(r"^([0-9]+) (.*)$").unwrap();
    let model_re = Regex::new(r"^([0-9\-\.]+)\t(.*)$").unwrap();

    // right-id.def contains the right hand ID of the left context, and left-id.def contains the
    // left hand ID of the right context. The left-right naming in this code is based on position
    // between words, so these names are the opposite of left-id.def and right-id.def files.

    // left features
    let right_id_def_rdr = BufReader::new(right_id_def_rdr);
    for line in right_id_def_rdr.lines() {
        let line = line?;
        if let Some(cap) = id_feature_re.captures(&line) {
            let id = cap.get(1).unwrap().as_str().parse::<usize>()?;
            let feature_str = cap.get(2).unwrap().as_str();
            let feature_spl = utils::parse_csv_row(feature_str);
            if id == 0 {
                if feature_spl.get(0).map_or(false, |s| s != "BOS/EOS") {
                    return Err(VibratoError::invalid_format(
                        "right_id_def_rdr",
                        "ID 0 must be BOS/EOS",
                    ));
                }
            }
            let feature_ids = feature_extractor.extract_left_feature_ids(&feature_spl);
            left_features.insert(id, feature_ids);
        } else {
            return Err(VibratoError::invalid_format(
                "right_id_def_rdr",
                "each line must be a pair of an ID and features",
            ));
        }
    }
    // right features
    let left_id_def_rdr = BufReader::new(left_id_def_rdr);
    for line in left_id_def_rdr.lines() {
        let line = line?;
        if let Some(cap) = id_feature_re.captures(&line) {
            let id = cap.get(1).unwrap().as_str().parse::<usize>()?;
            let feature_str = cap.get(2).unwrap().as_str();
            let feature_spl = utils::parse_csv_row(feature_str);
            if id == 0 {
                if feature_spl.get(0).map_or(false, |s| s != "BOS/EOS") {
                    return Err(VibratoError::invalid_format(
                        "left_id_def_rdr",
                        "ID 0 must be BOS/EOS",
                    ));
                }
            }
            let feature_ids = feature_extractor.extract_right_feature_ids(&feature_spl);
            right_features.insert(id, feature_ids);
        } else {
            return Err(VibratoError::invalid_format(
                "left_id_def_rdr",
                "each line must be a pair of an ID and features",
            ));
        }
    }
    // weights
    let model_def_rdr = BufReader::new(model_def_rdr);
    let mut bigram_cost_wtr = BufWriter::new(bigram_cost_wtr);
    for line in model_def_rdr.lines() {
        let line = line?;
        if let Some(cap) = model_re.captures(&line) {
            let weight = cap.get(1).unwrap().as_str().parse::<f64>()?;
            let cost = -(weight * cost_factor) as i32;
            if cost == 0 {
                continue;
            }
            let feature_str = cap.get(2).unwrap().as_str().replace("BOS/EOS", "");
            let mut spl = feature_str.split('/');
            let left_feat_str = spl.next();
            let right_feat_str = spl.next();
            if let (Some(left_feat_str), Some(right_feat_str)) = (left_feat_str, right_feat_str) {
                let left_id = if left_feat_str.is_empty() {
                    String::new()
                } else if let Some(id) = feature_extractor.left_feature_ids().get(left_feat_str) {
                    id.to_string()
                } else {
                    continue;
                };
                let right_id = if right_feat_str.is_empty() {
                    String::new()
                } else if let Some(id) = feature_extractor.right_feature_ids().get(right_feat_str) {
                    id.to_string()
                } else {
                    continue;
                };
                writeln!(&mut bigram_cost_wtr, "{left_id}/{right_id}\t{cost}")?;
            }
        }
    }

    let mut bigram_right_wtr = BufWriter::new(bigram_right_wtr);
    for id in 1..left_features.len() {
        write!(&mut bigram_right_wtr, "{id}\t")?;
        if let Some(features) = left_features.get(&id) {
            for (i, feat_id) in features.iter().enumerate() {
                if i != 0 {
                    write!(&mut bigram_right_wtr, ",")?;
                }
                if let Some(feat_id) = feat_id {
                    write!(&mut bigram_right_wtr, "{}", feat_id.get())?;
                } else {
                    write!(&mut bigram_right_wtr, "*")?;
                }
            }
        } else {
            return Err(VibratoError::invalid_format(
                "right_id_def_rdr",
                format!("feature ID {id} is undefined"),
            ));
        }
        writeln!(&mut bigram_right_wtr)?;
    }

    let mut bigram_left_wtr = BufWriter::new(bigram_left_wtr);
    for id in 1..right_features.len() {
        write!(&mut bigram_left_wtr, "{id}\t")?;
        if let Some(features) = right_features.get(&id) {
            for (i, feat_id) in features.iter().enumerate() {
                if i != 0 {
                    write!(&mut bigram_left_wtr, ",")?;
                }
                if let Some(feat_id) = feat_id {
                    write!(&mut bigram_left_wtr, "{}", feat_id.get())?;
                } else {
                    write!(&mut bigram_left_wtr, "*")?;
                }
            }
            writeln!(&mut bigram_left_wtr)?;
        } else {
            return Err(VibratoError::invalid_format(
                "left_id_def_rdr",
                format!("feature ID {id} is undefined"),
            ));
        }
    }

    Ok(())
}
