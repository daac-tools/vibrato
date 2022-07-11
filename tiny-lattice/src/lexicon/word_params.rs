#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub struct WordParam {
    pub left_id: i16,
    pub right_id: i16,
    pub cost: i16,
}

impl WordParam {
    pub const fn new(left_id: i16, right_id: i16, cost: i16) -> Self {
        Self {
            left_id,
            right_id,
            cost,
        }
    }
}

pub struct WordParams {
    params: Vec<WordParam>,
}

impl WordParams {
    pub fn from_iter<I>(params: I) -> Self
    where
        I: IntoIterator<Item = WordParam>,
    {
        Self {
            params: params.into_iter().collect(),
        }
    }

    pub fn get(&self, i: usize) -> WordParam {
        self.params[i]
    }
}
