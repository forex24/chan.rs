use crate::bi::bi::CBi;
use crate::buy_sell_point::bs_point_config::CPointConfig;
use crate::common::chan_exception::{CChanException, ErrCode};
use crate::common::func_util::has_overlap;
use crate::kline::kline_unit::CKLineUnit;
use crate::seg::seg::CSeg;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub enum LineType {
    Bi(Rc<RefCell<CBi>>),
    Seg(Rc<RefCell<CSeg>>),
}

pub struct CZS {
    is_sure: bool,
    sub_zs_lst: Vec<Rc<RefCell<CZS>>>,
    begin: Option<Rc<RefCell<CKLineUnit>>>,
    begin_bi: Option<LineType>,
    low: f64,
    high: f64,
    mid: f64,
    end: Option<Rc<RefCell<CKLineUnit>>>,
    end_bi: Option<LineType>,
    peak_high: f64,
    peak_low: f64,
    bi_in: Option<LineType>,
    bi_out: Option<LineType>,
    bi_lst: Vec<LineType>,
    memoize_cache: HashMap<String, Rc<RefCell<dyn Any>>>,
}

impl CZS {
    pub fn new(lst: Option<Vec<LineType>>, is_sure: bool) -> Self {
        let mut zs = CZS {
            is_sure,
            sub_zs_lst: Vec::new(),
            begin: None,
            begin_bi: None,
            low: 0.0,
            high: 0.0,
            mid: 0.0,
            end: None,
            end_bi: None,
            peak_high: f64::NEG_INFINITY,
            peak_low: f64::INFINITY,
            bi_in: None,
            bi_out: None,
            bi_lst: Vec::new(),
            memoize_cache: HashMap::new(),
        };

        if let Some(lst) = lst {
            if !lst.is_empty() {
                zs.begin = Some(lst[0].get_begin_klu());
                zs.begin_bi = Some(lst[0].clone());
                zs.update_zs_range(&lst);
                for item in lst {
                    zs.update_zs_end(&item);
                }
            }
        }

        zs
    }

    pub fn clean_cache(&mut self) {
        self.memoize_cache.clear();
    }

    pub fn update_zs_range(&mut self, lst: &[LineType]) {
        self.low = lst
            .iter()
            .map(|bi| bi._low())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        self.high = lst
            .iter()
            .map(|bi| bi._high())
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        self.mid = (self.low + self.high) / 2.0;
        self.clean_cache();
    }

    pub fn is_one_bi_zs(&self) -> bool {
        self.end_bi.as_ref().map_or(false, |end_bi| {
            self.begin_bi
                .as_ref()
                .map_or(false, |begin_bi| begin_bi.idx() == end_bi.idx())
        })
    }

    pub fn update_zs_end(&mut self, item: &LineType) {
        self.end = Some(item.get_end_klu());
        self.end_bi = Some(item.clone());
        if item._low() < self.peak_low {
            self.peak_low = item._low();
        }
        if item._high() > self.peak_high {
            self.peak_high = item._high();
        }
        self.clean_cache();
    }

