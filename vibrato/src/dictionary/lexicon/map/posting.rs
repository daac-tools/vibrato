use bincode::{Decode, Encode};

use crate::errors::Result;
use crate::utils::FromU32;

#[derive(Decode, Encode)]
pub struct Postings {
    data: Vec<u32>,
}

impl Postings {
    #[inline(always)]
    pub fn ids<'a>(&'a self, i: usize) -> impl Iterator<Item = u32> + 'a {
        let len = usize::from_u32(self.data[i]);
        // self.data[i + 1..i + 1 + len].iter().cloned()
        unsafe { self.data.get_unchecked(i + 1..i + 1 + len).iter().cloned() }
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
    pub fn push(&mut self, ids: &[u32]) -> Result<usize> {
        let offset = self.data.len();
        self.data.push(ids.len().try_into()?);
        self.data.extend_from_slice(ids);
        Ok(offset)
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn build(self) -> Postings {
        Postings { data: self.data }
    }
}
