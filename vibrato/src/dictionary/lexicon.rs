mod feature;
mod map;
mod param;

use std::io::Read;

use bincode::{Decode, Encode};
use csv_core::ReadFieldResult;

use crate::dictionary::connector::Connector;
use crate::dictionary::lexicon::feature::WordFeatures;
use crate::dictionary::lexicon::map::WordMap;
use crate::dictionary::lexicon::param::WordParams;
use crate::dictionary::mapper::ConnIdMapper;
use crate::dictionary::word_idx::WordIdx;
use crate::dictionary::LexType;
use crate::errors::{Result, VibratoError};
use crate::utils::FromU32;

pub use crate::dictionary::lexicon::param::WordParam;

/// Lexicon of words.
#[derive(Decode, Encode)]
pub struct Lexicon {
    map: WordMap,
    params: WordParams,
    features: WordFeatures,
    lex_type: LexType,
}

impl Lexicon {
    #[inline(always)]
    pub fn common_prefix_iterator<'a>(
        &'a self,
        input: &'a [char],
    ) -> impl Iterator<Item = LexMatch> + 'a {
        self.map
            .common_prefix_iterator(input)
            .map(move |(word_id, end_char)| {
                LexMatch::new(
                    WordIdx::new(self.lex_type, word_id),
                    self.params.get(usize::from_u32(word_id)),
                    end_char,
                )
            })
    }

    #[inline(always)]
    pub unsafe fn common_prefix_iterator_unchecked<'a>(
        &'a self,
        input: &'a [char],
    ) -> impl Iterator<Item = LexMatch> + 'a {
        self.map
            .common_prefix_iterator_unchecked(input)
            .map(move |(word_id, end_char)| {
                LexMatch::new(
                    WordIdx::new(self.lex_type, word_id),
                    self.params.get(usize::from_u32(word_id)),
                    end_char,
                )
            })
    }

    /// Do NOT make this function public to maintain consistency in
    /// the connection-id mapping among members of `Dictionary`.
    /// The consistency is managed in `Dictionary`.
    pub fn map_connection_ids(&mut self, mapper: &ConnIdMapper) {
        self.params.map_connection_ids(mapper);
    }

    #[inline(always)]
    pub fn word_param(&self, word_idx: WordIdx) -> WordParam {
        debug_assert_eq!(word_idx.lex_type, self.lex_type);
        self.params.get(usize::from_u32(word_idx.word_id))
    }

    #[inline(always)]
    pub fn word_feature(&self, word_idx: WordIdx) -> &str {
        debug_assert_eq!(word_idx.lex_type, self.lex_type);
        self.features.get(usize::from_u32(word_idx.word_id))
    }

    /// Checks if left/right-ids are valid with connector.
    pub fn verify<C>(&self, conn: &C) -> bool
    where
        C: Connector,
    {
        for i in 0..self.params.len() {
            let p = self.params.get(i);
            if conn.num_left() <= usize::from(p.left_id) {
                return false;
            }
            if conn.num_right() <= usize::from(p.right_id) {
                return false;
            }
        }
        true
    }

    /// Builds a new instance from a list of entries.
    pub fn from_entries(entries: &[RawWordEntry], lex_type: LexType) -> Result<Self> {
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

    /// Builds a new instance from a lexicon file in the CSV format.
    pub fn from_reader<R>(mut rdr: R, lex_type: LexType) -> Result<Self>
    where
        R: Read,
    {
        let mut buf = vec![];
        rdr.read_to_end(&mut buf)?;

        let entries = Self::parse_csv(&buf, "lex.csv")?;

        Self::from_entries(&entries, lex_type)
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
                if surface.is_empty() {
                    eprintln!(
                        "Skipped an empty surface, {:?}",
                        std::str::from_utf8(&record_bytes[..record_end_pos])?,
                    );
                } else {
                    entries.push(RawWordEntry {
                        surface,
                        param: WordParam::new(left_id, right_id, word_cost),
                        feature,
                    });
                }
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

#[derive(Eq, PartialEq, Debug)]
pub struct LexMatch {
    pub word_idx: WordIdx,
    pub word_param: WordParam,
    pub end_char: usize,
}

impl LexMatch {
    #[inline(always)]
    pub const fn new(word_idx: WordIdx, word_param: WordParam, end_char: usize) -> Self {
        Self {
            word_idx,
            word_param,
            end_char,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RawWordEntry<'a> {
    pub surface: String,
    pub param: WordParam,
    pub feature: &'a str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_prefix_iterator() {
        let lexicon = Lexicon {
            map: WordMap::new(["東京", "東京都", "東京", "京都"]).unwrap(),
            params: WordParams::new([
                WordParam::new(1, 2, 3),
                WordParam::new(4, 5, 6),
                WordParam::new(7, 8, 9),
                WordParam::new(10, 11, 12),
            ]),
            features: WordFeatures::default(),
            lex_type: LexType::System,
        };
        let input: Vec<_> = "東京都".chars().collect();
        let mut it = lexicon.common_prefix_iterator(&input);
        assert_eq!(
            it.next().unwrap(),
            LexMatch {
                end_char: 2,
                word_idx: WordIdx::new(LexType::System, 0),
                word_param: WordParam::new(1, 2, 3),
            }
        );
        assert_eq!(
            it.next().unwrap(),
            LexMatch {
                end_char: 2,
                word_idx: WordIdx::new(LexType::System, 2),
                word_param: WordParam::new(7, 8, 9),
            }
        );
        assert_eq!(
            it.next().unwrap(),
            LexMatch {
                end_char: 3,
                word_idx: WordIdx::new(LexType::System, 1),
                word_param: WordParam::new(4, 5, 6),
            }
        );
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_from_reader_system() {
        let data = "自然,0,2,1,sizen\n言語,1,0,-4,gengo,げんご";
        let lex = Lexicon::from_reader(data.as_bytes(), LexType::System).unwrap();
        assert_eq!(lex.params.get(0), WordParam::new(0, 2, 1));
        assert_eq!(lex.params.get(1), WordParam::new(1, 0, -4));
        assert_eq!(lex.features.get(0), "sizen");
        assert_eq!(lex.features.get(1), "gengo,げんご");
        assert_eq!(lex.lex_type, LexType::System);
    }

    #[test]
    fn test_from_reader_user() {
        let data = "自然,0,2,1,sizen\n言語,1,0,-4,gengo,げんご";
        let lex = Lexicon::from_reader(data.as_bytes(), LexType::User).unwrap();
        assert_eq!(lex.params.get(0), WordParam::new(0, 2, 1));
        assert_eq!(lex.params.get(1), WordParam::new(1, 0, -4));
        assert_eq!(lex.features.get(0), "sizen");
        assert_eq!(lex.features.get(1), "gengo,げんご");
        assert_eq!(lex.lex_type, LexType::User);
    }

    #[test]
    fn test_parse_csv_empty_surface() {
        let data = "自然,0,2,1,sizen\n,1,0,-4,gengo,げんご";
        let result = Lexicon::parse_csv(data.as_bytes(), "test").unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_from_reader_few_cols() {
        let data = "自然,0,2";
        let result = Lexicon::from_reader(data.as_bytes(), LexType::System);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_reader_invalid_left_id() {
        let data = "自然,-2,2,1,a";
        let result = Lexicon::from_reader(data.as_bytes(), LexType::System);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_reader_invalid_right_id() {
        let data = "自然,2,-2,1,a";
        let result = Lexicon::from_reader(data.as_bytes(), LexType::System);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_reader_invalid_cost() {
        let data = "自然,2,1,コスト,a";
        let result = Lexicon::from_reader(data.as_bytes(), LexType::System);
        assert!(result.is_err());
    }
}
