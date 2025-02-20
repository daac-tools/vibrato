use std::io::Read;

use bincode::{Decode, Encode};

use crate::dictionary::character::{CharInfo, CharProperty};
use crate::dictionary::connector::Connector;
use crate::dictionary::lexicon::{Lexicon, WordParam};
use crate::dictionary::mapper::ConnIdMapper;
use crate::dictionary::word_idx::WordIdx;
use crate::dictionary::LexType;
use crate::errors::{Result, VibratoError};
use crate::sentence::Sentence;
use crate::utils::FromU32;

#[cfg(feature = "train")]
use crate::utils;

use crate::common::MAX_SENTENCE_LENGTH;

#[derive(Default, Debug, Clone, Decode, Encode, PartialEq, Eq)]
pub struct UnkEntry {
    pub cate_id: u16,
    pub left_id: u16,
    pub right_id: u16,
    pub word_cost: i16,
    pub feature: String,
}

#[derive(Default, Debug, Clone)]
pub struct UnkWord {
    start_char: usize,
    end_char: usize,
    left_id: u16,
    right_id: u16,
    word_cost: i16,
    word_id: u16,
}

impl UnkWord {
    #[inline(always)]
    pub const fn start_char(&self) -> usize {
        self.start_char
    }

    #[inline(always)]
    pub const fn end_char(&self) -> usize {
        self.end_char
    }

    #[inline(always)]
    pub const fn word_param(&self) -> WordParam {
        WordParam::new(self.left_id, self.right_id, self.word_cost)
    }

    #[inline(always)]
    pub fn word_idx(&self) -> WordIdx {
        WordIdx::new(LexType::Unknown, u32::from(self.word_id))
    }
}

/// Handler of unknown words.
#[derive(Decode, Encode)]
pub struct UnkHandler {
    offsets: Vec<usize>, // indexed by category id
    entries: Vec<UnkEntry>,
}

impl UnkHandler {
    pub fn gen_unk_words<F>(
        &self,
        sent: &Sentence,
        start_char: usize,
        mut has_matched: bool,
        max_grouping_len: Option<usize>,
        mut f: F,
    ) where
        F: FnMut(UnkWord),
    {
        let cinfo = sent.char_info(start_char);
        if has_matched && !cinfo.invoke() {
            return;
        }

        let mut grouped = false;
        let groupable = sent.groupable(start_char);
        debug_assert_ne!(groupable, 0);

        if cinfo.group() {
            grouped = true;
            // Checks the number of grouped characters other than the first one
            // following the original MeCab implementation.
            let max_grouping_len = max_grouping_len.map_or(MAX_SENTENCE_LENGTH, |l| l);
            // Note: Do NOT write `max_grouping_len+1` to avoid overflow.
            if groupable - 1 <= max_grouping_len {
                f = self.scan_entries(start_char, start_char + groupable, cinfo, f);
                has_matched = true;
            }
        }

        for i in 1..=usize::from(cinfo.length()).min(groupable) {
            if grouped && i == groupable {
                continue;
            }
            let end_char = start_char + i;
            if sent.len_char() < end_char {
                break;
            }
            f = self.scan_entries(start_char, end_char, cinfo, f);
            has_matched = true;
        }

        // Generates at least one unknown word.
        if !has_matched {
            self.scan_entries(start_char, start_char + 1, cinfo, f);
        }
    }

    #[inline(always)]
    fn scan_entries<F>(&self, start_char: usize, end_char: usize, cinfo: CharInfo, mut f: F) -> F
    where
        F: FnMut(UnkWord),
    {
        let start = self.offsets[usize::from_u32(cinfo.base_id())];
        let end = self.offsets[usize::from_u32(cinfo.base_id()) + 1];
        for word_id in start..end {
            let e = &self.entries[word_id];
            f(UnkWord {
                start_char,
                end_char,
                left_id: e.left_id,
                right_id: e.right_id,
                word_cost: e.word_cost,
                word_id: word_id as u16,
            });
        }
        f
    }

    /// Returns the earliest occurrence of compatible unknown words for the given word.
    ///
    /// Returns `None` if no compatible entry exists.
    #[cfg(feature = "train")]
    pub fn compatible_unk_index(
        &self,
        sent: &Sentence,
        start_char: usize,
        end_char: usize,
        feature: &str,
    ) -> Option<WordIdx> {
        let features = utils::parse_csv_row(feature);

        let cinfo = sent.char_info(start_char);

        let groupable = sent.groupable(start_char);

        if cinfo.group() || end_char - start_char <= usize::from(cinfo.length()).min(groupable) {
            let start = self.offsets[usize::from_u32(cinfo.base_id())];
            let end = self.offsets[usize::from_u32(cinfo.base_id()) + 1];
            'a: for word_id in start..end {
                let e = &self.entries[word_id];
                let unk_features = utils::parse_csv_row(&e.feature);
                for (i, unk_feature) in unk_features.iter().enumerate() {
                    if unk_feature != "*" && (features.get(i) != Some(unk_feature)) {
                        continue 'a;
                    }
                }
                return Some(WordIdx::new(
                    LexType::Unknown,
                    u32::try_from(word_id).unwrap(),
                ));
            }
        }

