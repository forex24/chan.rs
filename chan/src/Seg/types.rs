use crate::{
    Bi::{Bi::CBi, BiList::CBiList},
    Common::types::Handle,
};

// 首先定义一个 trait 来描述我们需要的功能
pub trait BiListTrait {
    type Item;
    fn get(&self, index: usize) -> Option<Self::Item>;
    fn iter(&self) -> impl Iterator<Item = Self::Item>;
}

impl BiListTrait for CBiList {
    type Item = Handle<CBi>; // T 是您原来的 bi 类型

    fn get(&self, index: usize) -> Option<Self::Item> {
        self.get(index)
    }

    fn iter(&self) -> impl Iterator<Item = Self::Item> {
        self.as_slice().iter().cloned() // Call the concrete implementation directly
    }
}
