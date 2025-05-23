use std::io::{BufWriter, Read, Write};
use std::num::NonZeroU32;

use bincode::{Decode, Encode};
use hashbrown::HashMap;

use crate::common;
use crate::dictionary::lexicon::Lexicon;
use crate::dictionary::word_idx::WordIdx;
use crate::dictionary::{LexType, WordParam};
use crate::errors::Result;
pub use crate::trainer::config::TrainerConfig;
use crate::trainer::corpus::Word;
pub use crate::trainer::Trainer;
use crate::utils::{self, FromU32};

#[derive(Decode, Encode)]
pub struct ModelData {
    pub config: TrainerConfig,
    pub raw_model: rucrf::RawModel,
}

/// Tokenization Model
pub struct Model {
    pub(crate) data: ModelData,

    // This field is not filled in by default for processing efficiency. The data is pre-computed
    // in `write_used_features()` and `write_dictionary()` and shared throughout the structure.
    pub(crate) merged_model: Option<rucrf::MergedModel>,

    pub(crate) user_entries: Vec<(Word, WordParam, NonZeroU32)>,
}

impl Model {
    /// Reads the user-defined lexicon file.
    ///
    /// If you want to assign parameters to the user-defined lexicon file, you need to call this
    /// function before exporting the dictionary. The model overwrites the parameter only when it
    /// is `0,0,0`. Otherwise, the parameter is used as is.
    ///
    /// # Arguments
    ///
    /// * `rdr` - Read sink of the user-defined lexicon file.
    ///
    /// # Errors
    ///
    /// [`VibratoError`](crate::errors::VibratoError) is returned when the reading fails.
    pub fn read_user_lexicon<R>(&mut self, mut rdr: R) -> Result<()>
    where
        R: Read,
    {
        let mut bytes = vec![];
        rdr.read_to_end(&mut bytes)?;

        self.merged_model = None;
        let entries = Lexicon::parse_csv(&bytes, "user.csv")?;
        for entry in entries {
            let first_char = entry.surface.chars().next().unwrap();
            let cate_id = self
                .data
                .config
                .dict
                .char_prop()
                .char_info(first_char)
                .base_id();
            let feature_set = Trainer::extract_feature_set(
                &mut self.data.config.feature_extractor,
                &self.data.config.unigram_rewriter,
                &self.data.config.left_rewriter,
                &self.data.config.right_rewriter,
                entry.feature,
                cate_id,
            );
            let label_id = self
                .data
                .raw_model
                .feature_provider()
                .add_feature_set(feature_set)?;

            self.user_entries.push((
                Word::new(&entry.surface, entry.feature),
                entry.param,
                label_id,
            ));
        }

        Ok(())
    }

