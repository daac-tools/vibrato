use std::io::Read;

use crate::dictionary::character::CharProperty;
use crate::dictionary::lexicon::Lexicon;
use crate::dictionary::unknown::{UnkEntry, UnkHandler};
use crate::errors::{Result, VibratoError};

impl UnkHandler {
    /// Creates a new instance from `unk.def`.
    pub fn from_reader<R>(mut rdr: R, char_prop: &CharProperty) -> Result<Self>
    where
        R: Read,
    {
        let mut buf = vec![];
        rdr.read_to_end(&mut buf)?;

        let parsed = Lexicon::parse_csv(&buf, "unk.def")?;
        let mut map = vec![vec![]; char_prop.num_categories()];
        for item in parsed {
            let cate_id = u16::try_from(char_prop.cate_id(&item.surface).ok_or_else(|| {
                let msg = format!("Undefined category: {}", item.surface);
                VibratoError::invalid_format("unk.def", msg)
            })?)
            .unwrap();
            let e = UnkEntry {
                cate_id,
                left_id: item.param.left_id,
                right_id: item.param.right_id,
                word_cost: item.param.word_cost,
                feature: item.feature.to_string(),
            };
            map[usize::from(cate_id)].push(e);
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let char_def = "DEFAULT 0 1 0\nSPACE 0 1 0\nALPHA 1 1 0";
        let unk_def = "DEFAULT,0,2,1,補助記号\nALPHA,1,0,-4,名詞\nALPHA,2,2,3,Meishi";
        let prop = CharProperty::from_reader(char_def.as_bytes()).unwrap();
        let unk = UnkHandler::from_reader(unk_def.as_bytes(), &prop).unwrap();
        assert_eq!(
            unk.offsets,
            vec![
                0, //DEFAULT = 0
                1, 1, // ALPHA = 2
                3,
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
                    cate_id: 2,
                    left_id: 1,
                    right_id: 0,
                    word_cost: -4,
                    feature: "名詞".to_string(),
                },
                UnkEntry {
                    cate_id: 2,
                    left_id: 2,
                    right_id: 2,
                    word_cost: 3,
                    feature: "Meishi".to_string(),
                }
            ]
        );
    }

    #[test]
    fn test_few_cols() {
        let char_def = "DEFAULT 0 1 0";
        let unk_def = "DEFAULT,0,2";
        let prop = CharProperty::from_reader(char_def.as_bytes()).unwrap();
        let result = UnkHandler::from_reader(unk_def.as_bytes(), &prop);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_cate() {
        let char_def = "DEFAULT 0 1 0";
        let unk_def = "INVALID,0,2,1,補助記号";
        let prop = CharProperty::from_reader(char_def.as_bytes()).unwrap();
        let result = UnkHandler::from_reader(unk_def.as_bytes(), &prop);
        assert!(result.is_err());
    }
}
