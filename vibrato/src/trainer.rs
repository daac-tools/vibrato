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
//! // Files to store results
//! let mut trained_lex_path = std::env::temp_dir();
//! let mut trained_matrix_path = std::env::temp_dir();
//! let mut trained_unk_path = std::env::temp_dir();
//! trained_lex_path.push("trained_lex.csv");
//! trained_matrix_path.push("trained_matrix.def");
//! trained_unk_path.push("trained_unk.def");
//!
//! let lexicon_wtr = File::create(&trained_lex_path)?;
//! let connector_wtr = File::create(&trained_matrix_path)?;
//! let unk_handler_wtr = File::create(&trained_unk_path)?;
//!
//! // Starts training
//! trainer.train(corpus, lexicon_wtr, connector_wtr, unk_handler_wtr)?;
//!
//! // Loads trained model
//! let lexicon_rdr = File::open(&trained_lex_path)?;
//! let connector_rdr = File::open(&trained_matrix_path)?;
//! let char_prop_rdr = File::open("src/tests/resources/char.def")?;
//! let unk_handler_rdr = File::open(&trained_unk_path)?;
//! let dict =
//!     Dictionary::from_readers(lexicon_rdr, connector_rdr, char_prop_rdr, unk_handler_rdr)?;
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

use std::io::{BufWriter, Write};
use std::num::NonZeroU32;

use hashbrown::HashMap;
use rucrf::{Edge, FeatureProvider, FeatureSet, Lattice};

use crate::dictionary::{word_idx::WordIdx, Dictionary, LexType};
use crate::errors::Result;
pub use crate::trainer::config::TrainerConfig;
pub use crate::trainer::corpus::Corpus;
use crate::trainer::corpus::Example;
use crate::trainer::feature_extractor::FeatureExtractor;
use crate::trainer::feature_rewriter::FeatureRewriter;
use crate::utils::FromU32;

use crate::common::MAX_SENTENCE_LENGTH;

/// Trainer of morphological analyzer.
pub struct Trainer {
    dict: Dictionary,
    surfaces: Vec<String>,
    max_grouping_len: Option<u16>,
    provider: FeatureProvider,
    label_id_map: HashMap<String, HashMap<char, u32>>,
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
        let features = corpus::parse_csv_row(feature_str);
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
    /// [`VibratoError`] is returned when the model will become too large.
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
            provider.add_feature_set(feature_set)?;
            label_id_map
                .raw_entry_mut()
                .from_key(feature_str)
                .or_insert_with(|| (feature_str.to_string(), HashMap::new()))
                .1
                .insert(first_char, word_id);
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

        // virtual feature set
        provider.add_feature_set(FeatureSet::new(&[], &[], &[]))?;

