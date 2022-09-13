use bincode::{Decode, Encode};

#[derive(Default, Decode, Encode)]
pub struct WordFeatures {
    features: Vec<String>,
    chars: Vec<char>,
}

impl WordFeatures {
    pub fn new<I, S>(source: I) -> Self
    where
        I: IntoIterator<Item = (S, char)>,
        S: AsRef<str>,
    {
        let mut features = vec![];
        let mut chars = vec![];
        for (s, c) in source {
            features.push(s.as_ref().to_string());
            chars.push(c);
        }
        Self { features, chars }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.features.len()
    }

    #[inline(always)]
    pub fn get(&self, word_id: usize) -> &str {
        &self.features[word_id]
    }

    #[inline(always)]
    pub fn get_firstchar(&self, word_id: usize) -> char {
        self.chars[word_id]
    }
}
