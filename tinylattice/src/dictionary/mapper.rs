mod builder;

pub struct ConnIdMapper {
    left: Vec<(u16, u16)>,
    right: Vec<(u16, u16)>,
}

impl ConnIdMapper {
    #[inline(always)]
    pub(crate) fn num_left(&self) -> usize {
        self.left.len()
    }

    #[inline(always)]
    pub(crate) fn num_right(&self) -> usize {
        self.right.len()
    }

    #[inline(always)]
    pub(crate) fn left(&self, id: u16) -> u16 {
        self.left[id as usize].1
    }

    #[inline(always)]
    pub(crate) fn right(&self, id: u16) -> u16 {
        self.right[id as usize].1
    }

    #[inline(always)]
    pub(crate) fn left_inv(&self, id: u16) -> u16 {
        self.left[id as usize].0
    }

    #[inline(always)]
    pub(crate) fn right_inv(&self, id: u16) -> u16 {
        self.right[id as usize].0
    }
}
