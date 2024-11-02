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
        self.idx as usize
    }

    fn _high(&self) -> f64 {
        self._high()
    }

    fn _low(&self) -> f64 {
        self._low()
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
        self._high()
    }

    fn _low(&self) -> f64 {
        self._low()
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
// Add this implementation
impl Line for CSeg<CSeg<CBi>> {
    type Parent = CSeg<CSeg<CSeg<CBi>>>;

    fn _idx(&self) -> usize {
        self.idx
    }

    fn _high(&self) -> f64 {
        self._high()
    }

    fn _low(&self) -> f64 {
        self._low()
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
        Rc::clone(&self.start_bi.borrow().start_bi.borrow().begin_klc)
    }

    fn _get_end_klc(&self) -> Handle<CKLine> {
        Rc::clone(&self.end_bi.borrow().end_bi.borrow().end_klc)
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

impl SegLine for CSeg<CSeg<CSeg<CBi>>> {
    fn __get_bi_list_len(&self) -> usize {
        unimplemented!()
    }

    fn __idx(&self) -> usize {
        unimplemented!()
    }

    fn __dir(&self) -> BiDir {
        unimplemented!()
    }
}

/*impl Line for CSeg<CSeg<CSeg<CBi>>> {
    type Parent = CBi;

    fn idx(&self) -> usize {
        unimplemented!()
    }

    fn high(&self) -> f64 {
        unimplemented!()
    }

    fn low(&self) -> f64 {
        unimplemented!()
    }

    fn get_begin_val(&self) -> f64 {
        unimplemented!()
    }

    fn get_end_val(&self) -> f64 {
        unimplemented!()
    }

    fn dir(&self) -> BiDir {
        unimplemented!()
    }

    //fn get_pre(&self) -> Option<Handle<Self>> {
    //    self.pre.clone()
    //}
    //
    //fn get_next(&self) -> Option<Handle<Self>> {
    //    self.next.clone()
    //}

    fn set_pre(&mut self, _pre: Option<Handle<Self>>) {
        unimplemented!()
    }

    fn set_next(&mut self, _next: Option<Handle<Self>>) {
        unimplemented!()
    }

    fn get_begin_klu(&self) -> Handle<CKLineUnit> {
        unimplemented!()
    }

    fn get_end_klu(&self) -> Handle<CKLineUnit> {
        unimplemented!()
    }

    fn get_parent_seg(&self) -> Option<Handle<Self::Parent>> {
        unimplemented!()
    }
    fn set_parent_seg(&mut self, _parent_seg: Option<Handle<Self::Parent>>) {
        unimplemented!()
    }

    fn get_begin_klc(&self) -> Handle<CKLine> {
        unimplemented!()
    }

    fn get_end_klc(&self) -> Handle<CKLine> {
        unimplemented!()
    }

    fn is_sure(&self) -> bool {
        unimplemented!()
    }

    fn next(&self) -> Option<Handle<Self>> {
        unimplemented!()
    }

    fn pre(&self) -> Option<Handle<Self>> {
        unimplemented!()
    }

    fn seg_idx(&self) -> Option<usize> {
        unimplemented!()
    }

    fn cal_macd_metric(
        &self,
        _macd_algo: MacdAlgo,
        _is_reverse: bool,
    ) -> Result<f64, CChanException> {
        unimplemented!()
    }

    fn set_bsp(&mut self, _bsp: Option<Handle<CBSPoint<Self>>>)
    where
        Self: Sized,
    {
        unimplemented!()
    }

    fn amp(&self) -> Option<f64> {
        unimplemented!()
    }

    fn set_seg_idx(&mut self, idx: usize) {
        unimplemented!()
    }
}
*/
/*
pub enum BiType {
    CBi(CBi),
    CSeg(CSeg<CBi>), // 允许包含 CSeg<CBi>
}

pub enum SegType {
    CSeg(CSeg<CBi>),          // 当 BiType 为 CBi 时
    CSegCSeg(CSeg<CSeg<CBi>>), // 当 BiType 为 CSeg<CBi> 时
}

impl BiType {
    pub fn idx(&self) -> usize {
        match self {
            BiType::CBi(bi) => bi.idx(),
            BiType::CSeg(seg) => seg.idx(),
        }
    }

    pub fn high(&self) -> f64 {
        match self {
            BiType::CBi(bi) => bi.high(),
            BiType::CSeg(seg) => seg.high(),
        }
    }

    // 其他方法...
}

impl SegType {
    pub fn idx(&self) -> usize {
        match self {
            SegType::CSeg(seg) => seg.idx(),
            SegType::CSegCSeg(seg) => seg.idx(),
        }
    }

    pub fn high(&self) -> f64 {
        match self {
            SegType::CSeg(seg) => seg.high(),
            SegType::CSegCSeg(seg) => seg.high(),
        }
    }

    // 其他方法...
}

impl Line for BiType {
    type Parent = CSeg<CBi>; // 假设 BiType 的 Parent 是 CSeg<CBi>

    fn idx(&self) -> usize {
        match self {
            BiType::CBi(bi) => bi.idx(),
            BiType::CSeg(seg) => seg.idx(),
        }
    }

    fn high(&self) -> f64 {
        match self {
            BiType::CBi(bi) => bi.high(),
            BiType::CSeg(seg) => seg.high(),
        }
    }

    fn low(&self) -> f64 {
        match self {
            BiType::CBi(bi) => bi.low(),
            BiType::CSeg(seg) => seg.low(),
        }
    }

    fn get_begin_val(&self) -> f64 {
        match self {
            BiType::CBi(bi) => bi.get_begin_val(),
            BiType::CSeg(seg) => seg.get_begin_val(),
        }
    }

    fn get_end_val(&self) -> f64 {
        match self {
            BiType::CBi(bi) => bi.get_end_val(),
            BiType::CSeg(seg) => seg.get_end_val(),
        }
    }

    fn get_begin_klu(&self) -> Handle<CKLineUnit> {
        match self {
            BiType::CBi(bi) => bi.get_begin_klu(),
            BiType::CSeg(seg) => seg.get_begin_klu(),
        }
    }

    fn get_end_klu(&self) -> Handle<CKLineUnit> {
        match self {
            BiType::CBi(bi) => bi.get_end_klu(),
            BiType::CSeg(seg) => seg.get_end_klu(),
        }
    }

    fn dir(&self) -> BiDir {
        match self {
            BiType::CBi(bi) => bi.dir(),
            BiType::CSeg(seg) => seg.dir(),
        }
    }

    fn get_parent_seg(&self) -> Option<Handle<Self::Parent>> {
        match self {
            BiType::CBi(bi) => bi.get_parent_seg(),
            BiType::CSeg(seg) => seg.get_parent_seg(),
        }
    }

    fn set_parent_seg(&mut self, parent_seg: Option<Handle<Self::Parent>>) {
        match self {
            BiType::CBi(bi) => bi.set_parent_seg(parent_seg),
            BiType::CSeg(seg) => seg.set_parent_seg(parent_seg),
        }
    }

    fn seg_idx(&self) -> Option<usize> {
        match self {
            BiType::CBi(bi) => bi.seg_idx(),
            BiType::CSeg(seg) => seg.seg_idx(),
        }
    }

    fn set_seg_idx(&mut self, idx: usize) {
        match self {
            BiType::CBi(bi) => bi.set_seg_idx(idx),
            BiType::CSeg(seg) => seg.set_seg_idx(idx),
        }
    }

    fn set_pre(&mut self, pre: Option<Handle<Self>>) {
        match self {
            BiType::CBi(bi) => bi.set_pre(pre),
            BiType::CSeg(seg) => seg.set_pre(pre),
        }
    }

    fn set_next(&mut self, next: Option<Handle<Self>>) {
        match self {
            BiType::CBi(bi) => bi.set_next(next),
            BiType::CSeg(seg) => seg.set_next(next),
        }
    }

    fn get_begin_klc(&self) -> Handle<CKLine> {
        match self {
            BiType::CBi(bi) => bi.get_begin_klc(),
            BiType::CSeg(seg) => seg.get_begin_klc(),
        }
    }

    fn get_end_klc(&self) -> Handle<CKLine> {
        match self {
            BiType::CBi(bi) => bi.get_end_klc(),
            BiType::CSeg(seg) => seg.get_end_klc(),
        }
    }

    fn is_sure(&self) -> bool {
        match self {
            BiType::CBi(bi) => bi.is_sure(),
            BiType::CSeg(seg) => seg.is_sure(),
        }
    }

    fn next(&self) -> Option<Handle<Self>> {
        match self {
            BiType::CBi(bi) => bi.next(),
            BiType::CSeg(seg) => seg.next(),
        }
    }

    fn pre(&self) -> Option<Handle<Self>> {
        match self {
            BiType::CBi(bi) => bi.pre(),
            BiType::CSeg(seg) => seg.pre(),
        }
    }

    fn cal_macd_metric(&self, macd_algo: MacdAlgo, is_reverse: bool) -> Result<f64, CChanException> {
        match self {
            BiType::CBi(bi) => bi.cal_macd_metric(macd_algo, is_reverse),
            BiType::CSeg(seg) => seg.cal_macd_metric(macd_algo, is_reverse),
        }
    }

    fn set_bsp(&mut self, bsp: Option<Handle<CBSPoint<Self>>>)
    where
        Self: Sized,
    {
        match self {
            BiType::CBi(bi) => bi.set_bsp(bsp),
            BiType::CSeg(seg) => seg.set_bsp(bsp),
        }
    }

    fn amp(&self) -> Option<f64> {
        match self {
            BiType::CBi(bi) => bi.amp(),
            BiType::CSeg(seg) => seg.amp(),
        }
    }
}

impl Line for SegType {
    type Parent = CSeg<CBi>; // 假设 SegType 的 Parent 是 CSeg<CBi>

    fn idx(&self) -> usize {
        match self {
            SegType::CSeg(seg) => seg.idx(),
            SegType::CSegCSeg(seg) => seg.idx(),
        }
    }

    fn high(&self) -> f64 {
        match self {
            SegType::CSeg(seg) => seg.high(),
            SegType::CSegCSeg(seg) => seg.high(),
        }
    }

    fn low(&self) -> f64 {
        match self {
            SegType::CSeg(seg) => seg.low(),
            SegType::CSegCSeg(seg) => seg.low(),
        }
    }

    fn get_begin_val(&self) -> f64 {
        match self {
            SegType::CSeg(seg) => seg.get_begin_val(),
            SegType::CSegCSeg(seg) => seg.get_begin_val(),
        }
    }

    fn get_end_val(&self) -> f64 {
        match self {
            SegType::CSeg(seg) => seg.get_end_val(),
            SegType::CSegCSeg(seg) => seg.get_end_val(),
        }
    }

    fn get_begin_klu(&self) -> Handle<CKLineUnit> {
        match self {
            SegType::CSeg(seg) => seg.get_begin_klu(),
            SegType::CSegCSeg(seg) => seg.get_begin_klu(),
        }
    }

    fn get_end_klu(&self) -> Handle<CKLineUnit> {
        match self {
            SegType::CSeg(seg) => seg.get_end_klu(),
            SegType::CSegCSeg(seg) => seg.get_end_klu(),
        }
    }

    fn dir(&self) -> BiDir {
        match self {
            SegType::CSeg(seg) => seg.dir(),
            SegType::CSegCSeg(seg) => seg.dir(),
        }
    }

    fn get_parent_seg(&self) -> Option<Handle<Self::Parent>> {
        match self {
            SegType::CSeg(seg) => seg.get_parent_seg(),
            SegType::CSegCSeg(seg) => seg.get_parent_seg(),
        }
    }

    fn set_parent_seg(&mut self, parent_seg: Option<Handle<Self::Parent>>) {
        match self {
            SegType::CSeg(seg) => seg.set_parent_seg(parent_seg),
            SegType::CSegCSeg(seg) => seg.set_parent_seg(parent_seg),
        }
    }

    fn seg_idx(&self) -> Option<usize> {
        match self {
            SegType::CSeg(seg) => seg.seg_idx(),
            SegType::CSegCSeg(seg) => seg.seg_idx(),
        }
    }

    fn set_seg_idx(&mut self, idx: usize) {
        match self {
            SegType::CSeg(seg) => seg.set_seg_idx(idx),
            SegType::CSegCSeg(seg) => seg.set_seg_idx(idx),
        }
    }

    fn set_pre(&mut self, pre: Option<Handle<Self>>) {
        match self {
            SegType::CSeg(seg) => seg.set_pre(pre),
            SegType::CSegCSeg(seg) => seg.set_pre(pre),
        }
    }

    fn set_next(&mut self, next: Option<Handle<Self>>) {
        match self {
            SegType::CSeg(seg) => seg.set_next(next),
            SegType::CSegCSeg(seg) => seg.set_next(next),
        }
    }

    fn get_begin_klc(&self) -> Handle<CKLine> {
        match self {
            SegType::CSeg(seg) => seg.get_begin_klc(),
            SegType::CSegCSeg(seg) => seg.get_begin_klc(),
        }
    }

    fn get_end_klc(&self) -> Handle<CKLine> {
        match self {
            SegType::CSeg(seg) => seg.get_end_klc(),
            SegType::CSegCSeg(seg) => seg.get_end_klc(),
        }
    }

    fn is_sure(&self) -> bool {
        match self {
            SegType::CSeg(seg) => seg.is_sure(),
            SegType::CSegCSeg(seg) => seg.is_sure(),
        }
    }

    fn next(&self) -> Option<Handle<Self>> {
        match self {
            SegType::CSeg(seg) => seg.next(),
            SegType::CSegCSeg(seg) => seg.next(),
        }
    }

    fn pre(&self) -> Option<Handle<Self>> {
        match self {
            SegType::CSeg(seg) => seg.pre(),
            SegType::CSegCSeg(seg) => seg.pre(),
        }
    }

    fn cal_macd_metric(&self, macd_algo: MacdAlgo, is_reverse: bool) -> Result<f64, CChanException> {
        match self {
            SegType::CSeg(seg) => seg.cal_macd_metric(macd_algo, is_reverse),
            SegType::CSegCSeg(seg) => seg.cal_macd_metric(macd_algo, is_reverse),
        }
    }

    fn set_bsp(&mut self, bsp: Option<Handle<CBSPoint<Self>>>)
    where
        Self: Sized,
    {
        match self {
            SegType::CSeg(seg) => seg.set_bsp(bsp),
            SegType::CSegCSeg(seg) => seg.set_bsp(bsp),
        }
    }

    fn amp(&self) -> Option<f64> {
        match self {
            SegType::CSeg(seg) => seg.amp(),
            SegType::CSegCSeg(seg) => seg.amp(),
        }
    }
}
*/
