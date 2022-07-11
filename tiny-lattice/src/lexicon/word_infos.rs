pub struct WordInfos {
    infos: Vec<String>,
}

impl WordInfos {
    pub fn from_iter<I, S>(infos: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self {
            infos: infos.into_iter().map(|s| s.as_ref().to_owned()).collect(),
        }
    }

    pub fn get(&self, i: usize) -> &str {
        &self.infos[i]
    }
}
