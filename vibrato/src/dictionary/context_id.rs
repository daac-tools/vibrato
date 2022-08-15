mod builder;

use bincode::{Decode, Encode};

use crate::dictionary::mapper::ConnIdMapper;

#[derive(Decode, Encode)]
pub struct ContextIds {
    bos_right_id: u16,
    eos_left_id: u16,
}

impl ContextIds {
    #[inline(always)]
    pub fn bos_right_id(&self) -> u16 {
        self.bos_right_id
    }

    #[inline(always)]
    pub fn eos_left_id(&self) -> u16 {
        self.eos_left_id
    }

    /// Do NOT make this function public to maintain consistency in
    /// the connection-id mapping among members of `Dictionary`.
    /// The consistency is managed in `Dictionary`.
    pub fn do_mapping(&mut self, mapper: &ConnIdMapper) {
        self.eos_left_id = mapper.left(self.eos_left_id);
        self.bos_right_id = mapper.right(self.bos_right_id);
    }
}
