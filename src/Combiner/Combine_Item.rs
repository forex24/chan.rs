use crate::bi::bi::CBi;
use crate::common::chan_exception::{CChanException, ErrCode};
use crate::kline::kline_unit::CKLineUnit;
use crate::seg::seg::CSeg;
use std::cell::RefCell;
use std::rc::Rc;

pub enum CombineItemType {
    Bi(Rc<RefCell<CBi>>),
    KLineUnit(Rc<RefCell<CKLineUnit>>),
    Seg(Rc<RefCell<CSeg>>),
}

pub struct CCombineItem {
    pub time_begin: i64,
    pub time_end: i64,
    pub high: f64,
    pub low: f64,
}

impl CCombineItem {
    pub fn new(item: CombineItemType) -> Result<Self, CChanException> {
        match item {
            CombineItemType::Bi(bi) => {
                let bi = bi.borrow();
                Ok(CCombineItem {
                    time_begin: bi.begin_klc.borrow().idx,
                    time_end: bi.end_klc.borrow().idx,
                    high: bi._high(),
                    low: bi._low(),
                })
            }
            CombineItemType::KLineUnit(kline_unit) => {
                let kline_unit = kline_unit.borrow();
                Ok(CCombineItem {
                    time_begin: kline_unit.time,
                    time_end: kline_unit.time,
                    high: kline_unit.high,
                    low: kline_unit.low,
                })
            }
            CombineItemType::Seg(seg) => {
                let seg = seg.borrow();
                Ok(CCombineItem {
                    time_begin: seg.start_bi.borrow().begin_klc.borrow().idx,
                    time_end: seg.end_bi.borrow().end_klc.borrow().idx,
                    high: seg._high(),
                    low: seg._low(),
                })
            }
        }
    }
}
