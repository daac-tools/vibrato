//! Module for training models.
//!
//! # Examples
//!
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use std::fs::File;
//! use vibrato::trainer::{Corpus, Trainer, TrainerConfig};
//! use vibrato::{Dictionary, Tokenizer};
//!
//! // Loads configurations
//! let lexicon_rdr = File::open("src/tests/resources/train_lex.csv")?;
//! let char_prop_rdr = File::open("src/tests/resources/char.def")?;
//! let unk_handler_rdr = File::open("src/tests/resources/train_unk.def")?;
//! let feature_templates_rdr = File::open("src/tests/resources/feature.def")?;
//! let rewrite_rules_rdr = File::open("src/tests/resources/rewrite.def")?;
//! let config = TrainerConfig::from_readers(
//!     lexicon_rdr,
//!     char_prop_rdr,
//!     unk_handler_rdr,
//!     feature_templates_rdr,
//!     rewrite_rules_rdr,
//! )?;
//!
//! // Initializes trainer
//! let trainer = Trainer::new(config)?
//!     .regularization_cost(0.01)
//!     .max_iter(300)
//!     .num_threads(20);
//!
//! // Loads corpus
//! let corpus_rdr = File::open("src/tests/resources/corpus.txt")?;
//! let corpus = Corpus::from_reader(corpus_rdr)?;
//!
//! // Model data
//! let mut lexicon_trained = vec![];
//! let mut connector_trained = vec![];
//! let mut unk_handler_trained = vec![];
//! let mut user_lexicon_trained = vec![];
//!
//! // Starts training
//! let mut model = trainer.train(corpus)?;
//!
//! model.write_dictionary(
//!     &mut lexicon_trained,
//!     &mut connector_trained,
//!     &mut unk_handler_trained,
//!     &mut user_lexicon_trained,
//! )?;
//!
//! // Loads trained model
//! let char_prop_rdr = File::open("src/tests/resources/char.def")?;
//! let dict = Dictionary::from_readers(
//!     &*lexicon_trained,
//!     &*connector_trained,
//!     char_prop_rdr,
//!     &*unk_handler_trained,
//! )?;
//!
//! let tokenizer = Tokenizer::new(dict);
//! let mut worker = tokenizer.new_worker();
//!
//! worker.reset_sentence("外国人参政権")?;
//! worker.tokenize();
//! assert_eq!(worker.num_tokens(), 4); // 外国/人/参政/権
//! # Ok(())
//! # }
//! ```

mod config;
mod corpus;
mod feature_extractor;
mod feature_rewriter;

use std::io::{BufWriter, Read, Write};
use std::num::NonZeroU32;

use bincode::{Decode, Encode};
use hashbrown::{HashMap, HashSet};
use rucrf::{Edge, FeatureProvider, FeatureSet, Lattice};

use crate::dictionary::lexicon::Lexicon;
use crate::dictionary::word_idx::WordIdx;
use crate::dictionary::{LexType, WordParam};
use crate::errors::Result;
pub use crate::trainer::config::TrainerConfig;
pub use crate::trainer::corpus::Corpus;
use crate::trainer::corpus::Example;
use crate::trainer::feature_extractor::FeatureExtractor;
use crate::trainer::feature_rewriter::FeatureRewriter;
use crate::utils::{self, FromU32};

use crate::common::{self, MAX_SENTENCE_LENGTH};

use self::corpus::Word;

/// Trainer of morphological analyzer.
pub struct Trainer {
    config: TrainerConfig,
    max_grouping_len: Option<u16>,
    provider: FeatureProvider,

    // Assume a dictionary word W is associated with id X and feature string F.
    // It maps F to a hash table that maps the first character of W to X.
    label_id_map: HashMap<String, HashMap<char, NonZeroU32>>,

    regularization_cost: f64,
    max_iter: u64,
    num_threads: usize,
}

