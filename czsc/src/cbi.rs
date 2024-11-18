use std::cell::RefCell;

use crate::{
    AsHandle, Bar, CBspPoint, Candle, Handle, IBspInfo, ICalcMetric, IHighLow, IParent, LineType,
    ToHandle,
};
use crate::{BiType, Direction, FxType, MacdAlgo};

// 笔
#[derive(Debug)]
pub struct CBi {
    handle: Handle<Self>,
    pub begin_klc: Handle<Candle>,
    pub end_klc: Handle<Candle>,
    pub is_sure: bool,
    pub dir: Direction,
    pub bi_type: BiType,
    pub sure_end: Vec<Handle<Candle>>,
    pub klc_lst: Vec<Handle<Candle>>,
    pub seg_idx: Option<usize>,
    pub parent_seg_idx: Option<usize>, // 在哪个线段里面
    pub parent_seg_dir: Option<Direction>,
    pub bsp: Option<Handle<CBspPoint<CBi>>>, // 尾部是不是买卖点
    //pub next: Option<Handle<CBi>>,
    //pub pre: Option<Handle<CBi>>,
    // 缓存相关字段
    cached_begin_klu: RefCell<Option<Handle<Bar>>>,
    cached_end_klu: RefCell<Option<Handle<Bar>>>,
}

impl CBi {
    #[allow(clippy::borrowed_box)]
    pub fn new(
        boxed_vec: &Box<Vec<Self>>,
        begin_klc: Handle<Candle>,
        end_klc: Handle<Candle>,
        idx: usize,
        is_sure: bool,
    ) -> Self {
        let dir = match begin_klc.fx_type {
            FxType::Bottom => Direction::Up,
            FxType::Top => Direction::Down,
            _ => panic!("ERROR DIRECTION when creating bi"),
        };
        let bi = Self {
            handle: Handle::new(boxed_vec, idx),
            begin_klc,
            end_klc,
            is_sure,
            dir,
            bi_type: BiType::Strict,
            sure_end: Vec::new(),
            klc_lst: Vec::with_capacity(1024),
            seg_idx: None,
            parent_seg_idx: None,
            parent_seg_dir: None,
            bsp: None,
            //next: None,
            //pre: None,
            // Initialize cache fields
            cached_begin_klu: RefCell::new(None),
            cached_end_klu: RefCell::new(None),
        };
        bi.check();
        bi
    }

    /// Clean all cached values
    pub fn clean_cache(&mut self) {
        self.cached_begin_klu = RefCell::new(None);
        self.cached_end_klu = RefCell::new(None);
    }

    pub fn set_seg_idx(&mut self, idx: usize) {
        self.seg_idx = Some(idx);
    }

    // 已完备
    fn check(&self) {
        match self._is_down() {
            true => debug_assert!(self.begin_klc.high > self.end_klc.low),
            false => debug_assert!(self.begin_klc.low < self.end_klc.high),
        }
    }

    // 已完备
    pub fn _get_begin_val(&self) -> f64 {
        match self._is_up() {
            true => self.begin_klc.low,
            false => self.begin_klc.high,
        }
    }

    // 已完备
    pub fn _get_end_val(&self) -> f64 {
        match self._is_up() {
            true => self.end_klc.high,
            false => self.end_klc.low,
        }
    }

    // 已完备
    pub fn _get_begin_klu(&self) -> Handle<Bar> {
        if let Some(klu) = self.cached_begin_klu.borrow().as_ref() {
            return *klu;
        }
        let bar = match self._is_up() {
            true => self.begin_klc.get_peak_klu(false),
            false => self.begin_klc.get_peak_klu(true),
        };
        self.cached_begin_klu.borrow_mut().replace(bar.as_handle());
        bar.as_handle()
    }

    // 已完备
    // TODO: 性能热点
    pub fn _get_end_klu(&self) -> Handle<Bar> {
        if let Some(bar) = self.cached_end_klu.borrow().as_ref() {
            return *bar;
        }
        let bar = match self._is_up() {
            true => self.end_klc.get_peak_klu(true),
            false => self.end_klc.get_peak_klu(false),
        };
        self.cached_end_klu.borrow_mut().replace(bar.as_handle());
        bar.as_handle()
    }

    // 已完备
    pub fn _amp(&self) -> f64 {
        (self._get_end_val() - self._get_begin_val()).abs()
    }

    // 已完备
    pub fn _get_klu_cnt(&self) -> usize {
        self._get_end_klu().as_handle().index() - self._get_begin_klu().as_handle().index() + 1
    }

    // 已完备
    pub fn _get_klc_cnt(&self) -> usize {
        assert_eq!(
            self.end_klc.index(),
            self._get_end_klu().klc.unwrap().index()
        );
        assert_eq!(
            self.begin_klc.as_handle().index(),
            self._get_begin_klu().klc.unwrap().index()
        );

        self.end_klc.index() - self.begin_klc.index() + 1
    }

    // 已完备
    pub fn _high(&self) -> f64 {
        match self._is_up() {
            true => self.end_klc.high,
            false => self.begin_klc.high,
        }
    }

