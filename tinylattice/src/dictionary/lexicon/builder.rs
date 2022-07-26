use std::io::Read;

use anyhow::{anyhow, Result};

use super::{LexType, Lexicon, RawWordEntry, WordFeatures, WordMap, WordParam, WordParams};

const USER_COST: i16 = i16::MIN;

impl Lexicon {
    /// Builds a new [`Lexicon`] from a lexicon file in the CSV format.
    pub fn from_reader<R>(rdr: R, lex_type: LexType) -> Result<Self>
    where
        R: Read,
    {
        let mut entries = vec![];
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(rdr);

        for (i, rec) in reader.records().enumerate() {
            let rec = rec?;
            let e = match lex_type {
                LexType::System => Self::parse_csv_system(&rec)?,
                LexType::User => Self::parse_csv_user(&rec)?,
                _ => return Err(anyhow!("Unsupported LexType")),
            };
            if e.surface.is_empty() {
                println!("Skipped an empty surface (at line {})", i);
            } else {
                entries.push(e);
            }
        }

        let map = WordMap::new(entries.iter().map(|e| &e.surface))?;
        let params = WordParams::new(entries.iter().map(|e| e.param));
        let features = WordFeatures::new(entries.iter().map(|e| &e.feature));

        Ok(Self {
            map,
            params,
            features,
            lex_type,
        })
    }

    fn parse_csv_system(rec: &csv::StringRecord) -> Result<RawWordEntry> {
        if rec.len() < 4 {
            return Err(anyhow!("Invalid format: {:?}", rec));
        }

        let mut iter = rec.iter();
        let surface = iter.next().unwrap().parse()?;
        let left_id = iter.next().unwrap().parse()?;
        let right_id = iter.next().unwrap().parse()?;
        let word_cost = iter.next().unwrap().parse()?;
        let feature = iter.collect::<Vec<_>>().join(",");

        Ok(RawWordEntry {
            surface,
            param: WordParam::new(left_id, right_id, word_cost),
            feature,
        })
    }

    fn parse_csv_user(rec: &csv::StringRecord) -> Result<RawWordEntry> {
        if rec.len() < 1 {
            return Err(anyhow!("Invalid format: {:?}", rec));
        }

        let mut iter = rec.iter();
        let surface = iter.next().unwrap().parse()?;
        let feature = iter.collect::<Vec<_>>().join(",");

        Ok(RawWordEntry {
            surface,
            param: WordParam::new(0, 0, USER_COST),
            feature,
        })
    }
}
