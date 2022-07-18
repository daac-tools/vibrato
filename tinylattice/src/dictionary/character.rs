pub mod builder;
pub mod category;

pub use category::CategorySet;

const CATE_IDS_BITS: usize = 18;
const CATE_IDS_MASK: u32 = (1 << CATE_IDS_BITS) - 1;

const BASE_ID_BITS: usize = 8;
const BASE_ID_MASK: u32 = (1 << BASE_ID_BITS) - 1;

const LENGTH_BITS: usize = 4;

// cate_ids: 18
// base_id: 8
// invoke: 1
// group: 1
// length: 4
#[derive(Default, Debug, Clone, Copy)]
pub struct CharInfo(u32);

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
    pub fn base_id(&self) -> u32 {
        (self.0 >> CATE_IDS_BITS) & BASE_ID_MASK
    }

    #[inline(always)]
    pub fn invoke(&self) -> bool {
        (self.0 >> (CATE_IDS_BITS + BASE_ID_BITS)) & 1 != 0
    }

    #[inline(always)]
    pub fn group(&self) -> bool {
        (self.0 >> (CATE_IDS_BITS + BASE_ID_BITS + 1)) & 1 != 0
    }

    #[inline(always)]
    pub fn length(&self) -> usize {
        (self.0 >> (CATE_IDS_BITS + BASE_ID_BITS + 2)) as usize
    }
}

pub struct CharProperty {
    chr2inf: Vec<CharInfo>,
}

impl CharProperty {
    pub fn char_info(&self, c: char) -> CharInfo {
        let c = c as usize;
        if let Some(inf) = self.chr2inf.get(c) {
            inf.clone()
        } else {
            self.chr2inf[0].clone()
        }
    }
}
