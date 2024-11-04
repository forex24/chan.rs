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
    fn seg_line_idx(&self) -> usize;
    fn seg_line_dir(&self) -> BiDir;
    fn seg_line_get_bi_list_len(&self) -> usize;
}

pub trait Line: Sized {
    type Parent: SegLine;
    // 读取属性
    fn line_idx(&self) -> usize;
    fn line_high(&self) -> f64;
    fn line_low(&self) -> f64;
    fn line_get_begin_val(&self) -> f64;
    fn line_get_end_val(&self) -> f64;
    fn line_get_begin_klu(&self) -> Handle<CKLineUnit>;
    fn line_get_end_klu(&self) -> Handle<CKLineUnit>;
    fn line_dir(&self) -> BiDir;
    //fn get_pre(&self) -> Option<Handle<Self>>;
    //fn get_next(&self) -> Option<Handle<Self>>;

    fn line_get_parent_seg(&self) -> Option<Handle<CSeg<Self>>>;
    fn line_set_parent_seg(&mut self, parent_seg: Option<Handle<CSeg<Self>>>);

    fn line_seg_idx(&self) -> Option<usize>;
    fn line_set_seg_idx(&mut self, idx: usize);
    // 修改属性
    fn line_set_pre(&mut self, pre: Option<Handle<Self>>);
    fn line_set_next(&mut self, next: Option<Handle<Self>>);

    fn line_get_begin_klc(&self) -> Handle<CKLine>;
    fn line_get_end_klc(&self) -> Handle<CKLine>;
    // 默认方法实现
    fn line_is_up(&self) -> bool {
        self.line_dir() == BiDir::Up
    }

    fn line_is_down(&self) -> bool {
        self.line_dir() == BiDir::Down
    }

    fn line_is_sure(&self) -> bool;
    fn line_next(&self) -> Option<Handle<Self>>;
    fn line_pre(&self) -> Option<Handle<Self>>;

    fn line_cal_macd_metric(
        &self,
        macd_algo: MacdAlgo,
        is_reverse: bool,
    ) -> Result<f64, CChanException>;

    fn line_set_bsp(&mut self, bsp: Option<Handle<CBSPoint<Self>>>)
    where
        Self: Sized;

    fn line_amp(&self) -> Option<f64>;
}

// 更新 CBi 的实现
impl Line for CBi {
    type Parent = CSeg<CBi>;

    fn line_idx(&self) -> usize {
        self.idx
    }

    fn line_high(&self) -> f64 {
        self.high()
    }

    fn line_low(&self) -> f64 {
        self.low()
    }

    fn line_get_begin_val(&self) -> f64 {
        self.get_begin_val()
    }

    fn line_get_end_val(&self) -> f64 {
        self.get_end_val()
    }

    fn line_dir(&self) -> BiDir {
        self.dir
    }

    //fn get_pre(&self) -> Option<Handle<Self>> {
    //    self.pre.clone()
    //}
    //
    //fn get_next(&self) -> Option<Handle<Self>> {
    //    self.next.clone()
    //}

    fn line_set_pre(&mut self, pre: Option<Handle<Self>>) {
        self.pre = pre;
    }

    fn line_set_next(&mut self, next: Option<Handle<Self>>) {
        self.next = next;
    }

    fn line_get_begin_klu(&self) -> Handle<CKLineUnit> {
        self.get_begin_klu()
    }

    fn line_get_end_klu(&self) -> Handle<CKLineUnit> {
        self.get_end_klu()
    }

    fn line_set_parent_seg(&mut self, parent_seg: Option<Handle<Self::Parent>>) {
        self.parent_seg = parent_seg;
    }

    fn line_get_begin_klc(&self) -> Handle<CKLine> {
        Rc::clone(&self.begin_klc)
    }

    fn line_get_end_klc(&self) -> Handle<CKLine> {
        Rc::clone(&self.end_klc)
    }

    fn line_is_sure(&self) -> bool {
        self.is_sure
    }

    fn line_next(&self) -> Option<Handle<Self>> {
        self.next.as_ref().map(|x| Rc::clone(x))
    }

    fn line_pre(&self) -> Option<Handle<Self>> {
        self.pre.as_ref().map(|x| Rc::clone(x))
    }

    fn line_get_parent_seg(&self) -> Option<Handle<Self::Parent>> {
        self.parent_seg.as_ref().map(|x| Rc::clone(x))
    }

    fn line_is_up(&self) -> bool {
        self.dir() == BiDir::Up
    }

