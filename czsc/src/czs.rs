use crate::{
    has_overlap, AsHandle, Bar, CPivotCombineMode, CPointConfig, CSeg, Handle, ICalcMetric,
    IParent, LineType, ToHandle,
};

#[derive(Debug)]
pub struct CZs<T> {
    // begin/end：永远指向 klu
    // low/high: 中枢的范围
    // peak_low/peak_high: 中枢所涉及到的笔的最大值，最小值
    handle: Handle<Self>,
    pub is_sure: bool,
    pub sub_zs_lst: Vec<Handle<CZs<T>>>,
    pub begin: Handle<Bar>,
    pub begin_bi: Handle<T>, // 中枢内部的笔
    pub low: f64,
    pub high: f64,
    pub end: Option<Handle<Bar>>,
    pub end_bi: Option<Handle<T>>,
    pub peak_high: f64,
    pub peak_low: f64,
    pub bi_in: Option<Handle<T>>,
    pub bi_out: Option<Handle<T>>,
    pub bi_lst: Vec<Handle<T>>,
}

impl<T: LineType + IParent + ICalcMetric + ToHandle> CZs<T> {
    // 99% 完备，语义有点区别
    #[allow(clippy::borrowed_box)]
    pub fn new(
        box_vec: &Box<Vec<Self>>,
        zs_index: usize,
        lst: &[Handle<T>],
        is_sure: bool,
    ) -> Self {
        assert!(!lst.is_empty());
        let mut zs = Self {
            handle: Handle::new(box_vec, zs_index),
            is_sure,
            sub_zs_lst: Vec::new(),
            begin: lst[0].get_begin_klu().as_handle(),
            begin_bi: lst[0],
            end: None,
            end_bi: None,
            high: 0.0,
            low: 0.0,
            peak_high: f64::NEG_INFINITY,
            peak_low: f64::INFINITY,
            bi_in: None,                      //进中枢那一笔
            bi_out: None,                     //出中枢那一笔
            bi_lst: Vec::with_capacity(1024), // begin_bi~end_bi之间的笔，在update_zs_in_seg函数中更新
        };

        zs.update_zs_range(lst);

        for item in lst {
            zs.update_zs_end(*item);
        }

        zs
    }

    // 已完备
    fn update_zs_range(&mut self, lst: &[Handle<T>]) {
        self.low = lst
            .iter()
            .map(|bi| bi.low())
            .reduce(f64::max)
            .unwrap_or(f64::NEG_INFINITY);
        self.high = lst
            .iter()
            .map(|bi| bi.high())
            .reduce(f64::min)
            .unwrap_or(f64::INFINITY);
    }

    // 已完备
    fn update_zs_end(&mut self, item: Handle<T>) {
        self.end = Some(item.get_end_klu());
        self.end_bi = Some(item);
        if item.low() < self.peak_low {
            self.peak_low = item.low();
        }
        if item.high() > self.peak_high {
            self.peak_high = item.high();
        }
    }

    // 已完备
    pub(crate) fn can_combine(&self, zs2: &CZs<T>, combine_mode: CPivotCombineMode) -> bool {
        if zs2.is_one_bi_zs() {
            return false;
        }
        if self.begin_bi.seg_idx() != zs2.begin_bi.seg_idx() {
            return false;
        }

        match combine_mode {
            CPivotCombineMode::Zs => {
                if !has_overlap(self.low, self.high, zs2.low, zs2.high, true) {
                    return false;
                }
                true
            }
            CPivotCombineMode::Peak => has_overlap(
                self.peak_low,
                self.peak_high,
                zs2.peak_low,
                zs2.peak_high,
                false,
            ),
        }
    }

    // TODO: self.__sub_zs_lst.append(self.make_copy())
    pub(crate) fn do_combine(&mut self, rhs: &CZs<T>) {
        if self.sub_zs_lst.is_empty() {
            self.sub_zs_lst.push(self.handle);
        }
        self.sub_zs_lst.push(rhs.handle);
        self.low = self.low.min(rhs.low);
        self.high = self.high.max(rhs.high);
        self.peak_low = self.peak_low.min(rhs.peak_low);
        self.peak_high = self.peak_high.max(rhs.peak_high);
        self.end = rhs.end;
        self.bi_out = rhs.bi_out;
        self.end_bi = rhs.end_bi;
    }

