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
    pub fn begin_char(&self) -> usize {
        self.begin_char as usize
    }

    pub fn end_char(&self) -> usize {
        self.end_char as usize
    }

    pub fn word_param(&self) -> WordParam {
        WordParam::new(self.left_id, self.right_id, self.word_cost)
    }

    pub fn word_idx(&self) -> WordIdx {
        WordIdx::new(LexType::Unknown, self.word_id as u32)
    }
}

pub struct UnkHandler {
    // indexed by category id
    entries: Vec<Vec<UnkEntry>>,
}

impl UnkHandler {
    pub fn gen_unk_words(
        &self,
        sent: &Sentence,
        pos_char: usize,
        has_matched: bool,
    ) -> Vec<UnkWord> {
        let cinfo = sent.char_info(pos_char);

        let mut unk_words = vec![];
        if has_matched && !cinfo.invoke {
            return unk_words;
        }

        let glen = sent.groupable(pos_char);

        if cinfo.group {
            if glen < MAX_GROUPING_LEN {
                self.create_unk_words(pos_char, pos_char + glen, cinfo, &mut unk_words);
            }
        }

        for i in 1..=cinfo.length as usize {
            if i == glen {
                continue;
            }
            let end_char = pos_char + i;
            if sent.chars().len() < end_char {
                break;
            }
            self.create_unk_words(pos_char, end_char, cinfo, &mut unk_words);
        }
        unk_words
    }

    fn create_unk_words(
        &self,
        begin_char: usize,
        end_char: usize,
        cinfo: CharInfo,
        words: &mut Vec<UnkWord>,
    ) {
        let entries = &self.entries[cinfo.base_id as usize];
        for e in entries {
            words.push(UnkWord {
                begin_char: begin_char as u16,
                end_char: end_char as u16,
                left_id: e.left_id,
                right_id: e.right_id,
                word_cost: e.word_cost,
                word_id: e.cate_id,
            });
        }
    }
}