    // 已完备
    pub fn _low(&self) -> f64 {
        match self._is_up() {
            true => self.begin_klc.low,
            false => self.end_klc.low,
        }
    }

    // 已完备
    pub fn _is_down(&self) -> bool {
        self.dir == Direction::Down
    }

    // 已完备
    pub fn _is_up(&self) -> bool {
        self.dir == Direction::Up
    }

    pub fn update_virtual_end(&mut self, new_klc: Handle<Candle>) {
        self.append_sure_end(self.end_klc);
        self.update_new_end(new_klc);
        self.is_sure = false;
    }

    pub fn restore_from_virtual_end(&mut self, sure_end: Handle<Candle>) {
        self.is_sure = true;
        self.update_new_end(sure_end);
        self.sure_end.clear();
    }

    /// Append sure end point
    pub fn append_sure_end(&mut self, klc: Handle<Candle>) {
        self.sure_end.push(klc);
    }

    /// Update new end point
    pub fn update_new_end(&mut self, new_klc: Handle<Candle>) {
        self.end_klc = new_klc;
        self.check();
        self.clean_cache();
    }

    // 99% 完备
    fn _cal_macd_metric(&self, macd_algo: &MacdAlgo, is_reverse: bool) -> f64 {
        match macd_algo {
            MacdAlgo::Area => self.cal_macd_half(is_reverse),
            MacdAlgo::Peak => self.cal_macd_peak(),
            MacdAlgo::FullArea => self.cal_macd_area(),
            MacdAlgo::Diff => self.cal_macd_diff(),
            MacdAlgo::Slope => self.cal_macd_slope(),
            MacdAlgo::Amp => self.cal_macd_amp(),
            // MacdAlgo::Amount => self.cal_macd_trade_metric(DataField::Turnover, false),
            // MacdAlgo::Volume => self.cal_macd_trade_metric(DataField::Volume, false),
            // MacdAlgo::VolumeAvg => self.cal_macd_trade_metric(DataField::Volume, true),
            // MacdAlgo::AmountAvg => self.cal_macd_trade_metric(DataField::Turnover, true),
            // MacdAlgo::TurnrateAvg => self.cal_macd_trade_metric(DataField::Turnrate, true),
            //MacdAlgo::Rsi => self.cal_rsi(),
        }
    }

    /*pub fn cal_rsi(&self) -> f64 {
        let mut rsi_lst = Vec::new();
        for klc in &self.klc_lst {
            rsi_lst.extend(klc.lst.iter().map(|klu| klu.rsi));
        }
        if self.is_down() {
            10000.0 / (rsi_lst.iter().min().unwrap() + 1e-7)
        } else {
            rsi_lst.iter().max().unwrap()
        }
    }*/

    fn cal_macd_area(&self) -> f64 {
        let mut s = 1e-7;
        let begin_klu = self.get_begin_klu();
        let end_klu = self.get_end_klu();

        for klc in &self.klc_lst {
            for klu in &klc.lst {
                //s += klu.macd.unwrap().macd.abs();
                if klu.index() < begin_klu.index() || klu.index() > end_klu.index() {
                    continue;
                }
                if (self.is_down() && klu.macd.unwrap().macd < 0.0)
                    || (self.is_up() && klu.macd.unwrap().macd > 0.0)
                {
                    s += klu.macd.unwrap().macd.abs()
                }
            }
        }
        s
    }

    // TODO:
    #[allow(clippy::if_same_then_else)]
    fn cal_macd_peak(&self) -> f64 {
        let mut peak = 1e-7;
        for klc in &self.klc_lst {
            for klu in &klc.lst {
                if klu.macd.unwrap().macd.abs() > peak {
                    if self.is_down() && klu.macd.unwrap().macd < 0.0 {
                        peak = klu.macd.unwrap().macd.abs();
                    } else if self.is_up() && klu.macd.unwrap().macd > 0.0 {
                        peak = klu.macd.unwrap().macd.abs();
                    }
                }
            }
        }
        peak
    }

    // 已完备
    fn cal_macd_half(&self, is_reverse: bool) -> f64 {
        if is_reverse {
            self.cal_macd_half_reverse()
        } else {
            self.cal_macd_half_obverse()
        }
    }

    fn cal_macd_half_obverse(&self) -> f64 {
        let mut s = 1e-7;
        let begin_klu = self.get_begin_klu();
        let peak_macd = begin_klu.macd.unwrap().macd;
        for klc in &self.klc_lst {
            for klu in &klc.lst {
                if klu.index() < begin_klu.index() {
                    continue;
                }
                if klu.macd.unwrap().macd * peak_macd > 0.0 {
                    s += klu.macd.unwrap().macd.abs();
                } else {
                    break;
                }
            }
        }
        s
    }