impl Trainer {
    fn extract_feature_set(
        feature_extractor: &mut FeatureExtractor,
        unigram_rewriter: &FeatureRewriter,
        left_rewriter: &FeatureRewriter,
        right_rewriter: &FeatureRewriter,
        feature_str: &str,
        cate_id: u32,
    ) -> FeatureSet {
        let features = utils::parse_csv_row(feature_str);
        let unigram_features = if let Some(rewrite) = unigram_rewriter.rewrite(&features) {
            feature_extractor.extract_unigram_feature_ids(&rewrite, cate_id)
        } else {
            feature_extractor.extract_unigram_feature_ids(&features, cate_id)
        };
        let left_features = if let Some(rewrite) = left_rewriter.rewrite(&features) {
            feature_extractor.extract_left_feature_ids(&rewrite)
        } else {
            feature_extractor.extract_left_feature_ids(&features)
        };
        let right_features = if let Some(rewrite) = right_rewriter.rewrite(&features) {
            feature_extractor.extract_right_feature_ids(&rewrite)
        } else {
            feature_extractor.extract_right_feature_ids(&features)
        };
        FeatureSet::new(&unigram_features, &right_features, &left_features)
    }

    /// Creates a new [`Trainer`] using the specified configuration.
    ///
    /// # Arguments
    ///
    ///  * `config` - Training configuration.
    ///
    /// # Errors
    ///
    /// [`VibratoError`](crate::errors::VibratoError) is returned when the model will become too large.
    pub fn new(mut config: TrainerConfig) -> Result<Self> {
        let mut provider = FeatureProvider::default();
        let mut label_id_map = HashMap::new();
        for word_id in 0..u32::try_from(config.surfaces.len()).unwrap() {
            let word_idx = WordIdx::new(LexType::System, word_id);
            let feature_str = config.dict.system_lexicon().word_feature(word_idx);
            let first_char = config.surfaces[usize::from_u32(word_id)]
                .chars()
                .next()
                .unwrap();
            let cate_id = config.dict.char_prop().char_info(first_char).base_id();
            let feature_set = Self::extract_feature_set(
                &mut config.feature_extractor,
                &config.unigram_rewriter,
                &config.left_rewriter,
                &config.right_rewriter,
                feature_str,
                cate_id,
            );
            let label_id = provider.add_feature_set(feature_set)?;
            label_id_map
                .raw_entry_mut()
                .from_key(feature_str)
                .or_insert_with(|| (feature_str.to_string(), HashMap::new()))
                .1
                .insert(first_char, label_id);
        }
        for word_id in 0..u32::try_from(config.dict.unk_handler().len()).unwrap() {
            let word_idx = WordIdx::new(LexType::Unknown, word_id);
            let feature_str = config.dict.unk_handler().word_feature(word_idx);
            let cate_id = u32::from(config.dict.unk_handler().word_cate_id(word_idx));
            let feature_set = Self::extract_feature_set(
                &mut config.feature_extractor,
                &config.unigram_rewriter,
                &config.left_rewriter,
                &config.right_rewriter,
                feature_str,
                cate_id,
            );
            provider.add_feature_set(feature_set)?;
        }

        Ok(Self {
            config,
            max_grouping_len: None,
            provider,
            label_id_map,
            regularization_cost: 0.01,
            max_iter: 100,
            num_threads: 1,
        })
    }

    /// Changes the cost of L1-regularization.
    ///
    /// The greater this value, the stronger the regularization.
    /// Default to 0.01.
    ///
    /// # Panics
    ///
    /// The value must be greater than or equal to 0.
    pub fn regularization_cost(mut self, cost: f64) -> Self {
        assert!(cost >= 0.0);
        self.regularization_cost = cost;
        self
    }

    /// Changes the maximum number of iterations.
    ///
    /// Default to 100.
    ///
    /// # Panics
    ///
    /// The value must be positive.
    pub fn max_iter(mut self, n: u64) -> Self {
        assert!(n >= 1);
        self.max_iter = n;
        self
    }

    /// Enables multi-threading.
    ///
    /// Default to 1.
    ///
    /// # Panics
    ///
    /// The value must be positive.
    pub fn num_threads(mut self, n: usize) -> Self {
        assert!(n >= 1);
        self.num_threads = n;
        self
    }

