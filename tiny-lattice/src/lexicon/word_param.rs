use super::parser::RawLexiconEntry;

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

    pub const fn from_raw_entry(e: &RawLexiconEntry) -> Self {
        Self {
            left_id: e.left_id,
            right_id: e.right_id,
            cost: e.cost,
        }
    }
}

/// word_id -> array of (left_id, right_id, cost)
pub struct WordParamArrays {
    arrays: Vec<Vec<i16>>,
}

impl WordParamArrays {
    pub fn new<I, P>(params: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: AsRef<[WordParam]>,
    {
        let mut arrays = vec![];
        for ps in params {
            let ps = ps.as_ref();
            let mut array = Vec::with_capacity(ps.len() * 3);
            for p in ps {
                array.push(p.left_id);
                array.push(p.right_id);
                array.push(p.cost);
            }
            arrays.push(array);
        }
        Self { arrays }
    }

    pub fn iter<'a>(&'a self, word_id: u32) -> WordParamIter<'a> {
        WordParamIter {
            array: &self.arrays[word_id as usize],
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
        if self.array.len() <= self.i {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iter() {
        let arrays = vec![
            vec![WordParam::new(1, 2, 3), WordParam::new(4, 5, 6)],
            vec![WordParam::new(7, 8, 9)],
            vec![WordParam::new(10, 11, 12), WordParam::new(13, 14, 15)],
        ];
        let wpa = WordParamArrays::new(arrays);
        {
            let mut it = wpa.iter(0);
            assert_eq!(it.next().unwrap(), WordParam::new(1, 2, 3));
            assert_eq!(it.next().unwrap(), WordParam::new(4, 5, 6));
            assert_eq!(it.next(), None);
        }
        {
            let mut it = wpa.iter(1);
            assert_eq!(it.next().unwrap(), WordParam::new(7, 8, 9));
            assert_eq!(it.next(), None);
        }
        {
            let mut it = wpa.iter(2);
            assert_eq!(it.next().unwrap(), WordParam::new(10, 11, 12));
            assert_eq!(it.next().unwrap(), WordParam::new(13, 14, 15));
            assert_eq!(it.next(), None);
        }
    }
}
