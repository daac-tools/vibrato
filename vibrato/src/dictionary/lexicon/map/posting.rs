use std::ptr::NonNull;

use bincode::{Decode, Encode};

use crate::errors::{Result, VibratoError};

#[derive(Decode, Encode)]
pub struct Postings {
    data: Vec<u8>,
}

impl Postings {
    #[inline(always)]
    pub fn ids(&self, i: usize) -> PostingsIter {
        // Do NOT add debug_ for safety.
        assert!(i < self.data.len());
        let ptr = unsafe { self.data.as_ptr().add(i) };
        let cnt = unsafe { usize::from(ptr.read()) + 1 };
        // Note: If we skip this assertion, the tokenization time is improved by 5%-10%.
        // But, this is necessary for safety. What should we do?
        assert!(i + cnt * std::mem::size_of::<u32>() < self.data.len());
        let data_ptr = unsafe { ptr.offset(1) as *const u32 };
        PostingsIter {
            data: unsafe { NonNull::new_unchecked(data_ptr as _) },
            remaining: cnt,
        }
    }
}

pub struct PostingsIter {
    data: NonNull<u32>,
    remaining: usize,
}

impl Iterator for PostingsIter {
    type Item = u32;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        let ptr = self.data.as_ptr();
        let val = unsafe { ptr.read_unaligned() };
        self.data = unsafe { NonNull::new_unchecked(ptr.offset(1)) };
        self.remaining -= 1;
        Some(val)
    }
}

#[derive(Default)]
pub struct PostingsBuilder {
    data: Vec<u8>,
}

impl PostingsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline(always)]
    pub fn push(&mut self, ids: &[u32]) -> Result<usize> {
        if !(1..=256).contains(&ids.len()) {
            return Err(VibratoError::invalid_argument(
                "ids",
                "Number of ids associated with a word mustb be in [1,256]",
            ));
        }
        let offset = self.data.len();
        self.data.push(u8::try_from(ids.len() - 1).unwrap());
        for id in ids {
            self.data.extend_from_slice(&id.to_le_bytes());
        }
        Ok(offset)
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn build(self) -> Postings {
        Postings { data: self.data }
    }
}
