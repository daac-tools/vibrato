mod builder;

use bincode::{Decode, Encode};

use crate::dictionary::character::CharInfo;
use crate::dictionary::connector::Connector;
use crate::dictionary::lexicon::WordParam;
use crate::dictionary::mapper::ConnIdMapper;
use crate::dictionary::word_idx::WordIdx;
use crate::dictionary::LexType;
use crate::sentence::Sentence;
use crate::utils::{self, FromU32};

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
    start_char: u16,
    end_char: u16,
    left_id: u16,
    right_id: u16,
    word_cost: i16,
    word_id: u16,
}

impl UnkWord {
    #[inline(always)]
    pub const fn start_char(&self) -> u16 {
        self.start_char
    }

    #[inline(always)]
    pub const fn end_char(&self) -> u16 {
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
        start_char: u16,
        mut has_matched: bool,
        max_grouping_len: Option<u16>,
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

        for i in 1..=cinfo.length().min(groupable) {
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
    fn scan_entries<F>(&self, start_char: u16, end_char: u16, cinfo: CharInfo, mut f: F) -> F
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
    pub fn compatible_unk_index(
        &self,
        sent: &Sentence,
        start_char: u16,
        end_char: u16,
        feature: &str,
    ) -> Option<WordIdx> {
        let features = utils::parse_csv_row(feature);

        let cinfo = sent.char_info(start_char);

        let groupable = sent.groupable(start_char);

        if cinfo.group() || end_char - start_char <= cinfo.length().min(groupable) {
            let start = self.offsets[usize::from_u32(cinfo.base_id())];
            let end = self.offsets[usize::from_u32(cinfo.base_id()) + 1];
            'a: for word_id in start..end {
                let e = &self.entries[word_id];
                let unk_features = utils::parse_csv_row(&e.feature);
                for (i, unk_feature) in unk_features.iter().enumerate() {
                    if unk_feature != "*" && features.get(i).map_or(true, |f| unk_feature != f) {
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

    #[inline(always)]
    pub fn word_cate_id(&self, word_idx: WordIdx) -> u16 {
        debug_assert_eq!(word_idx.lex_type, LexType::Unknown);
        self.entries[usize::from_u32(word_idx.word_id)].cate_id
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Do NOT make this function public to maintain consistency in
    /// the connection-id mapping among members of `Dictionary`.
    /// The consistency is managed in `Dictionary`.
    pub fn do_mapping(&mut self, mapper: &ConnIdMapper) {
        for e in &mut self.entries {
            e.left_id = mapper.left(e.left_id);
            e.right_id = mapper.right(e.right_id);
        }
    }

    /// Checks if left/right-ids are valid to the connector.
    pub fn verify(&self, conn: &Connector) -> bool {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::dictionary::CharProperty;

    const CHAR_DEF: &'static str = "\
DEFAULT 0 1 0
ALPHA   1 1 6
NUMERIC 1 1 0
0x0030..0x0039 NUMERIC
0x0041..0x005A ALPHA NUMERIC
0x0061..0x007A ALPHA NUMERIC";
    const UNK_DEF: &'static str = "\
DEFAULT,0,0,0,補助記号,*
ALPHA,0,0,0,名詞,*,変数
ALPHA,0,0,0,動詞,*
NUMERIC,0,0,0,数字";

    #[test]
    fn test_compatible_unk_entry_1() {
        let prop = CharProperty::from_reader(CHAR_DEF.as_bytes()).unwrap();
        let unk = UnkHandler::from_reader(UNK_DEF.as_bytes(), &prop).unwrap();

        let mut sent = Sentence::new();
        sent.set_sentence("変数var42を書き換えます");
        sent.compile(&prop).unwrap();

        let unk_index = unk
            .compatible_unk_index(&sent, 2, 7, "名詞,一般,変数,バーヨンジューニ")
            .unwrap();
        assert_eq!(unk.word_feature(unk_index), "名詞,*,変数");
    }

    #[test]
    fn test_compatible_unk_entry_2() {
        let prop = CharProperty::from_reader(CHAR_DEF.as_bytes()).unwrap();
        let unk = UnkHandler::from_reader(UNK_DEF.as_bytes(), &prop).unwrap();

        let mut sent = Sentence::new();
        sent.set_sentence("変数var42を書き換えます");
        sent.compile(&prop).unwrap();

        let unk_index = unk
            .compatible_unk_index(&sent, 2, 7, "動詞,一般,変数,バーヨンジューニ")
            .unwrap();
        assert_eq!(unk.word_feature(unk_index), "動詞,*");
    }

    #[test]
    fn test_compatible_unk_entry_3() {
        let prop = CharProperty::from_reader(CHAR_DEF.as_bytes()).unwrap();
        let unk = UnkHandler::from_reader(UNK_DEF.as_bytes(), &prop).unwrap();

        let mut sent = Sentence::new();
        sent.set_sentence("変数var42を書き換えます");
        sent.compile(&prop).unwrap();

        let unk_index = unk
            .compatible_unk_index(&sent, 5, 7, "数字,一般,変数末尾,ヨンジューニ")
            .unwrap();
        assert_eq!(unk.word_feature(unk_index), "数字");
    }

    #[test]
    fn test_compatible_unk_entry_undefined_1() {
        let prop = CharProperty::from_reader(CHAR_DEF.as_bytes()).unwrap();
        let unk = UnkHandler::from_reader(UNK_DEF.as_bytes(), &prop).unwrap();

        let mut sent = Sentence::new();
        sent.set_sentence("変数var42を書き換えます");
        sent.compile(&prop).unwrap();

        assert!(unk.compatible_unk_index(&sent, 2, 7, "形容詞").is_none());
    }

    #[test]
    fn test_compatible_unk_entry_undefined_2() {
        let prop = CharProperty::from_reader(CHAR_DEF.as_bytes()).unwrap();
        let unk = UnkHandler::from_reader(UNK_DEF.as_bytes(), &prop).unwrap();

        let mut sent = Sentence::new();
        sent.set_sentence("変数var42を書き換えます");
        sent.compile(&prop).unwrap();

        assert!(unk
            .compatible_unk_index(&sent, 5, 7, "名詞,一般,変数,バーヨンジューニ")
            .is_none());
    }
}
