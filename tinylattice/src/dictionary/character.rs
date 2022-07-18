pub mod builder;
pub mod category;

pub use category::CategorySet;

#[derive(Default, Debug, Clone, Copy)]
pub struct CharInfo {
    pub base_id: u32,
    pub cate_ids: CategorySet,
    pub invoke: bool,
    pub group: bool,
    pub length: u16,
}

pub struct CharProperty {
    chr2inf: Vec<CharInfo>,
}

impl CharProperty {
    pub fn char_info(&self, c: char) -> CharInfo {
        let c = c as usize;
        if let Some(inf) = self.chr2inf.get(c) {
            inf.clone()
        } else {
            self.chr2inf[0].clone()
        }
    }
}
