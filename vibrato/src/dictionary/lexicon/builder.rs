use std::io::Read;

use csv_core::ReadFieldResult;

use crate::dictionary::lexicon::{
    LexType, Lexicon, RawWordEntry, WordFeatures, WordMap, WordParam, WordParams,
};
use crate::errors::{Result, VibratoError};

impl Lexicon {
    /// Builds a new instance from a lexicon file in the CSV format.
    pub fn from_reader<R>(mut rdr: R, lex_type: LexType) -> Result<Self>
    where
        R: Read,
    {
        let mut buf = vec![];
        rdr.read_to_end(&mut buf)?;

        let entries = Self::parse_csv(&buf, "lex.csv")?;

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

    pub(crate) fn parse_csv<'a>(
        mut bytes: &'a [u8],
        name: &'static str,
    ) -> Result<Vec<RawWordEntry<'a>>> {
        let mut entries = vec![];

        let mut rdr = csv_core::Reader::new();
        let mut features_bytes = bytes;
        let mut record_bytes = bytes;
        let mut field_cnt: usize = 0;
        let mut features_len = 0;
        let mut record_end_pos = 0;
        let mut output = [0; 4096];

        let mut surface = String::new();
        let mut left_id = 0;
        let mut right_id = 0;
        let mut word_cost = 0;

        loop {
            let (result, nin, nout) = rdr.read_field(bytes, &mut output);
            let record_end = match result {
                ReadFieldResult::InputEmpty => {
                    features_len += nin + 1;
                    record_end_pos += nin;
                    true
                }
                ReadFieldResult::OutputFull => {
                    return Err(VibratoError::invalid_format(name, "Field too large"))
                }
                ReadFieldResult::Field { record_end } => {
                    match field_cnt {
                        0 => {
                            surface = std::str::from_utf8(&output[..nout])?.to_string();
                            record_bytes = bytes;
                        }
                        1 => {
                            left_id = std::str::from_utf8(&output[..nout])?.parse()?;
                        }
                        2 => {
                            right_id = std::str::from_utf8(&output[..nout])?.parse()?;
                        }
                        3 => {
                            word_cost = std::str::from_utf8(&output[..nout])?.parse()?;
                            features_bytes = &bytes[nin..];
                            features_len = 0;
                        }
                        _ => {
                            features_len += nin;
                        }
                    }
                    record_end_pos += nin;
                    record_end
                }
                ReadFieldResult::End => break,
            };
            if record_end {
                if field_cnt == 0 && nin == 0 {
                    continue;
                }
                if field_cnt <= 3 {
                    let msg = format!(
                        "A csv row of lexicon must have five items at least, {:?}",
                        std::str::from_utf8(&record_bytes[..record_end_pos])?,
                    );
                    return Err(VibratoError::invalid_format(name, msg));
                }
                let feature = std::str::from_utf8(&features_bytes[..features_len - 1])?;
                entries.push(RawWordEntry {
                    surface,
                    param: WordParam::new(left_id, right_id, word_cost),
                    feature,
                });
                surface = String::new();
                field_cnt = 0;
                record_end_pos = 0;
            } else {
                field_cnt += 1;
            }
            bytes = &bytes[nin..];
        }
        Ok(entries)
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
    fn test_few_cols() {
        let data = "自然,0,2";
        let lex = Lexicon::from_reader(data.as_bytes(), LexType::System);
        assert!(lex.is_err());
    }

    #[test]
    fn test_invalid_left_id() {
        let data = "自然,-2,2,1,a";
        let lex = Lexicon::from_reader(data.as_bytes(), LexType::System);
        assert!(lex.is_err());
    }

    #[test]
    fn test_invalid_right_id() {
        let data = "自然,2,-2,1,a";
        let lex = Lexicon::from_reader(data.as_bytes(), LexType::System);
        assert!(lex.is_err());
    }

    #[test]
    fn test_invalid_cost() {
        let data = "自然,2,1,コスト,a";
        let lex = Lexicon::from_reader(data.as_bytes(), LexType::System);
        assert!(lex.is_err());
    }
}