    /// Specifies the maximum grouping length for unknown words.
    /// By default, the length is infinity.
    ///
    /// This option is for compatibility with MeCab.
    /// Specifies the argument with `24` if you want to obtain the same results as MeCab.
    ///
    /// # Arguments
    ///
    ///  * `max_grouping_len` - The maximum grouping length for unknown words.
    ///                         The default value is 0, indicating the infinity length.
    pub fn max_grouping_len(mut self, max_grouping_len: usize) -> Self {
        if max_grouping_len != 0 && max_grouping_len <= usize::from(MAX_SENTENCE_LENGTH) {
            self.max_grouping_len = Some(max_grouping_len as u16);
        } else {
            self.max_grouping_len = None;
        }
        self
    }

    fn build_lattice(&self, example: &Example) -> Lattice {
        let Example { sentence, tokens } = example;

        let input_chars = sentence.chars();
        let input_len = sentence.len_char();

        let virtual_edge_label =
            NonZeroU32::new(u32::try_from(self.provider.len()).unwrap() + 1).unwrap();
        let unk_label_offset =
            NonZeroU32::new(u32::try_from(self.config.surfaces.len() + 1).unwrap()).unwrap();

        // Add positive edges
        // 1. If the word is found in the dictionary, add the edge as it is.
        // 2. If the word is not found in the dictionary:
        //   a) If a compatible unknown word is found, add the unknown word edge instead.
        //   b) If there is no available word, add a virtual edge, which does not have any features.
        let mut edges = vec![];
        let mut pos = 0;
        for token in tokens {
            let len = token.surface().chars().count();
            let first_char = input_chars[pos];
            let label_id = self
                .label_id_map
                .get(token.feature())
                .and_then(|hm| hm.get(&first_char))
                .cloned()
                .unwrap_or_else(|| {
                    self.config
                        .dict
                        .unk_handler()
                        .compatible_unk_index(
                            sentence,
                            u16::try_from(pos).unwrap(),
                            u16::try_from(pos + len).unwrap(),
                            token.feature(),
                        )
                        .map_or_else(
                            || {
                                eprintln!(
                                    "adding virtual edge: {} {}",
                                    token.surface(),
                                    token.feature()
                                );
                                virtual_edge_label
                            },
                            |unk_index| {
                                NonZeroU32::new(unk_label_offset.get() + unk_index.word_id).unwrap()
                            },
                        )
                });
            edges.push((pos, Edge::new(pos + len, label_id)));
            pos += len;
        }
        assert_eq!(pos, usize::from(input_len));

        let mut lattice = Lattice::new(usize::from(input_len)).unwrap();

        for (pos, edge) in edges {
            lattice.add_edge(pos, edge).unwrap();
        }

        // Add negative edges
        for start_word in 0..input_len {
            let mut has_matched = false;

            let suffix = &input_chars[usize::from(start_word)..];

            for m in self
                .config
                .dict
                .system_lexicon()
                .common_prefix_iterator(suffix)
            {
                has_matched = true;
                let label_id = NonZeroU32::new(m.word_idx.word_id + 1).unwrap();
                let pos = usize::from(start_word);
                let target = pos + usize::from(m.end_char);
                let edge = Edge::new(target, label_id);
                // Skips adding if the edge is already added as a positive edge.
                if let Some(first_edge) = lattice.nodes()[pos].edges().first() {
                    if edge == *first_edge {
                        continue;
                    }
                }
                lattice.add_edge(pos, edge).unwrap();
            }

            self.config.dict.unk_handler().gen_unk_words(
                sentence,
                start_word,
                has_matched,
                self.max_grouping_len,
                |w| {
                    let id_offset = u32::try_from(self.config.surfaces.len()).unwrap();
                    let label_id = NonZeroU32::new(id_offset + w.word_idx().word_id + 1).unwrap();
                    let pos = usize::from(start_word);
                    let target = usize::from(w.end_char());
                    let edge = Edge::new(target, label_id);
                    // Skips adding if the edge is already added as a positive edge.
                    if let Some(first_edge) = lattice.nodes()[pos].edges().first() {
                        if edge == *first_edge {
                            return;
                        }
                    }
                    lattice.add_edge(pos, edge).unwrap();
                },
            );
        }

        lattice
    }

