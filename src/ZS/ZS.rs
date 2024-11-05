use crate::BuySellPoint::BSPointConfig::CPointConfig;
use crate::Common::func_util::has_overlap;
use crate::Common::types::{StrongHandle, WeakHandle};
use crate::Common::CEnum::ZsCombineMode;
use crate::KLine::KLine_Unit::CKLineUnit;
use crate::Seg::linetype::Line;
use crate::Seg::Seg::CSeg;
use std::cell::RefCell;
use std::rc::Rc;

pub struct CZS<T> {
    pub is_sure: bool,
    pub sub_zs_lst: Vec<StrongHandle<CZS<T>>>,
    pub begin: Option<WeakHandle<CKLineUnit>>,
    pub begin_bi: Option<WeakHandle<T>>,
    pub low: f64,
    pub high: f64,
    pub mid: f64,
    pub end: Option<WeakHandle<CKLineUnit>>,
    pub end_bi: Option<WeakHandle<T>>,
    pub peak_high: f64,
    pub peak_low: f64,
    pub bi_in: Option<WeakHandle<T>>,
    pub bi_out: Option<WeakHandle<T>>,
    pub bi_lst: Vec<WeakHandle<T>>,
}

impl<T: Line> CZS<T> {
    pub fn new(lst: Option<Vec<StrongHandle<T>>>, is_sure: bool) -> Self {
        // begin/end：永远指向 klu
        // low/high: 中枢的范围
        // peak_low/peak_high: 中枢所涉及到的笔的最大值，最小值
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
            bi_in: None,        //进中枢那一笔
            bi_out: None,       //出中枢那一笔
            bi_lst: Vec::new(), // begin_bi~end_bi之间的笔，在update_zs_in_seg函数中更新
        };

