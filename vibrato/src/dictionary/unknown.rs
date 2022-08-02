mod builder;

use bincode::{Decode, Encode};

use super::mapper::ConnIdMapper;
use super::{LexType, WordIdx};
use crate::dictionary::character::CharInfo;
use crate::dictionary::lexicon::WordParam;
use crate::sentence::Sentence;

#[derive(Default, Debug, Clone, Decode, Encode)]
pub struct UnkEntry {
    pub cate_id: u16,
    pub left_id: i16,
    pub right_id: i16,
    pub word_cost: i16,
    pub feature: String,
}

#[derive(Default, Debug, Clone)]
pub struct UnkWord {
    start_char: u16,
    end_char: u16,
    left_id: i16,
    right_id: i16,
    word_cost: i16,
    word_id: u16,
}

impl UnkWord {
    #[inline(always)]
    pub const fn start_char(&self) -> usize {
        self.start_char as usize
    }

    #[inline(always)]
    pub const fn end_char(&self) -> usize {
        self.end_char as usize
    }

    #[inline(always)]
    pub const fn word_param(&self) -> WordParam {
        WordParam::new(self.left_id, self.right_id, self.word_cost)
    }

    #[inline(always)]
    pub const fn word_idx(&self) -> WordIdx {
        WordIdx::new(LexType::Unknown, self.word_id as u32)
    }
}

/// Handler of unknown words.
#[derive(Decode, Encode)]
pub struct UnkHandler {
    offsets: Vec<usize>, // indexed by category id
    entries: Vec<UnkEntry>,
}

impl UnkHandler {
    pub(crate) fn gen_unk_words<F>(
        &self,
        sent: &Sentence,
        start_char: usize,
        mut has_matched: bool,
        max_grouping_len: usize,
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

        if cinfo.group() {
            grouped = true;
            if groupable <= max_grouping_len {
                f = self.scan_entries(start_char, start_char + groupable, cinfo, f);
                has_matched = true;
            }
        }

        for i in 1..=cinfo.length().min(groupable) {
            if grouped && i == groupable {
                continue;
            }
            let end_char = start_char + i;
            if sent.chars().len() < end_char {
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
        let start = self.offsets[cinfo.base_id() as usize];
        let end = self.offsets[cinfo.base_id() as usize + 1];
        for word_id in start..end {
            let e = &self.entries[word_id];
            f(UnkWord {
                start_char: start_char as u16,
                end_char: end_char as u16,
                left_id: e.left_id,
                right_id: e.right_id,
                word_cost: e.word_cost,
                word_id: word_id as u16,
            });
        }
        f
    }

    pub(crate) fn word_feature(&self, word_idx: WordIdx) -> &str {
        debug_assert_eq!(word_idx.lex_type(), LexType::Unknown);
        &self.entries[word_idx.word_id() as usize].feature
    }

    pub(crate) fn do_mapping(&mut self, mapper: &ConnIdMapper) {
        for e in &mut self.entries {
            e.left_id = mapper.left(e.left_id as u16) as i16;
            e.right_id = mapper.right(e.right_id as u16) as i16;
        }
    }
}
