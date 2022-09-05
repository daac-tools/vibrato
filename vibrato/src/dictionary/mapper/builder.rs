use crate::dictionary::mapper::ConnIdMapper;
use crate::errors::{Result, VibratoError};

use crate::common::BOS_EOS_CONNECTION_ID;

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
        let mut old_ids = vec![BOS_EOS_CONNECTION_ID];
        for old_id in map {
            if old_id == BOS_EOS_CONNECTION_ID {
                let msg = format!("Id {BOS_EOS_CONNECTION_ID} is reserved.");
                return Err(VibratoError::invalid_argument("map", msg));
            }
            old_ids.push(old_id);
        }

        let mut new_ids = vec![u16::MAX; old_ids.len()];
        new_ids[usize::from(BOS_EOS_CONNECTION_ID)] = BOS_EOS_CONNECTION_ID;

        for (new_id, &old_id) in old_ids.iter().enumerate().skip(1) {
            debug_assert_ne!(old_id, BOS_EOS_CONNECTION_ID);
            if let Some(e) = new_ids.get_mut(usize::from(old_id)) {
                if *e != u16::MAX {
                    return Err(VibratoError::invalid_argument("map", "ids are duplicate."));
                }
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
        let map = vec![2, 3, 4, 1];
        let mapping = ConnIdMapper::parse(map.into_iter()).unwrap();
        assert_eq!(mapping, vec![0, 4, 1, 2, 3]);
    }

    #[test]
    fn test_zero() {
        let map = vec![2, 3, 0, 1];
        let result = ConnIdMapper::parse(map.into_iter());
        assert!(result.is_err());
    }

    #[test]
    fn test_oor() {
        let map = vec![2, 3, 5, 1];
        let result = ConnIdMapper::parse(map.into_iter());
        assert!(result.is_err());
    }
}
