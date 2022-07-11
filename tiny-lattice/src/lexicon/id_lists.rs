#[derive(Default)]
pub struct IdListsBuilder {
    data: Vec<u32>,
}

impl IdListsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, ids: &[u32]) -> usize {
        let offset = self.data.len();
        self.data.push(ids.len() as u32);
        self.data.extend_from_slice(ids);
        offset
    }

    pub fn build(self) -> IdLists {
        IdLists { data: self.data }
    }
}

pub struct IdLists {
    data: Vec<u32>,
}

impl IdLists {
    #[inline(always)]
    pub fn get(&self, i: usize) -> &[u32] {
        let cnt = self.data[i] as usize;
        &self.data[i + 1..i + 1 + cnt]
    }
}
