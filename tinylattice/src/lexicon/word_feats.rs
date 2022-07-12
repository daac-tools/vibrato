pub struct WordFeats {
    feats: Vec<String>,
}

impl WordFeats {
    pub fn from_iter<I, S>(feats: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self {
            feats: feats.into_iter().map(|s| s.as_ref().to_owned()).collect(),
        }
    }

    #[inline(always)]
    pub fn get(&self, i: usize) -> &str {
        &self.feats[i]
    }
}
