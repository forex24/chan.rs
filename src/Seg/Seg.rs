use crate::BuySellPoint::BS_Point::CBSPoint;
use crate::Common::types::SharedCell;
use crate::Common::CEnum::{BiDir, MacdAlgo, TrendLineSide};
use crate::Common::ChanException::{CChanException, ErrCode};
use crate::KLine::KLine_Unit::CKLineUnit;
use crate::Math::TrendLine::CTrendLine;
use crate::Seg::EigenFX::CEigenFX;
use crate::ZS::ZS::CZS;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

pub struct CSeg<LINE_TYPE> {
    pub idx: i32,
    pub start_bi: SharedCell<LINE_TYPE>,
    pub end_bi: SharedCell<LINE_TYPE>,
    pub is_sure: bool,
    pub dir: BiDir,
    pub zs_lst: Vec<SharedCell<CZS>>,
    pub eigen_fx: Option<SharedCell<CEigenFX>>,
    pub seg_idx: Option<i32>,
    pub parent_seg: Option<SharedCell<CSeg<LINE_TYPE>>>,
    pub pre: Option<SharedCell<CSeg<LINE_TYPE>>>,
    pub next: Option<SharedCell<CSeg<LINE_TYPE>>>,
    pub bsp: Option<SharedCell<CBSPoint>>,
    pub bi_list: Vec<SharedCell<LINE_TYPE>>,
    pub reason: String,
    pub support_trend_line: Option<CTrendLine>,
    pub resistance_trend_line: Option<CTrendLine>,
    pub ele_inside_is_sure: bool,
    _phantom: PhantomData<LINE_TYPE>,
}

impl<LINE_TYPE> CSeg<LINE_TYPE> {
    pub fn new(
        idx: i32,
        start_bi: SharedCell<LINE_TYPE>,
        end_bi: SharedCell<LINE_TYPE>,
        is_sure: bool,
        seg_dir: Option<BiDir>,
        reason: &str,
    ) -> Result<Self, CChanException> {
        let dir = seg_dir.unwrap_or_else(|| end_bi.borrow().dir);
        let mut seg = CSeg {
            idx,
            start_bi: start_bi.clone(),
            end_bi: end_bi.clone(),
            is_sure,
            dir,
            zs_lst: Vec::new(),
            eigen_fx: None,
            seg_idx: None,
            parent_seg: None,
            pre: None,
            next: None,
            bsp: None,
            bi_list: Vec::new(),
            reason: reason.to_string(),
            support_trend_line: None,
            resistance_trend_line: None,
            ele_inside_is_sure: false,
            _phantom: PhantomData,
        };

        if end_bi.borrow().idx - start_bi.borrow().idx < 2 {
            seg.is_sure = false;
        }
        seg.check()?;
        Ok(seg)
    }

    pub fn set_seg_idx(&mut self, idx: i32) {
        self.seg_idx = Some(idx);
    }

    pub fn check(&self) -> Result<(), CChanException> {
        if !self.is_sure {
            return Ok(());
        }
        if self.is_down() {
            if self.start_bi.borrow().get_begin_val() < self.end_bi.borrow().get_end_val() {
                return Err(CChanException::new(
                    format!("下降线段起始点应该高于结束点! idx={}", self.idx),
                    ErrCode::SegEndValueErr,
                ));
            }
        } else if self.start_bi.borrow().get_begin_val() > self.end_bi.borrow().get_end_val() {
            return Err(CChanException::new(
                format!("上升线段起始点应该低于结束点! idx={}", self.idx),
                ErrCode::SegEndValueErr,
            ));
        }
        if self.end_bi.borrow().idx - self.start_bi.borrow().idx < 2 {
            return Err(CChanException::new(
                format!(
                    "线段({}-{})长度不能小于2! idx={}",
                    self.start_bi.borrow().idx,
                    self.end_bi.borrow().idx,
                    self.idx
                ),
                ErrCode::SegLenErr,
            ));
        }
        Ok(())
    }

    pub fn add_zs(&mut self, zs: SharedCell<CZS>) {
        self.zs_lst.insert(0, zs);
    }

    pub fn cal_klu_slope(&self) -> f64 {
        (self.get_end_val() - self.get_begin_val())
            / (self.get_end_klu().borrow().idx - self.get_begin_klu().borrow().idx) as f64
            / self.get_begin_val()
    }

    pub fn cal_amp(&self) -> f64 {
        (self.get_end_val() - self.get_begin_val()) / self.get_begin_val()
    }

    pub fn cal_bi_cnt(&self) -> i32 {
        self.end_bi.borrow().idx - self.start_bi.borrow().idx + 1
    }