    fn line_is_down(&self) -> bool {
        self.dir() == BiDir::Down
    }

    fn line_seg_idx(&self) -> Option<usize> {
        self.seg_idx
    }

    fn line_cal_macd_metric(
        &self,
        macd_algo: MacdAlgo,
        is_reverse: bool,
    ) -> Result<f64, CChanException> {
        self.cal_macd_metric(macd_algo, is_reverse)
    }

    fn line_set_bsp(&mut self, bsp: Option<Handle<CBSPoint<Self>>>)
    where
        Self: Sized,
    {
        self.bsp = bsp.map(|b| Rc::clone(&b));
    }

    fn line_amp(&self) -> Option<f64> {
        None
    }

    fn line_set_seg_idx(&mut self, idx: usize) {
        self.seg_idx = Some(idx);
    }
}

impl SegLine for CSeg<CBi> {
    fn seg_line_get_bi_list_len(&self) -> usize {
        self.bi_list.len()
    }

    fn seg_line_idx(&self) -> usize {
        self.idx
    }

    fn seg_line_dir(&self) -> BiDir {
        self.dir
    }
}
// 更新 CSeg 的实现
impl Line for CSeg<CBi> {
    type Parent = CSeg<CSeg<CBi>>;

    fn line_idx(&self) -> usize {
        self.idx
    }

    fn line_high(&self) -> f64 {
        self.high()
    }

    fn line_low(&self) -> f64 {
        self.low()
    }

    fn line_get_begin_val(&self) -> f64 {
        self.get_begin_val()
    }

    fn line_get_end_val(&self) -> f64 {
        self.get_end_val()
    }

    fn line_dir(&self) -> BiDir {
        self.dir
    }

    //fn get_pre(&self) -> Option<Handle<Self>> {
    //    self.pre.clone()
    //}
    //
    //fn get_next(&self) -> Option<Handle<Self>> {
    //    self.next.clone()
    //}

    fn line_set_pre(&mut self, pre: Option<Handle<Self>>) {
        self.pre = pre;
    }

    fn line_set_next(&mut self, next: Option<Handle<Self>>) {
        self.next = next;
    }

    fn line_get_begin_klu(&self) -> Handle<CKLineUnit> {
        self.get_begin_klu()
    }

    fn line_get_end_klu(&self) -> Handle<CKLineUnit> {
        self.get_end_klu()
    }

    fn line_get_parent_seg(&self) -> Option<Handle<Self::Parent>> {
        self.parent_seg.as_ref().and_then(|weak| weak.upgrade())
    }
    fn line_set_parent_seg(&mut self, parent_seg: Option<Handle<Self::Parent>>) {
        self.parent_seg = parent_seg.map(|rc| Rc::downgrade(&rc));
    }

    fn line_get_begin_klc(&self) -> Handle<CKLine> {
        Rc::clone(&self.start_bi.borrow().begin_klc)
    }

    fn line_get_end_klc(&self) -> Handle<CKLine> {
        Rc::clone(&self.end_bi.borrow().end_klc)
    }

    fn line_is_sure(&self) -> bool {
        self.is_sure
    }

    fn line_next(&self) -> Option<Handle<Self>> {
        self.next.as_ref().map(|x| Rc::clone(x))
    }

    fn line_pre(&self) -> Option<Handle<Self>> {
        self.pre.as_ref().map(|x| Rc::clone(x))
    }

    fn line_seg_idx(&self) -> Option<usize> {
        self.seg_idx
    }

    fn line_cal_macd_metric(
        &self,
        macd_algo: MacdAlgo,
        is_reverse: bool,
    ) -> Result<f64, CChanException> {
        self.cal_macd_metric(macd_algo, is_reverse)
    }

    fn line_set_bsp(&mut self, bsp: Option<Handle<CBSPoint<Self>>>)
    where
        Self: Sized,
    {
        self.bsp = bsp.map(|b| Rc::clone(&b));
    }

    fn line_amp(&self) -> Option<f64> {
        None
    }

    fn line_set_seg_idx(&mut self, idx: usize) {
        self.seg_idx = Some(idx);
    }
}

impl SegLine for CSeg<CSeg<CBi>> {
    fn seg_line_get_bi_list_len(&self) -> usize {
        self.bi_list.len()
    }

    fn seg_line_idx(&self) -> usize {
        self.idx
    }

    fn seg_line_dir(&self) -> BiDir {
        self.dir
    }
}