    /// Starts training and returns a model.
    ///
    /// # Arguments
    ///
    /// * `corpus` - Corpus used for training.
    ///
    /// # Errors
    ///
    /// [`VibratoError`](crate::errors::VibratoError) is returned when the sentence compilation
    /// fails.
    pub fn train(mut self, mut corpus: Corpus) -> Result<Model> {
        let mut lattices = vec![];
        for example in &mut corpus.examples {
            example.sentence.compile(self.config.dict.char_prop())?;
            lattices.push(self.build_lattice(example));
        }

        let trainer = rucrf::Trainer::new()
            .regularization(rucrf::Regularization::L1, self.regularization_cost)
            .unwrap()
            .max_iter(self.max_iter)
            .unwrap()
            .n_threads(self.num_threads)
            .unwrap();
        let model = trainer.train(&lattices, self.provider);

        // Remove unused feature strings
        let mut used_right_features = HashSet::new();
        let unigram_feature_keys: Vec<_> = self
            .config
            .feature_extractor
            .unigram_feature_ids
            .keys()
            .cloned()
            .collect();
        let left_feature_keys: Vec<_> = self
            .config
            .feature_extractor
            .left_feature_ids
            .keys()
            .cloned()
            .collect();
        let right_feature_keys: Vec<_> = self
            .config
            .feature_extractor
            .right_feature_ids
            .keys()
            .cloned()
            .collect();
        for k in &unigram_feature_keys {
            let id = self
                .config
                .feature_extractor
                .unigram_feature_ids
                .get(k)
                .unwrap();
            if model.unigram_weight_indices()[usize::from_u32(id.get() - 1)].is_none() {
                self.config.feature_extractor.unigram_feature_ids.remove(k);
            }
        }
        for feature_ids in model.bigram_weight_indices() {
            for (feature_id, _) in feature_ids {
                used_right_features.insert(*feature_id);
            }
        }
        for k in &left_feature_keys {
            let id = self
                .config
                .feature_extractor
                .left_feature_ids
                .get(k)
                .unwrap();
            if model.bigram_weight_indices()[usize::from_u32(id.get())].is_empty() {
                self.config.feature_extractor.left_feature_ids.remove(k);
            }
        }
        for k in &right_feature_keys {
            let id = self
                .config
                .feature_extractor
                .right_feature_ids
                .get(k)
                .unwrap();
            if !used_right_features.contains(&id.get()) {
                self.config.feature_extractor.right_feature_ids.remove(k);
            }
        }

        Ok(Model {
            data: ModelData {
                config: self.config,
                raw_model: model,
            },
            merged_model: None,
            user_entries: vec![],
        })
    }
}

#[derive(Decode, Encode)]
struct ModelData {
    config: TrainerConfig,
    raw_model: rucrf::RawModel,
}

/// Tokenization Model
pub struct Model {
    data: ModelData,

    // This field is not filled in by default for processing efficiency. The data is pre-computed
    // in `write_used_features()` and `write_dictionary()` and shared throughout the structure.
    merged_model: Option<rucrf::MergedModel>,

    user_entries: Vec<(Word, WordParam)>,
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
            self.data
                .raw_model
                .feature_provider()
                .add_feature_set(feature_set)?;

