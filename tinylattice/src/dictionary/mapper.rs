pub mod builder;

pub struct ConnIdMapper {
    left: Option<Vec<(u16, u16)>>,
    right: Option<Vec<(u16, u16)>>,
}

impl ConnIdMapper {
    pub fn has_left(&self) -> bool {
        self.left.is_some()
    }

    pub fn has_right(&self) -> bool {
        self.right.is_some()
    }

    #[inline(always)]
    pub fn left(&self, id: u16) -> u16 {
        self.left.as_ref().map_or_else(|| id, |m| m[id as usize].1)
    }

    #[inline(always)]
    pub fn right(&self, id: u16) -> u16 {
        self.right.as_ref().map_or_else(|| id, |m| m[id as usize].1)
    }

    #[inline(always)]
    pub fn left_inv(&self, id: u16) -> u16 {
        self.left.as_ref().map_or_else(|| id, |m| m[id as usize].0)
    }

    #[inline(always)]
    pub fn right_inv(&self, id: u16) -> u16 {
        self.right.as_ref().map_or_else(|| id, |m| m[id as usize].0)
    }
}