        Ok(Self {
            dict: config.dict,
            surfaces: config.surfaces,
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
    /// The value must be greater or equal to 0.
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

        let mut edges = vec![];
        let mut pos = 0;
        for token in tokens {
            let len = token.surface().chars().count();
            let first_char = input_chars[pos];
            let label_id = self
                .label_id_map
                .get(token.feature())
                .and_then(|hm| hm.get(&first_char))
                .map_or_else(
                    || {
                        // FIXME(vbkaisetsu): If an unknown word edge is available, add it instead.
                        eprintln!(
                            "adding virtual edge: {} {}",
                            token.surface(),
                            token.feature()
                        );
                        u32::try_from(self.surfaces.len() + self.dict.unk_handler().len() + 1)
                            .unwrap()
                    },
                    |label| *label + 1,
                );
            edges.push((
                pos,
                Edge::new(pos + len, NonZeroU32::new(label_id).unwrap()),
            ));
            pos += len;
        }
        assert_eq!(pos, usize::from(input_len));

        let mut lattice = Lattice::new(usize::from(input_len)).unwrap();

        // Add positive edges
        for (pos, edge) in edges {
            lattice.add_edge(pos, edge).unwrap();
        }

        // Add negative edges
        for start_word in 0..input_len {
            let mut has_matched = false;

            let suffix = &input_chars[usize::from(start_word)..];

            for m in self.dict.system_lexicon().common_prefix_iterator(suffix) {
                has_matched = true;
                let label_id = NonZeroU32::new(m.word_idx.word_id + 1).unwrap();
                let pos = usize::from(start_word);
                let target = pos + usize::from(m.end_char);
                let edge = Edge::new(target, label_id);
                if let Some(first_edge) = lattice.nodes()[pos].edges().first() {
                    if edge == *first_edge {
                        continue;
                    }
                }
                lattice.add_edge(pos, edge).unwrap();
            }

            self.dict.unk_handler().gen_unk_words(
                sentence,
                start_word,
                has_matched,
                self.max_grouping_len,
                |w| {
                    let id_offset = u32::try_from(self.surfaces.len()).unwrap();
                    let label_id = NonZeroU32::new(id_offset + w.word_idx().word_id + 1).unwrap();
                    let pos = usize::from(start_word);
                    let target = usize::from(w.end_char());
                    let edge = Edge::new(target, label_id);
                    lattice.add_edge(pos, edge).unwrap();
                },
            );
        }

        lattice
    }

    /// Starts training and outputs results.
    ///
    /// # Arguments
    ///
    /// * `corpus` - Corpus used for training.
    /// * `lexicon_wtr` - Write sink targetting to `lex.csv`.
    /// * `connector_wtr` - Write sink targetting to `matrix.def`.
    /// * `unk_handler_wtr` - Write sink targetting to `unk.def`.
    ///
    /// # Errors
    ///
    /// [`VibratoError`](crate::errors::VibratoError) is returned when
    ///
    ///  - the compilation of the corpus fails, or
    ///  - writing results fails.
    pub fn train<L, C, U>(
        self,
        mut corpus: Corpus,
        lexicon_wtr: L,
        connector_wtr: C,
        unk_handler_wtr: U,
    ) -> Result<()>
    where
        L: Write,
        C: Write,
        U: Write,
    {
        let mut lattices = vec![];
        for example in &mut corpus.examples {
            example.sentence.compile(self.dict.char_prop())?;
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

        let merged_model = model.merge()?;

        let mut lexicon_wtr = BufWriter::new(lexicon_wtr);
        let mut unk_handler_wtr = BufWriter::new(unk_handler_wtr);
        let mut connector_wtr = BufWriter::new(connector_wtr);

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

        for i in 0..self.surfaces.len() {
            let mut writer = csv_core::Writer::new();
            let mut surface = self.surfaces[i].as_bytes();
            let feature_set = merged_model.feature_sets[i];
            let word_idx = WordIdx::new(LexType::System, u32::try_from(i).unwrap());
            let feature = self.dict.system_lexicon().word_feature(word_idx);

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

        for i in 0..self.dict.unk_handler().len() {
            let word_idx = WordIdx::new(LexType::Unknown, u32::try_from(i).unwrap());
            let cate_id = self.dict.unk_handler().word_cate_id(word_idx);
            let feature = self.dict.unk_handler().word_feature(word_idx);
            let cate_string = self
                .dict
                .char_prop()
                .cate_string(u32::from(cate_id))
                .unwrap();
            let feature_set = merged_model.feature_sets[self.surfaces.len() + i];
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
            merged_model.right_conn_to_left_feats.len(),
            merged_model.left_conn_to_right_feats.len(),
        )?;
        for (i, hm) in merged_model.matrix.iter().enumerate() {
            let mut pairs: Vec<_> = hm.iter().map(|(&j, &w)| (j, w)).collect();
            pairs.sort_unstable_by_key(|&(k, _)| k);
            for (j, w) in pairs {
                writeln!(
                    &mut connector_wtr,
                    "{} {} {}",
                    i,
                    j,
                    (-w * weight_scale_factor) as i16
                )?;
            }
        }

        Ok(())
    }
}
