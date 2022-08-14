mod builder;

use std::fmt;

use bincode::{Decode, Encode};

use crate::utils::FromU32;

const CATE_IDSET_BITS: usize = 18;
const CATE_IDSET_MASK: u32 = (1 << CATE_IDSET_BITS) - 1;
const BASE_ID_BITS: usize = 8;
const BASE_ID_MASK: u32 = (1 << BASE_ID_BITS) - 1;
const LENGTH_BITS: usize = 4;

/// Information of a character defined in `char.def`.
///
/// The memory layout is
///   cate_idset = 18 bits
///      base_id =  8 bits
///       invoke =  1 bit
///        group =  1 bit
///       length =  4 bits
#[derive(Default, Clone, Copy, Decode, Encode)]
pub struct CharInfo(u32);

impl fmt::Debug for CharInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CharInfo")
            .field("cate_idset", &self.cate_idset())
            .field("base_id", &self.base_id())
            .field("invoke", &self.invoke())
            .field("group", &self.group())
            .field("length", &self.length())
            .finish()
    }
}

impl CharInfo {
    pub fn new(
        cate_idset: u32,
        base_id: u32,
        invoke: bool,
        group: bool,
        length: u16,
    ) -> Option<Self> {
        if cate_idset >> CATE_IDSET_BITS != 0 {
            return None;
        }
        if base_id >> BASE_ID_BITS != 0 {
            return None;
        }
        if length >> LENGTH_BITS != 0 {
            return None;
        }
        Some(Self(
            cate_idset
                | (base_id << CATE_IDSET_BITS)
                | (u32::from(invoke) << (CATE_IDSET_BITS + BASE_ID_BITS))
                | (u32::from(group) << (CATE_IDSET_BITS + BASE_ID_BITS + 1))
                | ((u32::from(length)) << (CATE_IDSET_BITS + BASE_ID_BITS + 2)),
        ))
    }

    #[inline(always)]
    pub fn reset_cate_idset(&mut self, cate_idset: u32) {
        self.0 &= !CATE_IDSET_MASK;
        self.0 |= cate_idset;
    }

    #[inline(always)]
    pub const fn cate_idset(&self) -> u32 {
        self.0 & CATE_IDSET_MASK
    }

    #[inline(always)]
    pub const fn base_id(&self) -> u32 {
        (self.0 >> CATE_IDSET_BITS) & BASE_ID_MASK
    }

    #[inline(always)]
    pub const fn invoke(&self) -> bool {
        (self.0 >> (CATE_IDSET_BITS + BASE_ID_BITS)) & 1 != 0
    }

    #[inline(always)]
    pub const fn group(&self) -> bool {
        (self.0 >> (CATE_IDSET_BITS + BASE_ID_BITS + 1)) & 1 != 0
    }

    #[inline(always)]
    pub const fn length(&self) -> u16 {
        (self.0 >> (CATE_IDSET_BITS + BASE_ID_BITS + 2)) as u16
    }
}

/// Mapping from characters to their information.
#[derive(Decode, Encode)]
pub struct CharProperty {
    chr2inf: Vec<CharInfo>,
    categories: Vec<String>, // indexed by category id
}

impl CharProperty {
    #[inline(always)]
    pub fn char_info(&self, c: char) -> CharInfo {
        self.chr2inf
            .get(usize::from_u32(u32::from(c)))
            .map_or_else(|| self.chr2inf[0], |cinfo| *cinfo)
    }

    #[inline(always)]
    pub fn cate_id(&self, category: &str) -> Option<u32> {
        self.categories
            .iter()
            .position(|cate| cate == category)
            .map(|id| u32::try_from(id).unwrap())
    }

    #[inline(always)]
    pub fn num_categories(&self) -> usize {
        self.categories.len()
    }
}