        if let Some(lst) = lst {
            if !lst.is_empty() {
                zs.begin = Some(Rc::downgrade(&lst[0].borrow().line_get_begin_klu()));
                zs.begin_bi = Some(Rc::downgrade(&lst[0]));
                zs.update_zs_range(&lst);
                for item in lst {
                    zs.update_zs_end(&item);
                }
            }
        }
        zs
    }

    //pub fn clean_cache(&mut self) {}

    pub fn update_zs_range(&mut self, lst: &[StrongHandle<T>]) {
        self.low = lst
            .iter()
            .map(|bi| bi.borrow().line_low())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        self.high = lst
            .iter()
            .map(|bi| bi.borrow().line_high())
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        self.mid = (self.low + self.high) / 2.0;
        //self.clean_cache();
    }

    pub fn is_one_bi_zs(&self) -> bool {
        self.end_bi
            .as_ref()
            .and_then(|end_bi| {
                self.begin_bi.as_ref().map(|begin_bi| {
                    begin_bi.upgrade().unwrap().borrow().line_idx()
                        == end_bi.upgrade().unwrap().borrow().line_idx()
                })
            })
            .unwrap_or(false)
    }

    pub fn update_zs_end(&mut self, item: &StrongHandle<T>) {
        self.end = Some(Rc::downgrade(&item.borrow().line_get_end_klu()));
        self.end_bi = Some(Rc::downgrade(item));
        if item.borrow().line_low() < self.peak_low {
            self.peak_low = item.borrow().line_low();
        }
        if item.borrow().line_high() > self.peak_high {
            self.peak_high = item.borrow().line_high();
        }
        //self.clean_cache();
    }

    pub fn combine(&mut self, zs2: &CZS<T>, combine_mode: ZsCombineMode) -> bool {
        if zs2.is_one_bi_zs() {
            return false;
        }
        if self
            .begin_bi
            .as_ref()
            .unwrap()
            .upgrade()
            .unwrap()
            .borrow()
            .line_seg_idx()
            != zs2
                .begin_bi
                .as_ref()
                .unwrap()
                .upgrade()
                .unwrap()
                .borrow()
                .line_seg_idx()
        {
            return false;
        }
        match combine_mode {
            ZsCombineMode::Zs => {
                if !has_overlap(self.low, self.high, zs2.low, zs2.high, true) {
                    return false;
                }
                self.do_combine(zs2);
                true
            }
            ZsCombineMode::Peak => {
                if has_overlap(
                    self.peak_low,
                    self.peak_high,
                    zs2.peak_low,
                    zs2.peak_high,
                    false,
                ) {
                    self.do_combine(zs2);
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn do_combine(&mut self, zs2: &CZS<T>) {
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
        //self.clean_cache();
    }

    pub fn try_add_to_end(&mut self, item: &StrongHandle<T>) -> bool {
        if !self.in_range(item) {
            return false;
        }
        if self.is_one_bi_zs() {
            self.update_zs_range(&[
                self.begin_bi.as_ref().unwrap().upgrade().unwrap().clone(),
                item.clone(),
            ]);
        }
        self.update_zs_end(item);
        true
    }

    pub fn in_range(&self, item: &StrongHandle<T>) -> bool {
        has_overlap(
            self.low,
            self.high,
            item.borrow().line_low(),
            item.borrow().line_high(),
            false,
        )
    }

    pub fn is_inside(&self, seg: &CSeg<T>) -> bool {
        seg.start_bi.borrow().line_idx()
            <= self
                .begin_bi
                .as_ref()
                .unwrap()
                .upgrade()
                .unwrap()
                .borrow()
                .line_idx()
            && self
                .begin_bi
                .as_ref()
                .unwrap()
                .upgrade()
                .unwrap()
                .borrow()
                .line_idx()
                <= seg.end_bi.borrow().line_idx()
    }

    pub fn is_divergence(
        &self,
        config: &CPointConfig,
        out_bi: Option<StrongHandle<T>>,
    ) -> (bool, Option<f64>) {
        let out_bi = out_bi.as_ref().map(|x| Rc::clone(&x));
        if !self.end_bi_break(out_bi.clone()) {
            return (false, None);
        }
        let in_metric = self
            .get_bi_in()
            .borrow()
            .line_cal_macd_metric(config.macd_algo, false)
            .unwrap();
        let out_metric = match &out_bi {
            Some(out_bi) => out_bi
                .borrow()
                .line_cal_macd_metric(config.macd_algo, true)
                .unwrap(),
            None => self
                .get_bi_out()
                .borrow()
                .line_cal_macd_metric(config.macd_algo, true)
                .unwrap(),
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

    pub fn init_from_zs(&mut self, zs: &CZS<T>) {
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

    pub fn make_copy(&self) -> CZS<T> {
        let mut copy = CZS::new(None, self.is_sure);
        copy.init_from_zs(self);
        copy
    }

    pub fn end_bi_break(&self, end_bi: Option<StrongHandle<T>>) -> bool {
        let end_bi = end_bi.unwrap_or_else(|| self.get_bi_out());
        let end_bi = end_bi.borrow();
        (end_bi.line_is_down() && end_bi.line_low() < self.low)
            || (end_bi.line_is_up() && end_bi.line_high() > self.high)
    }

    pub fn out_bi_is_peak(&self, end_bi_idx: usize) -> (bool, Option<f64>) {
        assert!(!self.bi_lst.is_empty());
        if self.bi_out.is_none() {
            return (false, None);
        }
        let bi_out = self.bi_out.as_ref().unwrap().upgrade().unwrap();
        let bi_out_ref = bi_out.borrow();
        let mut peak_rate = f64::INFINITY;
        for bi in &self.bi_lst {
            let bi_strong = bi.upgrade().unwrap();
            let bi_ref = bi_strong.borrow();
            if bi_ref.line_idx() > end_bi_idx {
                break;
            }
            if (bi_out_ref.line_is_down() && bi_ref.line_low() < bi_out_ref.line_low())
                || (bi_out_ref.line_is_up() && bi_ref.line_high() > bi_out_ref.line_high())
            {
                return (false, None);
            }
            let r = (bi_ref.line_get_end_val() - bi_out_ref.line_get_end_val()).abs()
                / bi_out_ref.line_get_end_val();
            if r < peak_rate {
                peak_rate = r;
            }
        }
        (true, Some(peak_rate))
    }

    pub fn get_bi_in(&self) -> StrongHandle<T> {
        self.bi_in
            .as_ref()
            .expect("bi_in is None")
            .upgrade()
            .expect("bi_in was dropped")
    }

    pub fn get_bi_out(&self) -> StrongHandle<T> {
        self.bi_out
            .as_ref()
            .expect("bi_out is None")
            .upgrade()
            .expect("bi_out was dropped")
    }

    pub fn set_bi_in(&mut self, bi: StrongHandle<T>) {
        self.bi_in = Some(Rc::downgrade(&bi));
        //self.clean_cache();
    }

    pub fn set_bi_out(&mut self, bi: StrongHandle<T>) {
        self.bi_out = Some(Rc::downgrade(&bi));
        //self.clean_cache();
    }

    pub fn set_bi_lst(&mut self, bi_lst: &[StrongHandle<T>]) {
        self.bi_lst = bi_lst.iter().map(|x| Rc::downgrade(x)).collect();
    }
}

impl<T: Line> std::fmt::Display for CZS<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let main_str = format!(
            "{}->{}",
            self.begin_bi
                .as_ref()
                .map_or(0, |bi| bi.upgrade().unwrap().borrow().line_idx()),
            self.end_bi
                .as_ref()
                .map_or(0, |bi| bi.upgrade().unwrap().borrow().line_idx())
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
