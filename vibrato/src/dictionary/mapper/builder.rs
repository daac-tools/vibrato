use crate::dictionary::mapper::ConnIdMapper;
use crate::errors::{Result, VibratoError};

impl ConnIdMapper {
    pub fn from_iter<L, R>(lmap: L, rmap: R) -> Result<Self>
    where
        L: IntoIterator<Item = u16>,
        R: IntoIterator<Item = u16>,
    {
        let left = Self::parse(lmap)?;
        let right = Self::parse(rmap)?;
        Ok(Self { left, right })
    }

    fn parse<I>(map: I) -> Result<Vec<u16>>
    where
        I: IntoIterator<Item = u16>,
    {
        let mut old_ids = vec![];
        for old_id in map {
            old_ids.push(old_id);
        }

        let mut new_ids = vec![u16::MAX; old_ids.len()];
        for (new_id, &old_id) in old_ids.iter().enumerate() {
            if new_ids[usize::from(old_id)] != u16::MAX {
                return Err(VibratoError::invalid_argument("map", "ids are duplicate."));
            }
            if let Some(e) = new_ids.get_mut(usize::from(old_id)) {
                *e = u16::try_from(new_id)?;
            } else {
                return Err(VibratoError::invalid_argument(
                    "map",
                    "ids are out of range.",
                ));
            }
        }
        Ok(new_ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let map = vec![2, 3, 0, 4, 1];
        let mapping = ConnIdMapper::parse(map.into_iter()).unwrap();
        assert_eq!(mapping, vec![2, 4, 0, 1, 3]);
    }

    #[test]
    #[should_panic]
    fn test_oor() {
        let map = vec![2, 3, 0, 5, 1];
        ConnIdMapper::parse(map.into_iter()).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_dup() {
        let map = vec![2, 3, 0, 2, 1];
        ConnIdMapper::parse(map.into_iter()).unwrap();
    }
}