    fn cal_macd_half_reverse(&self) -> f64 {
        let mut s = 1e-7;
        let begin_klu = self.get_end_klu();
        let peak_macd = begin_klu.macd.unwrap().macd;
        for klc in self.klc_lst.iter().rev() {
            for klu in klc.lst.iter().rev() {
                if klu.index() > begin_klu.index() {
                    continue;
                }
                if klu.macd.unwrap().macd * peak_macd > 0.0 {
                    s += klu.macd.unwrap().macd.abs();
                } else {
                    break;
                }
            }
        }
        s
    }

    // macd红绿柱最大值最小值之差
    fn cal_macd_diff(&self) -> f64 {
        let mut max_ = f64::NEG_INFINITY;
        let mut min_ = f64::INFINITY;
        for klc in &self.klc_lst {
            for klu in &klc.lst {
                let macd = klu.macd.unwrap().macd;
                if macd > max_ {
                    max_ = macd;
                }
                if macd < min_ {
                    min_ = macd;
                }
            }
        }
        max_ - min_
    }

    fn cal_macd_slope(&self) -> f64 {
        let begin_klu = self.get_begin_klu();
        let end_klu = self.get_end_klu();
        if self.is_up() {
            (end_klu.high - begin_klu.low)
                / end_klu.high
                / (end_klu.index() - begin_klu.index() + 1) as f64
        } else {
            (begin_klu.high - end_klu.low)
                / begin_klu.high
                / (end_klu.index() - begin_klu.index() + 1) as f64
        }
    }

    fn cal_macd_amp(&self) -> f64 {
        let begin_klu = self.get_begin_klu();
        let end_klu = self.get_end_klu();
        if self.is_down() {
            (begin_klu.high - end_klu.low) / begin_klu.high
        } else {
            (end_klu.high - begin_klu.low) / begin_klu.low
        }
    }

    /*pub fn cal_macd_trade_metric(&self, metric: DataField, cal_avg: bool) -> f64 {
        let mut s = 0.0;
        for klc in &self.klc_lst {
            for klu in &klc.lst {
                let metric_res = klu.trade_info.metric.get(&metric);
                if metric_res.is_none() {
                    return 0.0;
                }
                s += metric_res.unwrap();
            }
        }
        if cal_avg {
            s / self.get_klu_cnt() as f64
        } else {
            s
        }
    }*/

    //pub fn set_klc_lst(&mut self, lst: &[Candle]) {
    //    self.klc_lst = lst.iter().map(|x| x.as_handle()).collect();
    //}
}

impl IHighLow for CBi {
    fn high(&self) -> f64 {
        self._high()
    }

    fn low(&self) -> f64 {
        self._low()
    }
}

impl IParent for CBi {
    fn seg_idx(&self) -> Option<usize> {
        self.seg_idx
    }

    fn set_seg_idx(&mut self, idx: usize) {
        self.seg_idx = Some(idx)
    }
    fn set_parent_seg_idx(&mut self, parent_seg_idx: Option<usize>) {
        self.parent_seg_idx = parent_seg_idx;
    }

    fn parent_seg_idx(&self) -> Option<usize> {
        self.parent_seg_idx
    }

    fn set_parent_seg_dir(&mut self, dir: Option<Direction>) {
        self.parent_seg_dir = dir;
    }

    fn parent_seg_dir(&self) -> Option<Direction> {
        self.parent_seg_dir
    }
}

impl IBspInfo for CBi {
    fn set_bsp(&mut self, bsp: Handle<CBspPoint<Self>>) {
        self.bsp = Some(bsp);
    }
}

impl ICalcMetric for CBi {
    fn cal_macd_metric(&self, algo: &MacdAlgo, is_reverse: bool) -> f64 {
        self._cal_macd_metric(algo, is_reverse)
    }
}

impl ToHandle for CBi {
    fn to_handle(&self) -> Handle<Self> {
        self.handle
    }
}

impl LineType for CBi {
    fn get_begin_klu(&self) -> Handle<Bar> {
        self._get_begin_klu().as_handle()
    }

    fn get_end_klu(&self) -> Handle<Bar> {
        self._get_end_klu().as_handle()
    }

    fn get_begin_val(&self) -> f64 {
        self._get_begin_val()
    }

    fn get_end_val(&self) -> f64 {
        self._get_end_val()
    }

    fn direction(&self) -> Direction {
        self.dir
    }

    fn is_up(&self) -> bool {
        self._is_up()
    }

    fn is_down(&self) -> bool {
        self._is_down()
    }

    fn amp(&self) -> f64 {
        self._amp()
    }

    fn is_sure(&self) -> bool {
        self.is_sure
    }
}

impl std::fmt::Display for CBi {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "index {} dir:{} is_sure:{} begin:{} end:{} seg_idx:{:?} p_idx:{:?} p_dir:{:?} bsp_bi:{:?} bsp_klu:{:?}",
            self.as_handle().index(),
            self.dir,
            self.is_sure,
            self.begin_klc.index(),
            self.end_klc.index(),
            self.seg_idx,
            self.parent_seg_idx,
            self.parent_seg_dir,
            self.bsp.as_ref().map(|x| x.bi),
            self.bsp.as_ref().map(|x| x.klu)
        )
    }
}

impl_handle!(CBi);
