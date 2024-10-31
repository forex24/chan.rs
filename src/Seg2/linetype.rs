use crate::{
    Common::{types::Handle, CEnum::BiDir},
    KLine::KLine_Unit::CKLineUnit,
};
/*
pub enum LineType {
    Bi(Handle<CBi>),
    Seg(Handle<CSeg<CBi>>),
}

impl LineType {
    pub fn index(&self) -> i32 {
        match self {
            LineType::Bi(ref bi) => bi.borrow().idx,
            LineType::Seg(ref seg) => seg.borrow().idx,
        }
    }

    pub fn time_begin(&self) -> i32 {
        match self {
            LineType::Bi(ref bi) => bi.borrow().begin_klc.borrow().idx,
            LineType::Seg(ref seg) => seg.borrow().begin_klc.borrow().idx,
        }
    }

    pub fn time_end(&self) -> i32 {
        match self {
            LineType::Bi(ref bi) => bi.borrow().end_klc.borrow().idx,
            LineType::Seg(ref seg) => seg.borrow().begin_klc.borrow().idx,
        }
    }

    pub fn high(&self) -> f64 {
        match self {
            LineType::Bi(ref bi) => bi.borrow().high(),
            LineType::Seg(ref seg) => seg.borrow()._high(),
        }
    }

    pub fn low(&self) -> f64 {
        match self {
            LineType::Bi(ref bi) => bi.borrow().low(),
            LineType::Seg(ref seg) => seg.borrow()._low(),
        }
    }
}
*/
pub trait Line<T> {
    fn idx(&self) -> i32;

    fn is_sure(&self) -> bool;

    fn dir(&self) -> BiDir;
    fn is_down(&self) -> bool {
        self.dir() == BiDir::Down
    }

    fn is_up(&self) -> bool {
        self.dir() == BiDir::Up
    }

    fn set_parent_seg<U>(&mut self, parent_seg: Option<Handle<U>>);

    fn high(&self) -> f64;
    fn low(&self) -> f64;

    fn get_begin_val(&self) -> f64;
    fn get_end_val(&self) -> f64;

    fn get_begin_klu(&self) -> Handle<CKLineUnit>;
    fn get_end_klu(&self) -> Handle<CKLineUnit>;

    fn next(&self) -> Option<Handle<Self>>;
    fn pre(&self) -> Option<Handle<Self>>;

    fn update_bi_list(&self);
}