    // 已完备
    pub(crate) fn try_add_to_end(&mut self, item: Handle<T>) -> bool {
        if !self.in_range(item) {
            return false;
        }
        if self.is_one_bi_zs() {
            self.update_zs_range(&[self.begin_bi, item.to_handle()]);
        }
        self.update_zs_end(item.to_handle());
        true
    }

    // 已完备
    pub(crate) fn in_range(&self, item: Handle<T>) -> bool {
        has_overlap(self.low, self.high, item.low(), item.high(), false)
    }

    // 已完备
    pub fn is_inside(&self, seg: &CSeg<T>) -> bool {
        seg.start_bi.index() <= self.begin_bi.index() && self.begin_bi.index() <= seg.end_bi.index()
    }

    // 已完备
    pub fn is_divergence(&self, config: &CPointConfig, out_bi: Option<&T>) -> (bool, Option<f64>) {
        // 最后一笔必须突破中枢
        if !self.end_bi_break(out_bi) {
            return (false, None);
        }

        let in_metric = self.get_bi_in().cal_macd_metric(&config.macd_algo, false);

        let out_metric = if let Some(out_bi) = out_bi {
            out_bi.cal_macd_metric(&config.macd_algo, true)
        } else {
            self.get_bi_out().cal_macd_metric(&config.macd_algo, true)
        };

        if config.divergence_rate > 100.0 {
            // 保送
            (true, Some(out_metric / in_metric))
        } else {
            (
                out_metric <= config.divergence_rate * in_metric,
                Some(out_metric / in_metric),
            )
        }
    }

    // TODO:zs 有deepcopy，这里需要特别注意
    /*pub fn init_from_zs(&mut self, zs: &CZs<T>) {
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
    }*/

    // 已完备
    fn end_bi_break(&self, end_bi: Option<&T>) -> bool {
        let end_bi = end_bi.unwrap_or_else(|| self.get_bi_out());

        (end_bi.is_down() && end_bi.low() < self.low)
            || (end_bi.is_up() && end_bi.high() > self.high)
    }

    // 已完备
    pub fn out_bi_is_peak(&self, end_bi_idx: usize) -> (bool, Option<f64>) {
        //返回 (是否最低点，bi_out与中枢里面尾部最接近它的差距比例)
        assert!(!self.bi_lst.is_empty());

        if let Some(bi_out) = self.bi_out {
            let mut peak_rate = f64::INFINITY;
            for bi in &self.bi_lst {
                if bi.index() > end_bi_idx {
                    break;
                }
                if (bi_out.is_down() && bi.low() < bi_out.low())
                    || (bi_out.is_up() && bi.high() > bi_out.high())
                {
                    return (false, None);
                }
                let r = (bi.get_end_val() - bi_out.get_end_val()).abs() / bi_out.get_end_val();
                if r < peak_rate {
                    peak_rate = r;
                }
            }
            (true, Some(peak_rate))
        } else {
            (false, None)
        }
    }

    // 已完备
    pub fn get_bi_in(&self) -> &T {
        assert!(self.bi_in.is_some());

        self.bi_in.as_ref().unwrap()
    }

    // 已完备
    pub fn get_bi_out(&self) -> &T {
        assert!(self.bi_out.is_some());

        self.bi_out.as_ref().unwrap()
    }

    // 已完备
    pub fn set_bi_in(&mut self, bi: Handle<T>) {
        self.bi_in = Some(bi);
    }

    // 已完备
    pub fn set_bi_out(&mut self, bi: Handle<T>) {
        self.bi_out = Some(bi);
    }

    // 已完备
    pub fn set_bi_lst(&mut self, bi_lst: &[T]) {
        self.bi_lst = bi_lst.iter().map(|x| x.to_handle()).collect();
    }
}

impl<T> CZs<T> {
    // 已完备
    pub fn is_one_bi_zs(&self) -> bool {
        assert!(self.end_bi.is_some());

        self.begin_bi.index() == self.end_bi.unwrap().index()
    }
}

impl<T> std::fmt::Display for CZs<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "index {} is_sure:{} begin:{} begin_bi:{} end:{:?} end_bi:{:?} high:{} low{} peak_high:{} peak_low:{} bi_in:{:?} bi_out:{:?}",
            self.as_handle().index(),
            self.is_sure,
            self.begin.index(),
            self.begin_bi.index(),
            self.end.map(|x| x.index()),
            self.end_bi.map(|x| x.index()),
            self.high,
            self.low,
            self.peak_high,
            self.peak_low,
            self.bi_in.map(|x| x.index()),
            self.bi_out.map(|x| x.index())
        )
    }
}

impl_handle!(CZs<T>);
