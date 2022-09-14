mod config;
mod corpus;
mod feature_extractor;
mod feature_rewriter;

use std::io::Write;
use std::num::NonZeroU32;

use hashbrown::HashMap;
use rucrf::{Edge, FeatureProvider, FeatureSet, Lattice};

use crate::dictionary::LexType;
use crate::dictionary::{word_idx::WordIdx, Dictionary};
use crate::errors::Result;
pub use crate::trainer::config::TrainerConfig;
pub use crate::trainer::corpus::Corpus;
use crate::trainer::corpus::Example;
use crate::trainer::feature_extractor::FeatureExtractor;
use crate::trainer::feature_rewriter::FeatureRewriter;
use crate::utils::FromU32;

use crate::common::MAX_SENTENCE_LENGTH;

pub struct Trainer {
    dict: Dictionary,
    surfaces: Vec<String>,
    max_grouping_len: Option<u16>,
    provider: FeatureProvider,
    label_id_map: HashMap<String, HashMap<char, u32>>,
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
        FeatureSet::new(&unigram_features, &left_features, &right_features)
    }

    pub fn new(mut config: TrainerConfig) -> Self {
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
            provider.add_feature_set(feature_set);
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
            provider.add_feature_set(feature_set);
        }

        // virtual feature set
        provider.add_feature_set(FeatureSet::new(&[], &[], &[]));

        Self {
            dict: config.dict,
            surfaces: config.surfaces,
            max_grouping_len: None,
            provider,
            label_id_map,
        }
    }

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
            let label_id = if let Some(label) = self
                .label_id_map
                .get(token.feature())
                .and_then(|hm| hm.get(&first_char))
            {
                *label + 1
            } else {
                eprintln!(
                    "adding virtual edge: {} {}",
                    token.surface(),
                    token.feature()
                );
                u32::try_from(self.surfaces.len() + self.dict.unk_handler().len() + 1).unwrap()
            };
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

    pub fn train(self, mut corpus: Corpus) -> Result<()> {
        let mut lattices = vec![];
        for example in &mut corpus.examples {
            example.sentence.compile(self.dict.char_prop())?;
            lattices.push(self.build_lattice(example));
        }

        let trainer = rucrf::Trainer::new()
            .regularization(rucrf::Regularization::L1, 0.01)
            .unwrap()
            .max_iter(300)
            .unwrap()
            .n_threads(20)
            .unwrap();
        let model = trainer.train(&lattices, self.provider);

        let compiled_model = model.compile();

        let mut lex_file = std::io::BufWriter::new(std::fs::File::create("lex.csv")?);
        let mut unk_file = std::io::BufWriter::new(std::fs::File::create("unk.def")?);
        let mut matrix_file = std::io::BufWriter::new(std::fs::File::create("matrix.def")?);

        let mut output = [0; 4096];
        let mut writer = csv_core::Writer::new();

        for i in 0..self.surfaces.len() {
            let mut surface = self.surfaces[i].as_bytes();
            let feature_set = compiled_model.feature_sets[i];
            let word_idx = WordIdx::new(LexType::System, u32::try_from(i).unwrap());
            let feature = self.dict.system_lexicon().word_feature(word_idx);

            // writes surface
            loop {
                let (result, nin, nout) = writer.field(surface, &mut output);
                lex_file.write_all(&output[..nout])?;
                if result == csv_core::WriteResult::InputEmpty {
                    break;
                }
                surface = &surface[nin..];
            }
            let (result, nout) = writer.finish(&mut output);
            assert_eq!(result, csv_core::WriteResult::InputEmpty);
            lex_file.write_all(&output[..nout])?;

            // writes others
            lex_file.write_all(
                format!(
                    ",{},{},{},{}\n",
                    feature_set.left_id,
                    feature_set.right_id,
                    (feature_set.weight * 700.) as i32,
                    feature
                )
                .as_bytes(),
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
            let feature_set = compiled_model.feature_sets[self.surfaces.len() + i];
            unk_file.write_all(
                format!(
                    "{},{},{},{},{}\n",
                    cate_string,
                    feature_set.left_id,
                    feature_set.right_id,
                    (feature_set.weight * 700.) as i32,
                    feature
                )
                .as_bytes(),
            )?;
        }

        matrix_file.write_all(
            format!(
                "{} {}\n",
                compiled_model.left_ids.len(),
                compiled_model.right_ids.len()
            )
            .as_bytes(),
        )?;
        for (i, hm) in compiled_model.matrix.iter().enumerate() {
            for (j, w) in hm {
                matrix_file
                    .write_all(format!("{} {} {}\n", i + 1, j + 1, (w * 700.) as i32).as_bytes())?;
            }
        }

        Ok(())
    }
}
