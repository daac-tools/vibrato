mod builder;
mod category;

use std::fmt;

use bincode::{Decode, Encode};

use crate::utils::FromU32;

pub use category::CategorySet;

const CATE_IDS_BITS: usize = 18;
const CATE_IDS_MASK: u32 = (1 << CATE_IDS_BITS) - 1;
const BASE_ID_BITS: usize = 8;
const BASE_ID_MASK: u32 = (1 << BASE_ID_BITS) - 1;
const LENGTH_BITS: usize = 4;

/// Information of a character defined in `char.def`.
///
/// The memory layout is
///  - cate_ids: 18 bits
///  -  base_id:  8 bits
///  -   invoke:  1 bit
///  -    group:  1 bit
///  -   length:  4 bits
#[derive(Default, Clone, Copy, Decode, Encode)]
pub struct CharInfo(u32);

impl fmt::Debug for CharInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CharInfo")
            .field("cate_ids", &self.cate_ids().cate_strs())
            .field("base_id", &CategorySet::cate_str(self.base_id()))
            .field("invoke", &self.invoke())
            .field("group", &self.group())
            .field("length", &self.length())
            .finish()
    }
}

impl CharInfo {
    pub fn new(
        cate_ids: CategorySet,
        base_id: u32,
        invoke: bool,
        group: bool,
        length: usize,
    ) -> Option<Self> {
        if cate_ids.raw() >> CATE_IDS_BITS != 0 {
            return None;
        }
        if base_id >> BASE_ID_BITS != 0 {
            return None;
        }
        if length >> LENGTH_BITS != 0 {
            return None;
        }
        Some(Self(
            cate_ids.raw()
                | (base_id << CATE_IDS_BITS)
                | (u32::from(invoke) << (CATE_IDS_BITS + BASE_ID_BITS))
                | (u32::from(group) << (CATE_IDS_BITS + BASE_ID_BITS + 1))
                | ((length as u32) << (CATE_IDS_BITS + BASE_ID_BITS + 2)),
        ))
    }

    #[inline(always)]
    pub fn set_cate_ids(&mut self, cate_ids: CategorySet) {
        self.0 &= !CATE_IDS_MASK;
        self.0 |= cate_ids.raw();
    }

    #[inline(always)]
    pub fn cate_ids(&self) -> CategorySet {
        let bits = self.0 & CATE_IDS_MASK;
        unsafe { CategorySet::from_raw_unchecked(bits) }
    }

    #[inline(always)]
    pub const fn base_id(&self) -> u32 {
        (self.0 >> CATE_IDS_BITS) & BASE_ID_MASK
    }

    #[inline(always)]
    pub const fn invoke(&self) -> bool {
        (self.0 >> (CATE_IDS_BITS + BASE_ID_BITS)) & 1 != 0
    }

    #[inline(always)]
    pub const fn group(&self) -> bool {
        (self.0 >> (CATE_IDS_BITS + BASE_ID_BITS + 1)) & 1 != 0
    }

    #[inline(always)]
    pub fn length(&self) -> usize {
        usize::from_u32(self.0 >> (CATE_IDS_BITS + BASE_ID_BITS + 2))
    }
}

/// Mapping from characters to their information.
#[derive(Decode, Encode)]
pub struct CharProperty {
    chr2inf: Vec<CharInfo>,
}

impl CharProperty {
    #[inline(always)]
    pub(crate) fn char_info(&self, c: char) -> CharInfo {
        self.chr2inf
            .get(c as usize)
            .map_or_else(|| self.chr2inf[0], |cinfo| *cinfo)
    }
}
