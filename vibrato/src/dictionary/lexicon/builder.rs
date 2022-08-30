use std::io::Read;

use crate::dictionary::lexicon::{
    LexType, Lexicon, RawWordEntry, WordFeatures, WordMap, WordParam, WordParams,
};
use crate::errors::{Result, VibratoError};

impl Lexicon {
    /// Builds a new instance from a lexicon file in the CSV format.
    pub fn from_reader<R>(rdr: R, lex_type: LexType) -> Result<Self>
    where
        R: Read,
    {
        let mut entries = vec![];
        let mut reader = csv::ReaderBuilder::new()
            .flexible(true)
            .has_headers(false)
            .from_reader(rdr); // automatically buffered

        for (i, rec) in reader.records().enumerate() {
            let rec = rec.map_err(|e| VibratoError::invalid_format("lex.csv", e.to_string()))?;
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
            let msg = format!(
                "A csv row of lexicon must have four items at least, {:?}",
                rec
            );
            return Err(VibratoError::invalid_format("lex.csv", msg));
        }

        let mut iter = rec.iter();
        let surface = iter.next().unwrap().to_string();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system() {
        let data = "自然,0,2,1,sizen\n言語,1,0,-4,gengo,げんご";
        let lex = Lexicon::from_reader(data.as_bytes(), LexType::System).unwrap();
        assert_eq!(lex.params.get(0), WordParam::new(0, 2, 1));
        assert_eq!(lex.params.get(1), WordParam::new(1, 0, -4));
        assert_eq!(lex.features.get(0), "sizen");
        assert_eq!(lex.features.get(1), "gengo,げんご");
        assert_eq!(lex.lex_type, LexType::System);
    }

    #[test]
    fn test_user() {
        let data = "自然,0,2,1,sizen\n言語,1,0,-4,gengo,げんご";
        let lex = Lexicon::from_reader(data.as_bytes(), LexType::User).unwrap();
        assert_eq!(lex.params.get(0), WordParam::new(0, 2, 1));
        assert_eq!(lex.params.get(1), WordParam::new(1, 0, -4));
        assert_eq!(lex.features.get(0), "sizen");
        assert_eq!(lex.features.get(1), "gengo,げんご");
        assert_eq!(lex.lex_type, LexType::User);
    }

    #[test]
    #[should_panic]
    fn test_few_cols() {
        let data = "自然,0,2";
        Lexicon::from_reader(data.as_bytes(), LexType::System).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_invalid_left_id() {
        let data = "自然,-2,2,1";
        Lexicon::from_reader(data.as_bytes(), LexType::System).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_invalid_right_id() {
        let data = "自然,2,-2,1";
        Lexicon::from_reader(data.as_bytes(), LexType::System).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_invalid_cost() {
        let data = "自然,2,1,コスト";
        Lexicon::from_reader(data.as_bytes(), LexType::System).unwrap();
    }
}
