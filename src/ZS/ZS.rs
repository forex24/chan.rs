use crate::BuySellPoint::BSPointConfig::CPointConfig;
//use crate::BuySellPoint::BSPointConfig::CPointConfig;
use crate::Common::func_util::has_overlap;
use crate::Common::types::Handle;
use crate::Common::CEnum::ZsCombineMode;
use crate::KLine::KLine_Unit::CKLineUnit;
use crate::Seg::linetype::Line;
use crate::Seg::Seg::CSeg;
use std::cell::RefCell;
use std::rc::Rc;

pub struct CZS<T> {
    pub is_sure: bool,
    pub sub_zs_lst: Vec<Handle<CZS<T>>>,
    pub begin: Option<Handle<CKLineUnit>>,
    pub begin_bi: Option<Handle<T>>,
    pub low: f64,
    pub high: f64,
    pub mid: f64,
    pub end: Option<Handle<CKLineUnit>>,
    pub end_bi: Option<Handle<T>>,
    pub peak_high: f64,
    pub peak_low: f64,
    pub bi_in: Option<Handle<T>>,
    pub bi_out: Option<Handle<T>>,
    pub bi_lst: Vec<Handle<T>>,
}

impl<T: Line> CZS<T> {
    pub fn new(lst: Option<Vec<Handle<T>>>, is_sure: bool) -> Self {
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
                zs.begin = Some(lst[0].borrow().get_begin_klu());
                zs.begin_bi = Some(lst[0].clone());
                zs.update_zs_range(&lst);
                for item in lst {
                    zs.update_zs_end(&item);
                }
            }
        }
        zs
    }

    //pub fn clean_cache(&mut self) {}

    pub fn update_zs_range(&mut self, lst: &[Handle<T>]) {
        self.low = lst
            .iter()
            .map(|bi| bi.borrow().low())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        self.high = lst
            .iter()
            .map(|bi| bi.borrow().high())
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        self.mid = (self.low + self.high) / 2.0;
        //self.clean_cache();
    }

    pub fn is_one_bi_zs(&self) -> bool {
        self.end_bi.as_ref().map_or(false, |end_bi| {
            self.begin_bi.as_ref().map_or(false, |begin_bi| {
                begin_bi.borrow().idx() == end_bi.borrow().idx()
            })
        })
    }

    pub fn update_zs_end(&mut self, item: &Handle<T>) {
        self.end = Some(item.borrow().get_end_klu());
        self.end_bi = Some(item.clone());
        if item.borrow().low() < self.peak_low {
            self.peak_low = item.borrow().low();
        }
        if item.borrow().high() > self.peak_high {
            self.peak_high = item.borrow().high();
        }
        //self.clean_cache();
    }

    pub fn combine(&mut self, zs2: &CZS<T>, combine_mode: ZsCombineMode) -> bool {
        if zs2.is_one_bi_zs() {
            return false;
        }
        if self.begin_bi.as_ref().unwrap().borrow().seg_idx()
            != zs2.begin_bi.as_ref().unwrap().borrow().seg_idx()
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

    pub fn try_add_to_end(&mut self, item: &Handle<T>) -> bool {
        if !self.in_range(item) {
            return false;
        }
        if self.is_one_bi_zs() {
            self.update_zs_range(&[self.begin_bi.as_ref().unwrap().clone(), item.clone()]);
        }
        self.update_zs_end(item);
        true
    }

    pub fn in_range(&self, item: &Handle<T>) -> bool {
        has_overlap(
            self.low,
            self.high,
            item.borrow().low(),
            item.borrow().high(),
            false,
        )
    }

    pub fn is_inside<U: Line>(&self, seg: &CSeg<U>) -> bool {
        seg.start_bi.borrow().idx() <= self.begin_bi.as_ref().unwrap().borrow().idx()
            && self.begin_bi.as_ref().unwrap().borrow().idx() <= seg.end_bi.borrow().idx()
    }

    pub fn is_divergence(
        &self,
        config: &CPointConfig,
        out_bi: Option<Handle<T>>,
    ) -> (bool, Option<f64>) {
        let out_bi = out_bi.map(|x| Rc::clone(&x));
        if !self.end_bi_break(out_bi) {
            return (false, None);
        }
        let in_metric = self
            .get_bi_in()
            .borrow()
            .cal_macd_metric(config.macd_algo, false)
            .unwrap();
        let out_metric = if let Some(out_bi) = out_bi {
            out_bi
                .borrow()
                .cal_macd_metric(config.macd_algo, true)
                .unwrap()
        } else {
            self.get_bi_out()
                .borrow()
                .cal_macd_metric(config.macd_algo, true)
                .unwrap()
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

    pub fn end_bi_break(&self, end_bi: Option<Handle<T>>) -> bool {
        let end_bi = end_bi.unwrap_or_else(|| Rc::clone(self.get_bi_out()));
        let end_bi = end_bi.borrow();
        (end_bi.is_down() && end_bi.low() < self.low)
            || (end_bi.is_up() && end_bi.high() > self.high)
    }

    pub fn out_bi_is_peak(&self, end_bi_idx: usize) -> (bool, Option<f64>) {
        assert!(!self.bi_lst.is_empty());
        if self.bi_out.is_none() {
            return (false, None);
        }
        let bi_out = self.bi_out.as_ref().unwrap().borrow();
        let mut peak_rate = f64::INFINITY;
        for bi in &self.bi_lst {
            let bi_ref = bi.borrow();
            if bi_ref.idx() > end_bi_idx {
                break;
            }
            if (bi_out.is_down() && bi_ref.low() < bi_out.low())
                || (bi_out.is_up() && bi_ref.high() > bi_out.high())
            {
                return (false, None);
            }
            let r = (bi_ref.get_end_val() - bi_out.get_end_val()).abs() / bi_out.get_end_val();
            if r < peak_rate {
                peak_rate = r;
            }
        }
        (true, Some(peak_rate))
    }

    pub fn get_bi_in(&self) -> &Handle<T> {
        self.bi_in.as_ref().expect("bi_in is None")
    }

    pub fn get_bi_out(&self) -> &Handle<T> {
        self.bi_out.as_ref().expect("bi_out is None")
    }

    pub fn set_bi_in(&mut self, bi: Handle<T>) {
        self.bi_in = Some(bi);
        //self.clean_cache();
    }

    pub fn set_bi_out(&mut self, bi: Handle<T>) {
        self.bi_out = Some(bi);
        //self.clean_cache();
    }

    pub fn set_bi_lst(&mut self, bi_lst: &[Handle<T>]) {
        self.bi_lst = bi_lst.to_vec();
        //self.clean_cache();
    }
}

//impl<T: Line> std::fmt::Display for CZS<T> {
//    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//        let main_str = format!(
//            "{}->{}",
//            self.begin_bi.as_ref().map_or(0, |bi| bi.idx()),
//            self.end_bi.as_ref().map_or(0, |bi| bi.idx())
//        );
//        let sub_str: String = self
//            .sub_zs_lst
//            .iter()
//            .map(|sub_zs| sub_zs.borrow().to_string())
//            .collect::<Vec<_>>()
//            .join(",");
//        if !sub_str.is_empty() {
//            write!(f, "{}({})", main_str, sub_str)
//        } else {
//            write!(f, "{}", main_str)
//        }
//    }
//}