        None
    }

    #[inline(always)]
    pub fn word_param(&self, word_idx: WordIdx) -> WordParam {
        debug_assert_eq!(word_idx.lex_type, LexType::Unknown);
        let e = &self.entries[usize::from_u32(word_idx.word_id)];
        WordParam::new(e.left_id, e.right_id, e.word_cost)
    }

    #[inline(always)]
    pub fn word_feature(&self, word_idx: WordIdx) -> &str {
        debug_assert_eq!(word_idx.lex_type, LexType::Unknown);
        &self.entries[usize::from_u32(word_idx.word_id)].feature
    }

    #[cfg(feature = "train")]
    #[inline(always)]
    pub fn word_cate_id(&self, word_idx: WordIdx) -> u16 {
        debug_assert_eq!(word_idx.lex_type, LexType::Unknown);
        self.entries[usize::from_u32(word_idx.word_id)].cate_id
    }

    #[cfg(feature = "train")]
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Do NOT make this function public to maintain consistency in
    /// the connection-id mapping among members of `Dictionary`.
    /// The consistency is managed in `Dictionary`.
    pub fn map_connection_ids(&mut self, mapper: &ConnIdMapper) {
        for e in &mut self.entries {
            e.left_id = mapper.left(e.left_id);
            e.right_id = mapper.right(e.right_id);
        }
    }

    /// Checks if left/right-ids are valid to the connector.
    pub fn verify<C>(&self, conn: &C) -> bool
    where
        C: Connector,
    {
        for e in &self.entries {
            if conn.num_left() <= usize::from(e.left_id) {
                return false;
            }
            if conn.num_right() <= usize::from(e.right_id) {
                return false;
            }
        }
        true
    }

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

    #[cfg(feature = "train")]
    const CHAR_DEF: &'static str = "\
DEFAULT 0 1 0
ALPHA   1 1 6
NUMERIC 1 1 0
0x0030..0x0039 NUMERIC
0x0041..0x005A ALPHA NUMERIC
0x0061..0x007A ALPHA NUMERIC";
    #[cfg(feature = "train")]
    const UNK_DEF: &'static str = "\
DEFAULT,0,0,0,補助記号,*
ALPHA,0,0,0,名詞,*,変数
ALPHA,0,0,0,動詞,*
NUMERIC,0,0,0,数字";

    #[cfg(feature = "train")]
    #[test]
    fn test_compatible_unk_entry_1() {
        let prop = CharProperty::from_reader(CHAR_DEF.as_bytes()).unwrap();
        let unk = UnkHandler::from_reader(UNK_DEF.as_bytes(), &prop).unwrap();

        let mut sent = Sentence::new();
        sent.set_sentence("変数var42を書き換えます");
        sent.compile(&prop);

        let unk_index = unk
            .compatible_unk_index(&sent, 2, 7, "名詞,一般,変数,バーヨンジューニ")
            .unwrap();
        assert_eq!(unk.word_feature(unk_index), "名詞,*,変数");
    }

    #[cfg(feature = "train")]
    #[test]
    fn test_compatible_unk_entry_2() {
        let prop = CharProperty::from_reader(CHAR_DEF.as_bytes()).unwrap();
        let unk = UnkHandler::from_reader(UNK_DEF.as_bytes(), &prop).unwrap();

        let mut sent = Sentence::new();
        sent.set_sentence("変数var42を書き換えます");
        sent.compile(&prop);

        let unk_index = unk
            .compatible_unk_index(&sent, 2, 7, "動詞,一般,変数,バーヨンジューニ")
            .unwrap();
        assert_eq!(unk.word_feature(unk_index), "動詞,*");
    }

    #[cfg(feature = "train")]
    #[test]
    fn test_compatible_unk_entry_3() {
        let prop = CharProperty::from_reader(CHAR_DEF.as_bytes()).unwrap();
        let unk = UnkHandler::from_reader(UNK_DEF.as_bytes(), &prop).unwrap();

        let mut sent = Sentence::new();
        sent.set_sentence("変数var42を書き換えます");
        sent.compile(&prop);

        let unk_index = unk
            .compatible_unk_index(&sent, 5, 7, "数字,一般,変数末尾,ヨンジューニ")
            .unwrap();
        assert_eq!(unk.word_feature(unk_index), "数字");
    }

    #[cfg(feature = "train")]
    #[test]
    fn test_compatible_unk_entry_undefined_1() {
        let prop = CharProperty::from_reader(CHAR_DEF.as_bytes()).unwrap();
        let unk = UnkHandler::from_reader(UNK_DEF.as_bytes(), &prop).unwrap();

        let mut sent = Sentence::new();
        sent.set_sentence("変数var42を書き換えます");
        sent.compile(&prop);

        assert!(unk.compatible_unk_index(&sent, 2, 7, "形容詞").is_none());
    }

    #[cfg(feature = "train")]
    #[test]
    fn test_compatible_unk_entry_undefined_2() {
        let prop = CharProperty::from_reader(CHAR_DEF.as_bytes()).unwrap();
        let unk = UnkHandler::from_reader(UNK_DEF.as_bytes(), &prop).unwrap();

        let mut sent = Sentence::new();
        sent.set_sentence("変数var42を書き換えます");
        sent.compile(&prop);

        assert!(unk
            .compatible_unk_index(&sent, 5, 7, "名詞,一般,変数,バーヨンジューニ")
            .is_none());
    }

    #[test]
    fn test_from_reader_basic() {
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
    fn test_from_reader_few_cols() {
        let char_def = "DEFAULT 0 1 0";
        let unk_def = "DEFAULT,0,2";
        let prop = CharProperty::from_reader(char_def.as_bytes()).unwrap();
        let result = UnkHandler::from_reader(unk_def.as_bytes(), &prop);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_reader_invalid_cate() {
        let char_def = "DEFAULT 0 1 0";
        let unk_def = "INVALID,0,2,1,補助記号";
        let prop = CharProperty::from_reader(char_def.as_bytes()).unwrap();
        let result = UnkHandler::from_reader(unk_def.as_bytes(), &prop);
        assert!(result.is_err());
    }
}
