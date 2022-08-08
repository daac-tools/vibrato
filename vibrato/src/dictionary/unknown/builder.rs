use std::io::Read;

use super::{UnkEntry, UnkHandler};
use crate::dictionary::character::CategorySet;
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
            let rec = rec.map_err(|e| VibratoError::invalid_argument("rdr", e.to_string()))?;
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
            return Err(VibratoError::invalid_argument("rec", msg));
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
