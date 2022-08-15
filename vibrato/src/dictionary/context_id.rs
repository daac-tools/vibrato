mod builder;

use bincode::{Decode, Encode};

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
}
