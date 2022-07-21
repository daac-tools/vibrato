#[derive(Default)]
pub struct WordFeatures {
    features: Vec<String>,
}

impl WordFeatures {
    pub fn new<I, S>(features: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self {
            features: features
                .into_iter()
                .map(|s| s.as_ref().to_owned())
                .collect(),
        }
    }

    #[inline(always)]
    pub fn feature(&self, word_id: usize) -> &str {
        &self.features[word_id]
    }
}
