#![allow(dead_code)]

pub mod parser;
pub mod simple;

use std::collections::HashMap;

use super::{CategoryTypes, LexType, WordIdx, WordParam};

pub use simple::SimpleUnkHandler;
// use crate::Sentence;

#[derive(Default, Debug, Clone, Copy)]
pub struct CategoryDef {
    cate_type: CategoryTypes,
    is_invoke: bool,
    is_group: bool,
    length: u32,
}

#[derive(Default, Debug, Clone)]
pub struct UnkEntry {
    pub cate_type: CategoryTypes,
    pub left_id: i16,
    pub right_id: i16,
    pub word_cost: i16,
    pub feature: String,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct UnkWord {
    char_begin: u16,
    char_end: u16,
    left_id: i16,
    right_id: i16,
    word_cost: i16,
    word_id: u16,
}

impl UnkWord {
    pub fn char_begin(&self) -> usize {
        self.char_begin as usize
    }

    pub fn char_end(&self) -> usize {
        self.char_end as usize
    }

    pub fn word_param(&self) -> WordParam {
        WordParam::new(self.left_id, self.right_id, self.word_cost)
    }

    pub fn word_idx(&self) -> WordIdx {
        WordIdx::new(LexType::Unknown, self.word_id as u32)
    }
}

pub struct UnkHandler {
    cate_def_map: HashMap<CategoryTypes, CategoryDef>,
    unk_entries_map: HashMap<CategoryTypes, Vec<UnkEntry>>,
}

impl UnkHandler {
    pub fn new(_cate_defs: Vec<CategoryDef>, _unk_entries: Vec<UnkEntry>) -> Self {
        Self {
            cate_def_map: HashMap::new(),
            unk_entries_map: HashMap::new(),
        }
    }

    // pub fn unk_words(&self, sentence: &Sentence, char_pos: usize) -> Vec<UnkWord> {
    //     let concatable = sentence.concatable(char_pos);
    //     debug_assert_ne!(concatable, 0);

    //     let mut buf = vec![];
    //     for cate_def in sentence
    //         .category(char_pos)
    //         .iter()
    //         .flat_map(|ct| self.cate_def_map.get(&ct))
    //         .filter(|&cd| cd.is_invoke)
    //     {
    //         let entries = match self.unk_entries_map.get(&cate_def.cate_type) {
    //             Some(e) => e,
    //             None => continue,
    //         };

    //         let mut len = concatable;

    //         if cate_def.is_group {
    //             for e in entries {
    //                 buf.push(UnkWord {
    //                     char_begin: char_pos as u16,
    //                     char_end: (char_pos + concatable) as u16,
    //                     left_id: e.left_id,
    //                     right_id: e.right_id,
    //                     word_cost: e.word_cost,
    //                     word_id: 0,
    //                 });
    //             }
    //             len -= 1;
    //         }
    //         for i in 1..=cate_def.length {
    //             let sublen = (char_pos + i as usize).min(sentence.chars().len());
    //             if sublen > len {
    //                 break;
    //             }
    //             for e in entries {}

    //             for oov in oovs {
    //                 nodes.push(self.get_oov_node(oov, offset, offset + sublength));
    //                 num_created += 1;
    //             }
    //         }
    //     }
    //     buf
    // }
}
