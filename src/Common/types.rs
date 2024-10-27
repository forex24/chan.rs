use maybe_atomic_refcell::MaybeAtomicRefCell;
use std::{cell::RefCell, rc::Rc};

use crate::{Bi::Bi::CBi, Seg::Seg::CSeg};

pub type SharedCell<T> = Rc<RefCell<T>>;

pub enum LineType {
    Bi(SharedCell<CBi>),
    Seg(SharedCell<CSeg<CBi>>),
}

// 创建一个辅助函数来简化 SharedCell 的创建
pub fn new_shared_cell<T>(value: T) -> SharedCell<T> {
    Rc::new(MaybeAtomicRefCell::new(value))
}