    /// Write the relation between left/right connection IDs and features.
    ///
    /// # Arguments
    ///
    /// * `left_wtr` - Write sink targetting the `.left` file.
    /// * `right_wtr` - Write sink targetting the `.right` file.
    /// * `cost_wtr` - Write sink targetting the `.cost` file.
    ///
    /// # Errors
    ///
    /// [`VibratoError`](crate::errors::VibratoError) is returned when:
    ///
    /// - merging costs fails, or
    /// - the writing fails.
    pub fn write_bigram_details<L, R, C>(
        &mut self,
        left_wtr: L,
        right_wtr: R,
        cost_wtr: C,
    ) -> Result<()>
    where
        L: Write,
        R: Write,
        C: Write,
    {
        if self.merged_model.is_none() {
            self.merged_model = Some(self.data.raw_model.merge()?);
        }
        let merged_model = self.merged_model.as_ref().unwrap();

        // scales weights.
        let mut weight_abs_max = 0f64;
        for feature_set in &merged_model.feature_sets {
            weight_abs_max = weight_abs_max.max(feature_set.weight.abs());
        }
        for hm in &merged_model.matrix {
            for &w in hm.values() {
                weight_abs_max = weight_abs_max.max(w.abs());
            }
        }
        let weight_scale_factor = f64::from(i16::MAX) / weight_abs_max;

        let feature_extractor = &self.data.config.feature_extractor;

        // left
        let mut right_features = HashMap::new();
        for (feature, idx) in feature_extractor.right_feature_ids().iter() {
            right_features.insert(idx.get(), feature);
        }
        let feature_list = &merged_model.left_conn_to_right_feats;
        let mut left_wtr = BufWriter::new(left_wtr);
        for (conn_id, feat_ids) in feature_list[..feature_list.len()].iter().enumerate() {
            write!(&mut left_wtr, "{}\t", conn_id + 1)?;
            for (i, feat_id) in feat_ids.iter().enumerate() {
                if i != 0 {
                    write!(&mut left_wtr, ",")?;
                }
                if let Some(feat_id) = feat_id {
                    let feat_str = right_features.get(&feat_id.get()).unwrap();
                    utils::quote_csv_cell(&mut left_wtr, feat_str.as_bytes())?;
                } else {
                    write!(&mut left_wtr, "*")?;
                }
            }
            writeln!(&mut left_wtr)?;
        }

        // right
        let mut left_features = HashMap::new();
        for (feature, idx) in feature_extractor.left_feature_ids().iter() {
            left_features.insert(idx.get(), feature);
        }
        let feature_list = &merged_model.right_conn_to_left_feats;
        let mut right_wtr = BufWriter::new(right_wtr);
        for (conn_id, feat_ids) in feature_list[..feature_list.len()].iter().enumerate() {
            write!(&mut right_wtr, "{}\t", conn_id + 1)?;
            for (i, feat_id) in feat_ids.iter().enumerate() {
                if i != 0 {
                    write!(&mut right_wtr, ",")?;
                }
                if let Some(feat_id) = feat_id {
                    let feat_str = left_features.get(&feat_id.get()).unwrap();
                    utils::quote_csv_cell(&mut right_wtr, feat_str.as_bytes())?;
                } else {
                    write!(&mut right_wtr, "*")?;
                }
            }
            writeln!(&mut right_wtr)?;
        }

        let mut cost_wtr = BufWriter::new(cost_wtr);
        for (left_feat_id, hm) in self
            .data
            .raw_model
            .bigram_weight_indices()
            .iter()
            .enumerate()
        {
            let left_feat_str = left_features
                .get(&u32::try_from(left_feat_id).unwrap())
                .map_or("", |x| x.as_str());
            for (right_feat_id, widx) in hm {
                let right_feat_str = right_features.get(right_feat_id).map_or("", |x| x.as_str());
                let w = self.data.raw_model.weights()[usize::from_u32(*widx)];
                let cost = (-w * weight_scale_factor) as i32;
                writeln!(&mut cost_wtr, "{left_feat_str}/{right_feat_str}\t{cost}")?;
            }
        }
        Ok(())
    }

