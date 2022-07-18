pub mod builder;
pub mod category;

pub use category::CategorySet;

#[derive(Default, Debug, Clone, Copy)]
pub struct CharInfo {
    base_id: u32,
    cate_ids: CategorySet,
    invoke: bool,
    group: bool,
    length: u16,
}

impl CharInfo {
    pub fn new(
        base_id: u32,
        cate_ids: CategorySet,
        invoke: bool,
        group: bool,
        length: usize,
    ) -> Self {
        Self {
            base_id,
            cate_ids,
            invoke,
            group,
            length: length as u16,
        }
    }

    pub fn base_id(&self) -> u32 {
        self.base_id
    }

    pub fn cate_ids(&self) -> CategorySet {
        self.cate_ids
    }

    pub fn invoke(&self) -> bool {
        self.invoke
    }

    pub fn group(&self) -> bool {
        self.group
    }

    pub fn length(&self) -> usize {
        self.length as usize
    }
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