    pub fn clear_zs_lst(&mut self) {
        self.zs_lst.clear();
    }

    pub fn _low(&self) -> f64 {
        if self.is_down() {
            self.end_bi.borrow().get_end_klu().borrow().low
        } else {
            self.start_bi.borrow().get_begin_klu().borrow().low
        }
    }

    pub fn _high(&self) -> f64 {
        if self.is_up() {
            self.end_bi.borrow().get_end_klu().borrow().high
        } else {
            self.start_bi.borrow().get_begin_klu().borrow().high
        }
    }

    pub fn is_down(&self) -> bool {
        self.dir == BiDir::Down
    }

    pub fn is_up(&self) -> bool {
        self.dir == BiDir::Up
    }

    pub fn get_end_val(&self) -> f64 {
        self.end_bi.borrow().get_end_val()
    }

    pub fn get_begin_val(&self) -> f64 {
        self.start_bi.borrow().get_begin_val()
    }

    pub fn amp(&self) -> f64 {
        (self.get_end_val() - self.get_begin_val()).abs()
    }

    pub fn get_end_klu(&self) -> SharedCell<CKLineUnit> {
        self.end_bi.borrow().get_end_klu()
    }

    pub fn get_begin_klu(&self) -> SharedCell<CKLineUnit> {
        self.start_bi.borrow().get_begin_klu()
    }

    pub fn get_klu_cnt(&self) -> i32 {
        self.get_end_klu().borrow().idx - self.get_begin_klu().borrow().idx + 1
    }

    pub fn cal_macd_metric(
        &self,
        macd_algo: MacdAlgo,
        is_reverse: bool,
    ) -> Result<f64, CChanException> {
        match macd_algo {
            MacdAlgo::Slope => Ok(self.cal_macd_slope()),
            MacdAlgo::Amp => Ok(self.cal_macd_amp()),
            _ => Err(CChanException::new(
                format!(
                    "unsupport macd_algo={:?} of Seg, should be one of slope/amp",
                    macd_algo
                ),
                ErrCode::ParaError,
            )),
        }
    }

    pub fn cal_macd_slope(&self) -> f64 {
        let begin_klu = self.get_begin_klu();
        let end_klu = self.get_end_klu();
        if self.is_up() {
            (end_klu.borrow().high - begin_klu.borrow().low)
                / end_klu.borrow().high
                / (end_klu.borrow().idx - begin_klu.borrow().idx + 1) as f64
        } else {
            (begin_klu.borrow().high - end_klu.borrow().low)
                / begin_klu.borrow().high
                / (end_klu.borrow().idx - begin_klu.borrow().idx + 1) as f64
        }
    }

    pub fn cal_macd_amp(&self) -> f64 {
        let begin_klu = self.get_begin_klu();
        let end_klu = self.get_end_klu();
        if self.is_down() {
            (begin_klu.borrow().high - end_klu.borrow().low) / begin_klu.borrow().high
        } else {
            (end_klu.borrow().high - begin_klu.borrow().low) / begin_klu.borrow().low
        }
    }

    pub fn update_bi_list(&mut self, bi_lst: &[SharedCell<LINE_TYPE>], idx1: usize, idx2: usize) {
        for bi_idx in idx1..=idx2 {
            bi_lst[bi_idx].borrow_mut().parent_seg = Some(Rc::new(RefCell::new(self.clone())));
            self.bi_list.push(bi_lst[bi_idx].clone());
        }
        if self.bi_list.len() >= 3 {
            self.support_trend_line = Some(CTrendLine::new(&self.bi_list, TrendLineSide::Inside));
            self.resistance_trend_line =
                Some(CTrendLine::new(&self.bi_list, TrendLineSide::Outside));
        }
    }

    pub fn get_first_multi_bi_zs(&self) -> Option<SharedCell<CZS>> {
        self.zs_lst
            .iter()
            .find(|zs| !zs.borrow().is_one_bi_zs())
            .cloned()
    }

    pub fn get_final_multi_bi_zs(&self) -> Option<SharedCell<CZS>> {
        self.zs_lst
            .iter()
            .rev()
            .find(|zs| !zs.borrow().is_one_bi_zs())
            .cloned()
    }

    pub fn get_multi_bi_zs_cnt(&self) -> usize {
        self.zs_lst
            .iter()
            .filter(|zs| !zs.borrow().is_one_bi_zs())
            .count()
    }
}

impl<LINE_TYPE> std::fmt::Display for CSeg<LINE_TYPE> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}->{}:{:?} {}",
            self.start_bi.borrow().idx,
            self.end_bi.borrow().idx,
            self.dir,
            self.is_sure
        )
    }
}
