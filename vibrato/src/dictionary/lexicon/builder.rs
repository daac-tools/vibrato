use std::io::Read;

use anyhow::{anyhow, Result};

use super::{LexType, Lexicon, RawWordEntry, WordFeatures, WordMap, WordParam, WordParams};

impl Lexicon {
    /// Builds a new [`Lexicon`] from a lexicon file in the CSV format.
    pub fn from_reader<R>(rdr: R, lex_type: LexType) -> Result<Self>
    where
        R: Read,
    {
        let mut entries = vec![];
        let mut reader = csv::ReaderBuilder::new()
            .flexible(true)
            .has_headers(false)
            .from_reader(rdr);

        for (i, rec) in reader.records().enumerate() {
            let rec = rec?;
            let e = Self::parse_csv(&rec)?;
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

    fn parse_csv(rec: &csv::StringRecord) -> Result<RawWordEntry> {
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
}
