use std::rc::Rc;

use crate::{
    Bi::Bi::CBi,
    BuySellPoint::BS_Point::CBSPoint,
    Common::{
        types::Handle,
        CEnum::{BiDir, MacdAlgo},
        ChanException::CChanException,
    },
    KLine::{KLine::CKLine, KLine_Unit::CKLineUnit},
};

use super::Seg::CSeg;
pub trait SegLine: Sized {
    fn __idx(&self) -> usize;
    fn __dir(&self) -> BiDir;
    fn __get_bi_list_len(&self) -> usize;
}

pub trait Line: Sized {
    type Parent: SegLine;
    // 读取属性
    fn _idx(&self) -> usize;
    fn _high(&self) -> f64;
    fn _low(&self) -> f64;
    fn _get_begin_val(&self) -> f64;
    fn _get_end_val(&self) -> f64;
    fn _get_begin_klu(&self) -> Handle<CKLineUnit>;
    fn _get_end_klu(&self) -> Handle<CKLineUnit>;
    fn _dir(&self) -> BiDir;
    //fn get_pre(&self) -> Option<Handle<Self>>;
    //fn get_next(&self) -> Option<Handle<Self>>;

    fn _get_parent_seg(&self) -> Option<Handle<Self::Parent>>;
    fn _set_parent_seg(&mut self, parent_seg: Option<Handle<Self::Parent>>);

    fn _seg_idx(&self) -> Option<usize>;
    fn _set_seg_idx(&mut self, idx: usize);
    // 修改属性
    fn _set_pre(&mut self, pre: Option<Handle<Self>>);
    fn _set_next(&mut self, next: Option<Handle<Self>>);

    fn _get_begin_klc(&self) -> Handle<CKLine>;
    fn _get_end_klc(&self) -> Handle<CKLine>;
    // 默认方法实现
    fn _is_up(&self) -> bool {
        self._dir() == BiDir::Up
    }

    fn _is_down(&self) -> bool {
        self._dir() == BiDir::Down
    }

    fn _is_sure(&self) -> bool;
    fn _next(&self) -> Option<Handle<Self>>;
    fn _pre(&self) -> Option<Handle<Self>>;

    fn _cal_macd_metric(
        &self,
        macd_algo: MacdAlgo,
        is_reverse: bool,
    ) -> Result<f64, CChanException>;

    fn _set_bsp(&mut self, bsp: Option<Handle<CBSPoint<Self>>>)
    where
        Self: Sized;

    fn _amp(&self) -> Option<f64>;
}

// 更新 CBi 的实现
impl Line for CBi {
    type Parent = CSeg<CBi>;

    fn _idx(&self) -> usize {
        self.idx
    }

    fn _high(&self) -> f64 {
        self.high()
    }

    fn _low(&self) -> f64 {
        self.low()
    }

    fn _get_begin_val(&self) -> f64 {
        self.get_begin_val()
    }

    fn _get_end_val(&self) -> f64 {
        self.get_end_val()
    }

    fn _dir(&self) -> BiDir {
        self.dir
    }

    //fn get_pre(&self) -> Option<Handle<Self>> {
    //    self.pre.clone()
    //}
    //
    //fn get_next(&self) -> Option<Handle<Self>> {
    //    self.next.clone()
    //}

    fn _set_pre(&mut self, pre: Option<Handle<Self>>) {
        self.pre = pre;
    }

    fn _set_next(&mut self, next: Option<Handle<Self>>) {
        self.next = next;
    }

    fn _get_begin_klu(&self) -> Handle<CKLineUnit> {
        self.get_begin_klu()
    }

    fn _get_end_klu(&self) -> Handle<CKLineUnit> {
        self.get_end_klu()
    }

    fn _set_parent_seg(&mut self, parent_seg: Option<Handle<Self::Parent>>) {
        self.parent_seg = parent_seg;
    }

    fn _get_begin_klc(&self) -> Handle<CKLine> {
        Rc::clone(&self.begin_klc)
    }

    fn _get_end_klc(&self) -> Handle<CKLine> {
        Rc::clone(&self.end_klc)
    }

    fn _is_sure(&self) -> bool {
        self.is_sure
    }

    fn _next(&self) -> Option<Handle<Self>> {
        self.next.as_ref().map(|x| Rc::clone(x))
    }

    fn _pre(&self) -> Option<Handle<Self>> {
        self.pre.as_ref().map(|x| Rc::clone(x))
    }