    pub fn combine(&mut self, zs2: &CZS, combine_mode: &str) -> Result<bool, CChanException> {
        if zs2.is_one_bi_zs() {
            return Ok(false);
        }
        if self.begin_bi.as_ref().unwrap().seg_idx() != zs2.begin_bi.as_ref().unwrap().seg_idx() {
            return Ok(false);
        }
        match combine_mode {
            "zs" => {
                if !has_overlap(self.low, self.high, zs2.low, zs2.high, true) {
                    return Ok(false);
                }
                self.do_combine(zs2);
                Ok(true)
            }
            "peak" => {
                if has_overlap(
                    self.peak_low,
                    self.peak_high,
                    zs2.peak_low,
                    zs2.peak_high,
                    false,
                ) {
                    self.do_combine(zs2);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            _ => Err(CChanException::new(
                &format!("{} is unsupport zs combine mode", combine_mode),
                ErrCode::ParaError,
            )),
        }
    }

    pub fn do_combine(&mut self, zs2: &CZS) {
        if self.sub_zs_lst.is_empty() {
            self.sub_zs_lst
                .push(Rc::new(RefCell::new(self.make_copy())));
        }
        self.sub_zs_lst.push(Rc::new(RefCell::new(zs2.make_copy())));

        self.low = self.low.min(zs2.low);
        self.high = self.high.max(zs2.high);
        self.peak_low = self.peak_low.min(zs2.peak_low);
        self.peak_high = self.peak_high.max(zs2.peak_high);
        self.end = zs2.end.clone();
        self.bi_out = zs2.bi_out.clone();
        self.end_bi = zs2.end_bi.clone();
        self.clean_cache();
    }

    pub fn try_add_to_end(&mut self, item: &LineType) -> bool {
        if !self.in_range(item) {
            return false;
        }
        if self.is_one_bi_zs() {
            self.update_zs_range(&[self.begin_bi.as_ref().unwrap().clone(), item.clone()]);
        }
        self.update_zs_end(item);
        true
    }

    pub fn in_range(&self, item: &LineType) -> bool {
        has_overlap(self.low, self.high, item._low(), item._high(), false)
    }

    pub fn is_inside(&self, seg: &CSeg) -> bool {
        seg.start_bi.borrow().idx <= self.begin_bi.as_ref().unwrap().idx()
            && self.begin_bi.as_ref().unwrap().idx() <= seg.end_bi.borrow().idx
    }

    pub fn is_divergence(
        &self,
        config: &CPointConfig,
        out_bi: Option<&LineType>,
    ) -> (bool, Option<f64>) {
        if !self.end_bi_break(out_bi) {
            return (false, None);
        }
        let in_metric = self.get_bi_in().cal_macd_metric(config.macd_algo, false);
        let out_metric = if let Some(out_bi) = out_bi {
            out_bi.cal_macd_metric(config.macd_algo, true)
        } else {
            self.get_bi_out().cal_macd_metric(config.macd_algo, true)
        };

        let ratio = out_metric / in_metric;
        if config.divergence_rate > 100.0 {
            (true, Some(ratio))
        } else {
            (
                out_metric <= config.divergence_rate * in_metric,
                Some(ratio),
            )
        }
    }

    pub fn init_from_zs(&mut self, zs: &CZS) {
        self.begin = zs.begin.clone();
        self.end = zs.end.clone();
        self.low = zs.low;
        self.high = zs.high;
        self.peak_high = zs.peak_high;
        self.peak_low = zs.peak_low;
        self.begin_bi = zs.begin_bi.clone();
        self.end_bi = zs.end_bi.clone();
        self.bi_in = zs.bi_in.clone();
        self.bi_out = zs.bi_out.clone();
    }

    pub fn make_copy(&self) -> CZS {
        let mut copy = CZS::new(None, self.is_sure);
        copy.init_from_zs(self);
        copy
    }

    pub fn end_bi_break(&self, end_bi: Option<&LineType>) -> bool {
        let end_bi = end_bi.unwrap_or_else(|| self.get_bi_out());
        (end_bi.is_down() && end_bi._low() < self.low)
            || (end_bi.is_up() && end_bi._high() > self.high)
    }

    pub fn out_bi_is_peak(&self, end_bi_idx: i32) -> (bool, Option<f64>) {
        assert!(!self.bi_lst.is_empty());
        if self.bi_out.is_none() {
            return (false, None);
        }
        let bi_out = self.bi_out.as_ref().unwrap();
        let mut peak_rate = f64::INFINITY;
        for bi in &self.bi_lst {
            if bi.idx() > end_bi_idx {
                break;
            }
            if (bi_out.is_down() && bi._low() < bi_out._low())
                || (bi_out.is_up() && bi._high() > bi_out._high())
            {
                return (false, None);
            }
            let r = (bi.get_end_val() - bi_out.get_end_val()).abs() / bi_out.get_end_val();
            if r < peak_rate {
                peak_rate = r;
            }
        }
        (true, Some(peak_rate))
    }

    pub fn get_bi_in(&self) -> &LineType {
        self.bi_in.as_ref().expect("bi_in is None")
    }

    pub fn get_bi_out(&self) -> &LineType {
        self.bi_out.as_ref().expect("bi_out is None")
    }

    pub fn set_bi_in(&mut self, bi: LineType) {
        self.bi_in = Some(bi);
        self.clean_cache();
    }

    pub fn set_bi_out(&mut self, bi: LineType) {
        self.bi_out = Some(bi);
        self.clean_cache();
    }

    pub fn set_bi_lst(&mut self, bi_lst: Vec<LineType>) {
        self.bi_lst = bi_lst;
        self.clean_cache();
    }
}

impl std::fmt::Display for CZS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let main_str = format!(
            "{}->{}",
            self.begin_bi.as_ref().map_or(0, |bi| bi.idx()),
            self.end_bi.as_ref().map_or(0, |bi| bi.idx())
        );
        let sub_str: String = self
            .sub_zs_lst
            .iter()
            .map(|sub_zs| sub_zs.borrow().to_string())
            .collect::<Vec<_>>()
            .join(",");
        if !sub_str.is_empty() {
            write!(f, "{}({})", main_str, sub_str)
        } else {
            write!(f, "{}", main_str)
        }
    }
}
