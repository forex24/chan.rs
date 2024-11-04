use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::{Rc, Weak};

use crate::BuySellPoint::BS_Point::CBSPoint;
use crate::Common::types::Handle;
use crate::Common::CEnum::{BiDir, MacdAlgo};
use crate::Common::ChanException::{CChanException, ErrCode};
use crate::KLine::KLine_Unit::CKLineUnit;
use crate::ZS::ZS::CZS;

use super::linetype::Line;
use super::EigenFX::CEigenFX;

//#[derive(Debug, Clone)]
pub struct CSeg<T> {
    pub idx: usize,
    pub start_bi: Handle<T>,
    pub end_bi: Handle<T>,
    pub is_sure: bool,
    pub dir: BiDir,
    pub zs_lst: VecDeque<Handle<CZS<T>>>,
    pub eigen_fx: Option<CEigenFX<T>>,
    pub seg_idx: Option<usize>,
    pub parent_seg: Option<Weak<RefCell<CSeg<Self>>>>,
    pub pre: Option<Handle<CSeg<T>>>,
    pub next: Option<Handle<CSeg<T>>>,
    pub bsp: Option<Handle<CBSPoint<CSeg<T>>>>,
    pub bi_list: Vec<Handle<T>>,
    pub reason: String,
    //pub support_trend_line: Option<CTrendLine>,
    //pub resistance_trend_line: Option<CTrendLine>,
    pub ele_inside_is_sure: bool,
}

impl<T: Line> CSeg<T> {
    pub fn new(
        idx: usize,
        start_bi: Handle<T>,
        end_bi: Handle<T>,
        is_sure: bool,
        seg_dir: Option<BiDir>,
        reason: &str,
    ) -> Result<Self, CChanException> {
        assert!(
            start_bi.borrow().line_idx() == 0
                || start_bi.borrow().line_dir() == end_bi.borrow().line_dir()
                || !is_sure,
            "start_bi and end_bi direction mismatch"
        );

        let is_sure = if end_bi.borrow().line_idx() - start_bi.borrow().line_idx() < 2 {
            false
        } else {
            is_sure
        };

        let dir = seg_dir.unwrap_or_else(|| end_bi.borrow().line_dir());
        let seg = Self {
            idx,
            start_bi,
            end_bi,
            is_sure,
            dir,
            zs_lst: VecDeque::new(),
            eigen_fx: None,
            seg_idx: None,
            parent_seg: None,
            pre: None,
            next: None,
            bsp: None,
            bi_list: Vec::new(),
            reason: reason.to_string(),
            //support_trend_line: None,
            //resistance_trend_line: None,
            ele_inside_is_sure: false,
        };

        seg.check()?;
        Ok(seg)
    }

    pub fn set_seg_idx(&mut self, idx: usize) {
        self.seg_idx = Some(idx);
    }

    pub fn check(&self) -> Result<(), CChanException> {
        if !self.is_sure {
            return Ok(());
        }

        let start_val = self.start_bi.borrow().line_get_begin_val();
        let end_val = self.end_bi.borrow().line_get_end_val();
        let start_idx = self.start_bi.borrow().line_idx();
        let end_idx = self.end_bi.borrow().line_idx();

        if self.is_down() {
            if start_val < end_val {
                return Err(CChanException::new(
                    format!("下降线段起始点应该高于结束点! idx={}", self.idx).to_string(),
                    ErrCode::SegEndValueErr,
                ));
            }
        } else if start_val > end_val {
            return Err(CChanException::new(
                format!("上升线段起始点应该低于结束点! idx={}", self.idx).to_string(),
                ErrCode::SegEndValueErr,
            ));
        }

        if end_idx - start_idx < 2 {
            return Err(CChanException::new(
                format!(
                    "线段({}-{})长度不能小于2! idx={}",
                    start_idx, end_idx, self.idx
                )
                .to_string(),
                ErrCode::SegLenErr,
            ));
        }

        Ok(())
    }

    pub fn add_zs(&mut self, zs: Handle<CZS<T>>) {
        self.zs_lst.push_front(zs);
    }

    pub fn cal_klu_slope(&self) -> f64 {
        assert!(self.end_bi.borrow().line_idx() >= self.start_bi.borrow().line_idx());
        let end_val = self.get_end_val();
        let begin_val = self.get_begin_val();
        let end_idx = self.get_end_klu().borrow().idx;
        let begin_idx = self.get_begin_klu().borrow().idx;

        (end_val - begin_val) / ((end_idx - begin_idx) as f64) / begin_val
    }

    pub fn cal_amp(&self) -> f64 {
        (self.get_end_val() - self.get_begin_val()) / self.get_begin_val()
    }

