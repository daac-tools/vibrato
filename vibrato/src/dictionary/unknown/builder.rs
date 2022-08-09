use std::io::Read;

use crate::dictionary::character::CategorySet;
use crate::dictionary::unknown::{UnkEntry, UnkHandler};
use crate::errors::{Result, VibratoError};

impl UnkHandler {
    /// Creates a new instance from `unk.def`.
    ///
    /// # Arguments
    ///
    ///  - `rdr`: A reader of the file.
    pub fn from_reader<R>(rdr: R) -> Result<Self>
    where
        R: Read,
    {
        let mut map = vec![vec![]; CategorySet::NUM_CATEGORIES];
        let mut reader = csv::ReaderBuilder::new()
            .flexible(true)
            .has_headers(false)
            .from_reader(rdr);
        for rec in reader.records() {
            let rec = rec.map_err(|e| VibratoError::invalid_argument("unk.def", e.to_string()))?;
            let e = Self::parse_unk_entry(&rec)?;
            map[usize::from(e.cate_id)].push(e);
        }

        let mut offsets = vec![];
        let mut entries = vec![];
        for mut v in map {
            offsets.push(entries.len());
            entries.append(&mut v);
        }
        offsets.push(entries.len());
        Ok(Self { offsets, entries })
    }

    fn parse_unk_entry(rec: &csv::StringRecord) -> Result<UnkEntry> {
        if rec.len() < 4 {
            let msg = format!(
                "A csv row of lexicon must have four items at least, {:?}",
                rec
            );
            return Err(VibratoError::invalid_argument("unk.def", msg));
        }

        let mut iter = rec.iter();
        let category: CategorySet = iter.next().unwrap().parse()?;
        let left_id = iter.next().unwrap().parse()?;
        let right_id = iter.next().unwrap().parse()?;
        let word_cost = iter.next().unwrap().parse()?;
        let feature = iter.collect::<Vec<_>>().join(",");

        let cate_id = u16::try_from(category.first_id().unwrap()).unwrap();

        Ok(UnkEntry {
            cate_id,
            left_id,
            right_id,
            word_cost,
            feature,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let data = "DEFAULT,0,2,1,補助記号\nALPHA,1,0,-4,名詞\nALPHA,2,2,3,Meishi";
        let unk = UnkHandler::from_reader(data.as_bytes()).unwrap();
        assert_eq!(
            unk.offsets,
            vec![
                0, //DEFAULT = 0
                1, 1, 1, 1, 1, // ALPHA = 5
                3, 3, 3, 3, 3, 3
            ]
        );
        assert_eq!(
            unk.entries,
            vec![
                UnkEntry {
                    cate_id: 0,
                    left_id: 0,
                    right_id: 2,
                    word_cost: 1,
                    feature: "補助記号".to_string(),
                },
                UnkEntry {
                    cate_id: 5,
                    left_id: 1,
                    right_id: 0,
                    word_cost: -4,
                    feature: "名詞".to_string(),
                },
                UnkEntry {
                    cate_id: 5,
                    left_id: 2,
                    right_id: 2,
                    word_cost: 3,
                    feature: "Meishi".to_string(),
                }
            ]
        );
    }

    #[test]
    #[should_panic]
    fn test_few_cols() {
        let data = "DEFAULT,0,2";
        UnkHandler::from_reader(data.as_bytes()).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_invalid_cate() {
        let data = "INVALID,0,2,1,補助記号";
        UnkHandler::from_reader(data.as_bytes()).unwrap();
    }
}
