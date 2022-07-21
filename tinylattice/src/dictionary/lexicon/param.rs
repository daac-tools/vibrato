use super::ConnIdMapper;

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub struct WordParam {
    pub left_id: i16,
    pub right_id: i16,
    pub word_cost: i16,
}

impl WordParam {
    #[inline(always)]
    pub const fn new(left_id: i16, right_id: i16, word_cost: i16) -> Self {
        Self {
            left_id,
            right_id,
            word_cost,
        }
    }
}

pub struct WordParams {
    params: Vec<WordParam>,
}

impl WordParams {
    // (left_id, right_id, word_cost)
    pub fn new<I>(params: I) -> Self
    where
        I: IntoIterator<Item = WordParam>,
    {
        Self {
            params: params.into_iter().collect(),
        }
    }

    #[inline(always)]
    pub fn param(&self, word_id: usize) -> WordParam {
        self.params[word_id]
    }

    pub fn map_ids(&mut self, mapper: &ConnIdMapper) {
        for p in &mut self.params {
            p.left_id = mapper.left(p.left_id as u16) as i16;
            p.right_id = mapper.right(p.right_id as u16) as i16;
        }
    }
}