pub mod builder;

use super::{LexType, WordIdx, WordParam};
use crate::dictionary::character::CharInfo;
use crate::Sentence;

const MAX_GROUPING_LEN: usize = 24;

#[derive(Default, Debug, Clone)]
pub struct UnkEntry {
    pub cate_id: u16,
    pub left_id: i16,
    pub right_id: i16,
    pub word_cost: i16,
    pub feature: String,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct UnkWord {
    begin_char: u16,
    end_char: u16,
    left_id: i16,
    right_id: i16,
    word_cost: i16,
    word_id: u16,
}

impl UnkWord {
    #[inline(always)]
    pub fn begin_char(&self) -> usize {
        self.begin_char as usize
    }

    #[inline(always)]
    pub fn end_char(&self) -> usize {
        self.end_char as usize
    }

    #[inline(always)]
    pub fn word_param(&self) -> WordParam {
        WordParam::new(self.left_id, self.right_id, self.word_cost)
    }

    #[inline(always)]
    pub fn word_idx(&self) -> WordIdx {
        WordIdx::new(LexType::Unknown, self.word_id as u32)
    }
}

pub struct UnkHandler {
    // indexed by category id
    offsets: Vec<usize>,
    entries: Vec<UnkEntry>,
}

impl UnkHandler {
    pub fn gen_unk_words(
        &self,
        sent: &Sentence,
        pos_char: usize,
        has_matched: bool,
        unk_words: &mut Vec<UnkWord>,
    ) {
        let cinfo = sent.char_info(pos_char);
        dbg!(pos_char, cinfo, has_matched);

        if has_matched && !cinfo.invoke {
            return;
        }

        let mut grouped = None;

        if cinfo.group {
            let glen = sent.groupable(pos_char);
            if glen < MAX_GROUPING_LEN {
                self.push_entries(pos_char, pos_char + glen, cinfo, unk_words);
            } else {
                grouped = Some(glen);
            }
        }
        dbg!(grouped);

        for i in 1..=cinfo.length as usize {
            dbg!(i);
            if i == grouped.unwrap_or(0) {
                continue;
            }
            let end_char = pos_char + i;
            dbg!(end_char);
            if sent.chars().len() < end_char {
                break;
            }
            self.push_entries(pos_char, end_char, cinfo, unk_words);
        }
    }

    fn push_entries(
        &self,
        begin_char: usize,
        end_char: usize,
        cinfo: CharInfo,
        unk_words: &mut Vec<UnkWord>,
    ) {
        let start = self.offsets[cinfo.base_id as usize];
        let end = self.offsets[cinfo.base_id as usize + 1];
        for word_id in start..end {
            let e = &self.entries[word_id];
            unk_words.push(UnkWord {
                begin_char: begin_char as u16,
                end_char: end_char as u16,
                left_id: e.left_id,
                right_id: e.right_id,
                word_cost: e.word_cost,
                word_id: word_id as u16,
            });
        }
    }
}
