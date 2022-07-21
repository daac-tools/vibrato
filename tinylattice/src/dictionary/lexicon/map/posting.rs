// TODO: Opt
pub struct Postings {
    data: Vec<u32>,
}

impl Postings {
    #[inline(always)]
    pub fn ids(&self, i: usize) -> &[u32] {
        let cnt = self.data[i] as usize;
        &self.data[i + 1..i + 1 + cnt]
    }
}

#[derive(Default)]
pub struct PostingsBuilder {
    data: Vec<u32>,
}

impl PostingsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline(always)]
    pub fn push(&mut self, ids: &[u32]) -> usize {
        let offset = self.data.len();
        self.data.push(ids.len() as u32);
        self.data.extend_from_slice(ids);
        offset
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn build(self) -> Postings {
        Postings { data: self.data }
    }
}