    /// Write the dictionary.
    ///
    /// # Arguments
    ///
    /// * `lexicon_wtr` - Write sink targetting `lex.csv`.
    /// * `connector_wtr` - Write sink targetting `matrix.def`.
    /// * `unk_handler_wtr` - Write sink targetting `unk.def`.
    /// * `user_lexicon_wtr` - Write sink targetting `user.csv`. Set a dummy argument if no user-defined
    ///   lexicon file is specified.
    ///
    /// # Errors
    ///
    /// [`VibratoError`](crate::errors::VibratoError) is returned when:
    ///
    /// - merging costs fails, or
    /// - the writing fails.
    pub fn write_dictionary<L, C, U, S>(
        &mut self,
        lexicon_wtr: L,
        connector_wtr: C,
        unk_handler_wtr: U,
        user_lexicon_wtr: S,
    ) -> Result<()>
    where
        L: Write,
        C: Write,
        U: Write,
        S: Write,
    {
        if self.merged_model.is_none() {
            self.merged_model = Some(self.data.raw_model.merge()?);
        }
        let merged_model = self.merged_model.as_ref().unwrap();

        let mut lexicon_wtr = BufWriter::new(lexicon_wtr);
        let mut unk_handler_wtr = BufWriter::new(unk_handler_wtr);
        let mut connector_wtr = BufWriter::new(connector_wtr);
        let mut user_lexicon_wtr = BufWriter::new(user_lexicon_wtr);

        // scales weights to represent them in i16.
        let mut weight_abs_max = 0f64;
        for feature_set in &merged_model.feature_sets {
            weight_abs_max = weight_abs_max.max(feature_set.weight.abs());
        }
        for hm in &merged_model.matrix {
            for &w in hm.values() {
                weight_abs_max = weight_abs_max.max(w.abs());
            }
        }
        let weight_scale_factor = f64::from(i16::MAX) / weight_abs_max;

        let config = &self.data.config;

        for i in 0..config.surfaces.len() {
            let feature_set = merged_model.feature_sets[i];
            let word_idx = WordIdx::new(LexType::System, u32::try_from(i).unwrap());
            let feature = config.dict.system_lexicon().word_feature(word_idx);

            // writes surface
            utils::quote_csv_cell(&mut lexicon_wtr, config.surfaces[i].as_bytes())?;

            // writes others
            writeln!(
                &mut lexicon_wtr,
                ",{},{},{},{}",
                feature_set.left_id,
                feature_set.right_id,
                (-feature_set.weight * weight_scale_factor) as i16,
                feature,
            )?;
        }

        for i in 0..config.dict.unk_handler().len() {
            let word_idx = WordIdx::new(LexType::Unknown, u32::try_from(i).unwrap());
            let cate_id = config.dict.unk_handler().word_cate_id(word_idx);
            let feature = config.dict.unk_handler().word_feature(word_idx);
            let cate_string = config
                .dict
                .char_prop()
                .cate_str(u32::from(cate_id))
                .unwrap();
            let feature_set = merged_model.feature_sets[config.surfaces.len() + i];
            writeln!(
                &mut unk_handler_wtr,
                "{},{},{},{},{}",
                cate_string,
                feature_set.left_id,
                feature_set.right_id,
                (-feature_set.weight * weight_scale_factor) as i16,
                feature,
            )?;
        }

        writeln!(
            &mut connector_wtr,
            "{} {}",
            merged_model.right_conn_to_left_feats.len() + 1,
            merged_model.left_conn_to_right_feats.len() + 1,
        )?;
        for (right_conn_id, hm) in merged_model.matrix.iter().enumerate() {
            let mut pairs: Vec<_> = hm.iter().map(|(&j, &w)| (j, w)).collect();
            pairs.sort_unstable_by_key(|&(k, _)| k);
            for (left_conn_id, w) in pairs {
                writeln!(
                    &mut connector_wtr,
                    "{} {} {}",
                    right_conn_id,
                    left_conn_id,
                    (-w * weight_scale_factor) as i16
                )?;
            }
        }

        for (word, param, label_id) in &self.user_entries {
            let feature_set = merged_model.feature_sets[usize::from_u32(label_id.get() - 1)];

            // writes surface
            utils::quote_csv_cell(&mut user_lexicon_wtr, word.surface().as_bytes())?;

            // writes others
            if *param == WordParam::default() {
                writeln!(
                    &mut user_lexicon_wtr,
                    ",{},{},{},{}",
                    feature_set.left_id,
                    feature_set.right_id,
                    (-feature_set.weight * weight_scale_factor) as i16,
                    word.feature(),
                )?;
            } else {
                writeln!(
                    &mut user_lexicon_wtr,
                    ",{},{},{},{}",
                    param.left_id,
                    param.right_id,
                    param.word_cost,
                    word.feature(),
                )?;
            }
        }

        Ok(())
    }

    /// Exports the model data.
    ///
    /// # Errors
    ///
    /// When bincode generates an error, it will be returned as is.
    pub fn write_model<W>(&self, mut wtr: W) -> Result<usize>
    where
        W: Write,
    {
        let num_bytes =
            bincode::encode_into_std_write(&self.data, &mut wtr, common::bincode_config())?;
        Ok(num_bytes)
    }

    /// Reads a model.
    ///
    /// # Errors
    ///
    /// When bincode generates an error, it will be returned as is.
    pub fn read_model<R>(mut rdr: R) -> Result<Self>
    where
        R: Read,
    {
        let data = bincode::decode_from_std_read(&mut rdr, common::bincode_config())?;
        Ok(Self {
            data,
            merged_model: None,
            user_entries: vec![],
        })
    }
}