    fn _get_parent_seg(&self) -> Option<Handle<Self::Parent>> {
        self.parent_seg.as_ref().map(|x| Rc::clone(x))
    }

    fn _is_up(&self) -> bool {
        self.dir() == BiDir::Up
    }

    fn _is_down(&self) -> bool {
        self.dir() == BiDir::Down
    }

    fn _seg_idx(&self) -> Option<usize> {
        self.seg_idx
    }

    fn _cal_macd_metric(
        &self,
        macd_algo: MacdAlgo,
        is_reverse: bool,
    ) -> Result<f64, CChanException> {
        self.cal_macd_metric(macd_algo, is_reverse)
    }

    fn _set_bsp(&mut self, bsp: Option<Handle<CBSPoint<Self>>>)
    where
        Self: Sized,
    {
        self.bsp = bsp.map(|b| Rc::clone(&b));
    }

    fn _amp(&self) -> Option<f64> {
        None
    }

    fn _set_seg_idx(&mut self, idx: usize) {
        self.seg_idx = Some(idx);
    }
}

impl SegLine for CSeg<CBi> {
    fn __get_bi_list_len(&self) -> usize {
        self.bi_list.len()
    }

    fn __idx(&self) -> usize {
        self.idx
    }

    fn __dir(&self) -> BiDir {
        self.dir
    }
}
// 更新 CSeg 的实现
impl Line for CSeg<CBi> {
    type Parent = CSeg<CSeg<CBi>>;

    fn _idx(&self) -> usize {
        self.idx
    }

    fn _high(&self) -> f64 {
        self.high()
    }

    fn _low(&self) -> f64 {
        self.low()
    }

    fn _get_begin_val(&self) -> f64 {
        self.get_begin_val()
    }

    fn _get_end_val(&self) -> f64 {
        self.get_end_val()
    }

    fn _dir(&self) -> BiDir {
        self.dir
    }

    //fn get_pre(&self) -> Option<Handle<Self>> {
    //    self.pre.clone()
    //}
    //
    //fn get_next(&self) -> Option<Handle<Self>> {
    //    self.next.clone()
    //}

    fn _set_pre(&mut self, pre: Option<Handle<Self>>) {
        self.pre = pre;
    }

    fn _set_next(&mut self, next: Option<Handle<Self>>) {
        self.next = next;
    }

    fn _get_begin_klu(&self) -> Handle<CKLineUnit> {
        self.get_begin_klu()
    }

    fn _get_end_klu(&self) -> Handle<CKLineUnit> {
        self.get_end_klu()
    }

    fn _get_parent_seg(&self) -> Option<Handle<Self::Parent>> {
        self.parent_seg.as_ref().and_then(|weak| weak.upgrade())
    }
    fn _set_parent_seg(&mut self, parent_seg: Option<Handle<Self::Parent>>) {
        self.parent_seg = parent_seg.map(|rc| Rc::downgrade(&rc));
    }

    fn _get_begin_klc(&self) -> Handle<CKLine> {
        Rc::clone(&self.start_bi.borrow().begin_klc)
    }

    fn _get_end_klc(&self) -> Handle<CKLine> {
        Rc::clone(&self.end_bi.borrow().end_klc)
    }

    fn _is_sure(&self) -> bool {
        self.is_sure
    }

    fn _next(&self) -> Option<Handle<Self>> {
        self.next.as_ref().map(|x| Rc::clone(x))
    }

    fn _pre(&self) -> Option<Handle<Self>> {
        self.pre.as_ref().map(|x| Rc::clone(x))
    }

    fn _seg_idx(&self) -> Option<usize> {
        self.seg_idx
    }

    fn _cal_macd_metric(
        &self,
        macd_algo: MacdAlgo,
        is_reverse: bool,
    ) -> Result<f64, CChanException> {
        self.cal_macd_metric(macd_algo, is_reverse)
    }

    fn _set_bsp(&mut self, bsp: Option<Handle<CBSPoint<Self>>>)
    where
        Self: Sized,
    {
        self.bsp = bsp.map(|b| Rc::clone(&b));
    }

    fn _amp(&self) -> Option<f64> {
        None
    }

    fn _set_seg_idx(&mut self, idx: usize) {
        self.seg_idx = Some(idx);
    }
}

impl SegLine for CSeg<CSeg<CBi>> {
    fn __get_bi_list_len(&self) -> usize {
        self.bi_list.len()
    }

    fn __idx(&self) -> usize {
        self.idx
    }

    fn __dir(&self) -> BiDir {
        self.dir
    }
}