    pub fn cal_bi_cnt(&self) -> usize {
        self.end_bi.borrow().line_idx() - self.start_bi.borrow().line_idx() + 1
    }

    pub fn clear_zs_lst(&mut self) {
        self.zs_lst.clear();
    }

    pub fn low(&self) -> f64 {
        if self.is_down() {
            self.end_bi.borrow().line_get_end_klu().borrow().low
        } else {
            self.start_bi.borrow().line_get_begin_klu().borrow().low
        }
    }

    pub fn high(&self) -> f64 {
        if self.is_up() {
            self.end_bi.borrow().line_get_end_klu().borrow().high
        } else {
            self.start_bi.borrow().line_get_begin_klu().borrow().high
        }
    }

    pub fn is_down(&self) -> bool {
        self.dir == BiDir::Down
    }

    pub fn is_up(&self) -> bool {
        self.dir == BiDir::Up
    }

    pub fn get_end_val(&self) -> f64 {
        self.end_bi.borrow().line_get_end_val()
    }

    pub fn get_begin_val(&self) -> f64 {
        self.start_bi.borrow().line_get_begin_val()
    }

    pub fn amp(&self) -> f64 {
        (self.get_end_val() - self.get_begin_val()).abs()
    }

    pub fn get_end_klu(&self) -> Handle<CKLineUnit> {
        self.end_bi.borrow().line_get_end_klu()
    }

    pub fn get_begin_klu(&self) -> Handle<CKLineUnit> {
        self.start_bi.borrow().line_get_begin_klu()
    }

    pub fn get_klu_cnt(&self) -> usize {
        self.get_end_klu().borrow().idx - self.get_begin_klu().borrow().idx + 1
    }

    pub fn cal_macd_metric(
        &self,
        macd_algo: MacdAlgo,
        _is_reverse: bool,
    ) -> Result<f64, CChanException> {
        match macd_algo {
            MacdAlgo::Slope => Ok(self.cal_macd_slope()),
            MacdAlgo::Amp => Ok(self.cal_macd_amp()),
            _ => Err(CChanException::new(
                format!(
                    "unsupport macd_algo={:?} of Seg, should be one of slope/amp",
                    macd_algo
                )
                .to_string(),
                ErrCode::ParaError,
            )),
        }
    }

    // 计算MACD斜率
    pub fn cal_macd_slope(&self) -> f64 {
        let begin_klu = self.get_begin_klu();
        let end_klu = self.get_end_klu();
        let begin_klu = begin_klu.borrow();
        let end_klu = end_klu.borrow();

        if self.is_up() {
            (end_klu.high - begin_klu.low)
                / end_klu.high
                / ((end_klu.idx - begin_klu.idx + 1) as f64)
        } else {
            (begin_klu.high - end_klu.low)
                / begin_klu.high
                / ((end_klu.idx - begin_klu.idx + 1) as f64)
        }
    }

    // 计算MACD强度
    pub fn cal_macd_amp(&self) -> f64 {
        let begin_klu = self.get_begin_klu();
        let end_klu = self.get_end_klu();
        let begin_klu = begin_klu.borrow();
        let end_klu = end_klu.borrow();

        if self.is_down() {
            (begin_klu.high - end_klu.low) / begin_klu.high
        } else {
            (end_klu.high - begin_klu.low) / begin_klu.low
        }
    }

    pub fn update_bi_list(
        &mut self,
        bi_lst: &[Handle<T>],
        idx1: usize,
        idx2: usize,
        parent: Handle<CSeg<T>>,
    ) {
        for bi_idx in idx1..=idx2 {
            let bi = bi_lst.get(bi_idx).unwrap().clone();
            bi.borrow_mut().line_set_parent_seg(Some(parent.clone()));
            self.bi_list.push(bi);
        }

        //if self.bi_list.len() >= 3 {
        //    self.support_trend_line = Some(CTrendLine::new(&self.bi_list, TrendLineSide::Inside));
        //    self.resistance_trend_line =
        //        Some(CTrendLine::new(&self.bi_list, TrendLineSide::Outside));
        //}
    }

    pub fn get_first_multi_bi_zs(&self) -> Option<Handle<CZS<T>>> {
        self.zs_lst
            .iter()
            .find(|zs| !zs.borrow().is_one_bi_zs())
            .cloned()
    }

    pub fn get_final_multi_bi_zs(&self) -> Option<Handle<CZS<T>>> {
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

impl<T: Line> std::fmt::Display for CSeg<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}->{}: {:?}  {}",
            self.start_bi.borrow().line_idx(),
            self.end_bi.borrow().line_idx(),
            self.dir,
            self.is_sure
        )
    }
}