            self.user_entries
                .push((Word::new(&entry.surface, entry.feature), entry.param));
        }

        Ok(())
    }

    /// Write the relation between left/right connection IDs and features.
    ///
    /// # Arguments
    ///
    /// * `left_wtr` - Write sink targetting `left.def`.
    /// * `right_wtr` - Write sink targetting `right.def`.
    ///
    /// # Errors
    ///
    /// [`VibratoError`](crate::errors::VibratoError) is returned when:
    ///
    /// - merging weights fails, or
    /// - the writing fails.
    pub fn write_used_features<L, R>(&mut self, left_wtr: L, right_wtr: R) -> Result<()>
    where
        L: Write,
        R: Write,
    {
        if self.merged_model.is_none() {
            self.merged_model = Some(self.data.raw_model.merge()?);
        }
        let merged_model = self.merged_model.as_ref().unwrap();

        // left
        let mut left_wtr = BufWriter::new(left_wtr);
        let mut left_features: Vec<_> = self
            .data
            .config
            .feature_extractor
            .left_feature_ids()
            .iter()
            .collect();
        left_features.sort_unstable_by_key(|(_, v)| **v);
        let feature_list = &merged_model.left_conn_to_right_feats;
        for (conn_id, feat_ids) in feature_list[..feature_list.len() - 1].iter().enumerate() {
            write!(&mut left_wtr, "{}", conn_id + 1)?;
            for (i, feat_id) in feat_ids.iter().enumerate() {
                if let Some(feat_id) = feat_id {
                    let feat_str = &left_features[usize::from_u32(feat_id.get()) - 1].0;
                    write!(&mut left_wtr, " {i}:{feat_str}")?;
                }
            }
            writeln!(&mut left_wtr)?;
        }

        // right
        let mut right_wtr = BufWriter::new(right_wtr);
        let mut right_features: Vec<_> = self
            .data
            .config
            .feature_extractor
            .right_feature_ids()
            .iter()
            .collect();
        right_features.sort_unstable_by_key(|(_, v)| **v);
        let feature_list = &merged_model.right_conn_to_left_feats;
        for (conn_id, feat_ids) in feature_list[..feature_list.len() - 1].iter().enumerate() {
            write!(&mut right_wtr, "{}", conn_id + 1)?;
            for (i, feat_id) in feat_ids.iter().enumerate() {
                if let Some(feat_id) = feat_id {
                    let feat_str = &right_features[usize::from_u32(feat_id.get()) - 1].0;
                    write!(&mut right_wtr, " {i}:{feat_str}")?;
                }
            }
            writeln!(&mut right_wtr)?;
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
    ///                        lexicon file is specified.
    ///
    /// # Errors
    ///
    /// [`VibratoError`](crate::errors::VibratoError) is returned when:
    ///
    /// - merging weights fails, or
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

        let mut output = [0; 4096];

        // scales weights to represent them in i16.
        let mut weight_abs_max = 0f64;
        for feature_set in &merged_model.feature_sets {
            weight_abs_max = weight_abs_max.max(feature_set.weight.abs());
        }
        for hm in &merged_model.matrix {
            for &w in hm.values() {
                weight_abs_max = weight_abs_max.max(w);
            }
        }
        let weight_scale_factor = f64::from(i16::MAX) / weight_abs_max;

        for i in 0..self.data.config.surfaces.len() {
            let mut writer = csv_core::Writer::new();
            let mut surface = self.data.config.surfaces[i].as_bytes();
            let feature_set = merged_model.feature_sets[i];
            let word_idx = WordIdx::new(LexType::System, u32::try_from(i).unwrap());
            let feature = self
                .data
                .config
                .dict
                .system_lexicon()
                .word_feature(word_idx);

            // writes surface
            loop {
                let (result, nin, nout) = writer.field(surface, &mut output);
                lexicon_wtr.write_all(&output[..nout])?;
                if result == csv_core::WriteResult::InputEmpty {
                    break;
                }
                surface = &surface[nin..];
            }
            let (result, nout) = writer.finish(&mut output);
            assert_eq!(result, csv_core::WriteResult::InputEmpty);
            lexicon_wtr.write_all(&output[..nout])?;

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

        for i in 0..self.data.config.dict.unk_handler().len() {
            let word_idx = WordIdx::new(LexType::Unknown, u32::try_from(i).unwrap());
            let cate_id = self.data.config.dict.unk_handler().word_cate_id(word_idx);
            let feature = self.data.config.dict.unk_handler().word_feature(word_idx);
            let cate_string = self
                .data
                .config
                .dict
                .char_prop()
                .cate_str(u32::from(cate_id))
                .unwrap();
            let feature_set = merged_model.feature_sets[self.data.config.surfaces.len() + i];
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

        for (i, (word, param)) in self.user_entries.iter().enumerate() {
            let mut writer = csv_core::Writer::new();
            let feature_set = merged_model.feature_sets
                [self.data.config.surfaces.len() + self.data.config.dict.unk_handler().len() + i];

            // writes surface
            let mut surface = word.surface().as_bytes();
            loop {
                let (result, nin, nout) = writer.field(surface, &mut output);
                user_lexicon_wtr.write_all(&output[..nout])?;
                if result == csv_core::WriteResult::InputEmpty {
                    break;
                }
                surface = &surface[nin..];
            }
            let (result, nout) = writer.finish(&mut output);
            assert_eq!(result, csv_core::WriteResult::InputEmpty);
            user_lexicon_wtr.write_all(&output[..nout])?;

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
