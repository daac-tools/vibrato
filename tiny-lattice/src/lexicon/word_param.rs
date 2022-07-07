#[derive(Eq, PartialEq, Debug)]
pub struct WordParam {
    pub left_id: i16,
    pub right_id: i16,
    pub cost: i16,
}

/// word_id -> array of (left_id, right_id, cost)
pub struct WordParamArrays {
    arrays: Vec<Vec<i16>>,
}

impl WordParamArrays {
    pub fn new(arrays: Vec<Vec<i16>>) -> Self {
        Self { arrays }
    }

    pub fn get_array(&self, word_id: u32) -> &[i16] {
        &self.arrays[word_id as usize]
    }

    pub fn iter<'a>(&'a self, word_id: u32) -> WordParamIter<'a> {
        WordParamIter {
            array: self.get_array(word_id),
            i: 0,
        }
    }
}

pub struct WordParamIter<'a> {
    array: &'a [i16],
    i: usize,
}

impl<'a> Iterator for WordParamIter<'a> {
    type Item = WordParam;

    fn next(&mut self) -> Option<Self::Item> {
        if self.array.len() < self.i {
            return None;
        }
        let param = WordParam {
            left_id: self.array[self.i],
            right_id: self.array[self.i + 1],
            cost: self.array[self.i + 2],
        };
        self.i += 3;
        Some(param)
    }
}
