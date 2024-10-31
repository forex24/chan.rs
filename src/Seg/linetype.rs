use std::rc::Rc;

use crate::{
    Bi::Bi::CBi,
    Common::{types::Handle, CEnum::BiDir},
    KLine::{KLine::CKLine, KLine_Unit::CKLineUnit},
};

use super::Seg::CSeg;

pub trait Line {
    // 读取属性
    fn idx(&self) -> usize;
    fn high(&self) -> f64;
    fn low(&self) -> f64;
    fn get_begin_val(&self) -> f64;
    fn get_end_val(&self) -> f64;
    fn get_begin_klu(&self) -> Handle<CKLineUnit>;
    fn get_end_klu(&self) -> Handle<CKLineUnit>;
    fn dir(&self) -> BiDir;
    fn get_pre(&self) -> Option<Handle<Self>>;
    fn get_next(&self) -> Option<Handle<Self>>;
    fn set_parent_seg<T>(&mut self, parent_seg: Option<T>);
    // 修改属性
    fn set_pre(&mut self, pre: Option<Handle<Self>>);
    fn set_next(&mut self, next: Option<Handle<Self>>);

    fn get_begin_klc(&self) -> Handle<CKLine>;
    fn get_end_klc(&self) -> Handle<CKLine>;
    // 默认方法实现
    fn is_up(&self) -> bool {
        self.dir() == BiDir::Up
    }

    fn is_down(&self) -> bool {
        self.dir() == BiDir::Down
    }

    fn is_sure(&self) -> bool;
    fn next(&self) -> Option<Handle<Self>>;
    fn pre(&self) -> Option<Handle<Self>>;
}

// 更新 CBi 的实现
impl Line for CBi {
    fn idx(&self) -> usize {
        self.idx as usize
    }

    fn high(&self) -> f64 {
        self._high()
    }

    fn low(&self) -> f64 {
        self._low()
    }

    fn get_begin_val(&self) -> f64 {
        self.get_begin_val()
    }

    fn get_end_val(&self) -> f64 {
        self.get_end_val()
    }

    fn dir(&self) -> BiDir {
        self.dir
    }

    fn get_pre(&self) -> Option<Handle<Self>> {
        self.pre.clone()
    }

    fn get_next(&self) -> Option<Handle<Self>> {
        self.next.clone()
    }

    fn set_pre(&mut self, pre: Option<Handle<Self>>) {
        self.pre = pre;
    }

    fn set_next(&mut self, next: Option<Handle<Self>>) {
        self.next = next;
    }

    fn get_begin_klu(&self) -> Handle<CKLineUnit> {
        self.get_begin_klu()
    }

    fn get_end_klu(&self) -> Handle<CKLineUnit> {
        self.get_end_klu()
    }

    fn set_parent_seg<T>(&mut self, parent_seg: Option<T>) {
        todo!()
    }

    fn get_begin_klc(&self) -> Handle<CKLine> {
        Rc::clone(&self.begin_klc)
    }

    fn get_end_klc(&self) -> Handle<CKLine> {
        Rc::clone(&self.end_klc)
    }

    fn is_sure(&self) -> bool {
        self.is_sure
    }

    fn next(&self) -> Option<Handle<Self>> {
        self.next.as_ref().map(|x| Rc::clone(x))
    }

    fn pre(&self) -> Option<Handle<Self>> {
        self.pre.as_ref().map(|x| Rc::clone(x))
    }
}

// 更新 CSeg 的实现
impl Line for CSeg<CBi> {
    fn idx(&self) -> usize {
        self.idx
    }

    fn high(&self) -> f64 {
        self._high()
    }

    fn low(&self) -> f64 {
        self._low()
    }

    fn get_begin_val(&self) -> f64 {
        self.get_begin_val()
    }

    fn get_end_val(&self) -> f64 {
        self.get_end_val()
    }

    fn dir(&self) -> BiDir {
        self.dir
    }

    fn get_pre(&self) -> Option<Handle<Self>> {
        self.pre.clone()
    }

    fn get_next(&self) -> Option<Handle<Self>> {
        self.next.clone()
    }

    fn set_pre(&mut self, pre: Option<Handle<Self>>) {
        self.pre = pre;
    }

    fn set_next(&mut self, next: Option<Handle<Self>>) {
        self.next = next;
    }

    fn get_begin_klu(&self) -> Handle<CKLineUnit> {
        self.get_begin_klu()
    }

    fn get_end_klu(&self) -> Handle<CKLineUnit> {
        self.get_end_klu()
    }

    fn set_parent_seg<T>(&mut self, parent_seg: Option<T>) {
        todo!()
    }

    fn get_begin_klc(&self) -> Handle<CKLine> {
        Rc::clone(&self.start_bi.borrow().begin_klc)
    }

    fn get_end_klc(&self) -> Handle<CKLine> {
        Rc::clone(&self.end_bi.borrow().end_klc)
    }

    fn is_sure(&self) -> bool {
        self.is_sure
    }

    fn next(&self) -> Option<Handle<Self>> {
        self.next.as_ref().map(|x| Rc::clone(x))
    }

    fn pre(&self) -> Option<Handle<Self>> {
        self.pre.as_ref().map(|x| Rc::clone(x))
    }
}
