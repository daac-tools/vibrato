mod config;
mod corpus;
mod feature_extractor;
mod feature_rewriter;

use std::num::NonZeroU32;

use hashbrown::HashMap;
use rucrf::{Edge, FeatureProvider, FeatureSet, Lattice};
use crawdad::Trie;

use crate::errors::{Result, VibratoError};
use crate::trainer::config::TrainerConfig;
use crate::trainer::corpus::{Corpus, Dictionary, Word, Sentence};
use crate::utils::FromU32;

#[derive(Default)]
struct LatticeGeneratorBuilder<'a> {
    surface_feature_map: HashMap<&'a str, HashMap<&'a str, NonZeroU32>>,
    feature_provider: FeatureProvider,
}

impl<'a> LatticeGeneratorBuilder<'a> {
    fn extract_features(word: &Word, config: &mut TrainerConfig) -> Result<FeatureSet> {
        let TrainerConfig {
            char_property,
            feature_extractor,
            unigram_rewriter,
            left_rewriter,
            right_rewriter,
        } = config;
        let c = word.surface().chars().next().ok_or_else(|| {
            VibratoError::invalid_argument("word", "word must contains at least a character")
        })?;
        let char_info = char_property.char_info(c);
        let cate_id = char_info.base_id();

        let features = word.features_vec();

        // If the rewriting rule is available, the rule is applied; otherwise, the original feature is used.
        let unigram_fids = if let Some(features) = unigram_rewriter.rewrite(&features) {
            feature_extractor.extract_unigram_feature_ids(&features, cate_id)
        } else {
            feature_extractor.extract_unigram_feature_ids(&features, cate_id)
        };
        let left_fids = if let Some(features) = left_rewriter.rewrite(&features) {
            feature_extractor.extract_left_feature_ids(&features)
        } else {
            feature_extractor.extract_left_feature_ids(&features)
        };
        let right_fids = if let Some(features) = right_rewriter.rewrite(&features) {
            feature_extractor.extract_right_feature_ids(&features)
        } else {
            feature_extractor.extract_right_feature_ids(&features)
        };
        Ok(FeatureSet::new(&unigram_fids, &left_fids, &right_fids))
    }

    fn update_feature_map(&mut self, word: &'a Word, config: &mut TrainerConfig) -> Result<()> {
        let new_feature_id = NonZeroU32::new(u32::try_from(self.feature_provider.len() + 1).unwrap()).unwrap();
        let feature_id = *self.surface_feature_map
            .entry(word.surface())
            .or_insert_with(|| HashMap::new())
            .entry(word.features())
            .or_insert(new_feature_id);
        if feature_id == new_feature_id {
            let feature_set = Self::extract_features(
                word,
                config,
            )?;
            self.feature_provider.add_feature_set(feature_set);
        }
        Ok(())
    }

    fn build(self) -> LatticeGenerator<'a> {
        let mut surface_feature_map: Vec<_> = self.surface_feature_map.iter().collect();
        surface_feature_map.sort_unstable_by(|a, b| a.0.cmp(&b.0));

        let mut surfaces = vec![];
        let mut feature_ids = vec![];
        for (surface, features) in surface_feature_map {
            surfaces.push(surface);
            feature_ids.push(features.values().copied().collect());
        }
        LatticeGenerator {
            trie: Trie::from_keys(surfaces).unwrap(),
            feature_ids,
            surface_feature_map: self.surface_feature_map,
            feature_provider: self.feature_provider,
        }
    }
}

struct LatticeGenerator<'a> {
    surface_feature_map: HashMap<&'a str, HashMap<&'a str, NonZeroU32>>,
    feature_provider: FeatureProvider,
    trie: Trie,
    feature_ids: Vec<Vec<NonZeroU32>>,
}

impl<'a> LatticeGenerator<'a> {
    fn generate_lattice(&self, sentence: &Sentence) -> Lattice {
        let mut sentence_chars = vec![];
        let mut trunk = vec![];
        for token in sentence.tokens() {
            sentence_chars.extend(token.surface().chars());
            let feature_id = *self.surface_feature_map
                .get(token.surface())
                .unwrap()
                .get(token.features())
                .unwrap();
            trunk.push(Edge::new(sentence_chars.len(), feature_id));
        }
        let mut lattice = Lattice::new(&self.feature_provider, sentence_chars.len()).unwrap();

        // Adds positive edges
        let mut pos = 0;
        for edge in trunk {
            lattice.add_edge(pos, edge).unwrap();
            pos = edge.target();
        }

        // Adds negative edges
        for i in 0..sentence_chars.len() {
            for (val, k) in self.trie.common_prefix_search(sentence_chars[i..].iter().copied()) {
                for &feature_id in &self.feature_ids[usize::from_u32(val)] {
                    let edge = Edge::new(i + k, feature_id);
                    // If the edge is a positive edge, skips insertion.
                    if let Some(first_edge) = lattice.nodes()[i].edges().first() {
                        if edge == *first_edge {
                            continue;
                        }
                    }
                    lattice.add_edge(i, edge).unwrap();
                }
            }
        }
        lattice
    }
}

#[allow(unused)]
pub struct Trainer {
    config: TrainerConfig,
}

impl Trainer {
    #[allow(unused)]
    pub fn new(config: TrainerConfig) -> Self {
        Self { config }
    }

    #[allow(unused)]
    pub fn train(mut self, corpus: Corpus, dict: Option<Dictionary>) -> Result<()> {
        let mut builder = LatticeGeneratorBuilder::default();

        eprintln!("extracting features");
        for sentence in corpus.sentences() {
            for token in sentence.tokens() {
                builder.update_feature_map(token, &mut self.config)?;
            }
        }
        if let Some(dict) = dict.as_ref() {
            for word in dict.words() {
                if word.surface().is_empty() {
                    continue;
                }
                builder.update_feature_map(word, &mut self.config)?;
            }
        }

        eprintln!("generating lattices");
        let lattice_generator = builder.build();
        let mut lattices = vec![];
        for sentence in corpus.sentences() {
            lattices.push(lattice_generator.generate_lattice(sentence));
        }

        eprintln!("start training");
        let trainer = rucrf::Trainer::new()
            .regularization(rucrf::Regularization::L1, 0.1)
            .unwrap()
            .max_iter(500)
            .unwrap()
            .n_threads(20)
            .unwrap();
        let model = trainer.train(&lattices);

        let mut n_nonzero = 0;
        for w in &model.weights {
            if w.abs() > f64::EPSILON {
                n_nonzero += 1;
            }
        }
        dbg!(n_nonzero);

        Ok(())
    }
}
