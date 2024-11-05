use crate::impl_handle;
use crate::Common::func_util::has_overlap;
use crate::Common::handle::{AsHandle, Handle, Indexable};
use crate::Common::CEnum::{FxCheckMethod, FxType, KLineDir};
use crate::Common::CTime::CTime;
use crate::Common::ChanException::{CChanException, ErrCode};
use crate::KLine::KLine_Unit::CKLineUnit;

// 合并后的K线
pub struct CKLine {
    handle: Handle<Self>,
    pub kl_type: Option<String>,
    pub fx: FxType,
    pub time_begin: CTime,
    pub time_end: CTime,
    pub low: f64,
    pub high: f64,
    pub dir: KLineDir,
    pub lst: Vec<Handle<CKLineUnit>>,
    pub pre: Option<Handle<CKLine>>,
    pub next: Option<Handle<CKLine>>,
}

impl CKLine {
    pub fn new(
        box_vec: &Box<Vec<Self>>,
        kl_unit: Handle<CKLineUnit>,
        idx: usize,
        dir: KLineDir, /*缺省值为KLINE_DIR.UP*/
    ) -> Self {
        let kline = CKLine {
            handle: Handle::new(box_vec, idx),
            kl_type: kl_unit.kl_type.clone(),
            fx: FxType::Unknown,
            time_begin: kl_unit.time,
            time_end: kl_unit.time,
            low: kl_unit.low,
            high: kl_unit.high,
            dir,
            lst: vec![kl_unit],
            pre: None,
            next: None,
        };

        kl_unit.as_mut().set_klc(kline.as_handle());
        kline
    }

    //pub fn get_sub_klc(&self) -> impl Iterator<Item = &Handle<CKLine>> + '_ {
    //    let mut last_klc = None;
    //    self.lst.iter().flat_map(move |klu| {
    //        klu.get_children().filter_map(move |sub_klu| {
    //            let sub_klc = sub_klu.get_klc();
    //            if last_klc.as_ref() != Some(&sub_klc) {
    //                last_klc = Some(sub_klc.clone());
    //                Some(&sub_klc)
    //            } else {
    //                None
    //            }
    //        })
    //    })
    //}

    pub fn get_klu_max_high(&self) -> f64 {
        self.lst
            .iter()
            .map(|x| x.high)
            .fold(f64::NEG_INFINITY, f64::max)
    }

    pub fn get_klu_min_low(&self) -> f64 {
        self.lst.iter().map(|x| x.low).fold(f64::INFINITY, f64::min)
    }

    pub fn has_gap_with_next(&self) -> bool {
        if let Some(next) = &self.next {
            let next = next;
            !has_overlap(
                self.get_klu_min_low(),
                self.get_klu_max_high(),
                next.get_klu_min_low(),
                next.get_klu_max_high(),
                true,
            )
        } else {
            false
        }
    }

    pub fn check_fx_valid(
        &self,
        item2: &Handle<CKLine>,
        method: FxCheckMethod,
        for_virtual: bool,
    ) -> Result<bool, CChanException> {
        if self.next.is_none()
            || self.pre.is_none()
            || item2.pre.is_none()
            || item2.idx <= self.index()
        {
            return Err(CChanException::new(
                "Invalid kline sequence".to_string(),
                ErrCode::BiErr,
            ));
        }

        match self.fx {
            FxType::Top => {
                if !for_virtual && item2.fx != FxType::Bottom {
                    return Err(CChanException::new(
                        "Invalid fx sequence".to_string(),
                        ErrCode::BiErr,
                    ));
                }
                if for_virtual && item2.dir != KLineDir::Down {
                    return Ok(false);
                }

                let (item2_high, self_low) = match method {
                    FxCheckMethod::Half => (
                        item2.pre.as_ref().unwrap().high.max(item2.high),
                        self.low.min(self.next.as_ref().unwrap().low),
                    ),
                    FxCheckMethod::Loss => (item2.high, self.low),
                    FxCheckMethod::Strict | FxCheckMethod::Totally => {
                        let item2_high = if for_virtual {
                            item2.pre.as_ref().unwrap().high.max(item2.high)
                        } else {
                            if item2.next.is_none() {
                                return Err(CChanException::new(
                                    "Invalid kline sequence".to_string(),
                                    ErrCode::BiErr,
                                ));
                            }
                            item2
                                .pre
                                .as_ref()
                                .unwrap()
                                .high
                                .max(item2.high)
                                .max(item2.next.as_ref().unwrap().high)
                        };
                        let self_low = self
                            .pre
                            .as_ref()
                            .unwrap()
                            .low
                            .min(self.low)
                            .min(self.next.as_ref().unwrap().low);
                        (item2_high, self_low)
                    }
                };

                Ok(if method == FxCheckMethod::Totally {
                    self.low > item2_high
                } else {
                    self.high > item2_high && item2.low < self_low
                })
            }
            FxType::Bottom => {
                if !for_virtual && item2.fx != FxType::Top {
                    return Err(CChanException::new(
                        "Invalid fx sequence".to_string(),
                        ErrCode::BiErr,
                    ));
                }
                if for_virtual && item2.dir != KLineDir::Up {
                    return Ok(false);
                }

                let (item2_low, cur_high) = match method {
                    FxCheckMethod::Half => (
                        item2.pre.as_ref().unwrap().low.min(item2.low),
                        self.high.max(self.next.as_ref().unwrap().high),
                    ),
                    FxCheckMethod::Loss => (item2.low, self.high),
                    FxCheckMethod::Strict | FxCheckMethod::Totally => {
                        let item2_low = if for_virtual {
                            item2.pre.as_ref().unwrap().low.min(item2.low)
                        } else {
                            if item2.next.is_none() {
                                return Err(CChanException::new(
                                    "Invalid kline sequence".to_string(),
                                    ErrCode::BiErr,
                                ));
                            }
                            item2
                                .pre
                                .as_ref()
                                .unwrap()
                                .low
                                .min(item2.low)
                                .min(item2.next.as_ref().unwrap().low)
                        };
                        let cur_high = self
                            .pre
                            .as_ref()
                            .unwrap()
                            .high
                            .max(self.high)
                            .max(self.next.as_ref().unwrap().high);
                        (item2_low, cur_high)
                    }
                };

                Ok(if method == FxCheckMethod::Totally {
                    self.high < item2_low
                } else {
                    self.low < item2_low && item2.high > cur_high
                })
            }
            _ => Err(CChanException::new(
                "only top/bottom fx can check_valid_top_button".to_string(),
                ErrCode::BiErr,
            )),
        }
    }
}

impl CKLine {
    pub fn pre(&self) -> Result<Handle<CKLine>, CChanException> {
        self.pre.clone().ok_or_else(|| {
            CChanException::new("No previous combiner".to_string(), ErrCode::CombinerErr)
        })
    }

    pub fn next(&self) -> Option<Handle<CKLine>> {
        self.next.clone()
    }

    pub fn get_next(&self) -> Option<Handle<CKLine>> {
        debug_assert!(self.next.is_some());
        //self.next.clone().ok_or_else(|| {
        //    CChanException::new("No next combiner".to_string(), ErrCode::CombinerErr)
        //})
        self.next()
    }

    pub fn test_combine(&self, item: &Handle<CKLineUnit>) -> KLineDir {
        if (self.high >= item.high && self.low <= item.low)
            || (self.high <= item.high && self.low >= item.low)
        {
            return KLineDir::Combine;
        }

        if self.high > item.high && self.low > item.low {
            return KLineDir::Down;
        }
        if self.high < item.high && self.low < item.low {
            return KLineDir::Up;
        }

        unreachable!();
    }

    pub fn add(&mut self, unit_kl: Handle<CKLineUnit>) {
        self.lst.push(unit_kl);
    }

    pub fn set_fx(&mut self, fx: FxType) {
        self.fx = fx;
    }

    pub fn try_add(&mut self, unit_kl: Handle<CKLineUnit>) -> KLineDir {
        let dir = self.test_combine(&unit_kl);
        if dir == KLineDir::Combine {
            self.lst.push(unit_kl);
            unit_kl.as_mut().set_klc(self.as_handle());

            let dir_ = self.dir;
            match dir_ {
                KLineDir::Up => {
                    if unit_kl.high != unit_kl.low || unit_kl.high != self.high {
                        let high_ = self.high.max(unit_kl.high);
                        let low_ = self.low.max(unit_kl.low);
                        self.high = high_;
                        self.low = low_;
                    }
                }
                KLineDir::Down => {
                    if unit_kl.high != unit_kl.low || unit_kl.low != self.low {
                        let high_ = self.high.min(unit_kl.high);
                        let low_ = self.low.min(unit_kl.low);
                        self.high = high_;
                        self.low = low_;
                    }
                }
                _ => {
                    panic!("KlineDir {} err!!! must be Up/Down", dir);
                }
            }
            self.time_end = unit_kl.time;
        }
        dir
    }

    pub fn get_peak_klu(&self, is_high: bool) -> Option<&Handle<CKLineUnit>> {
        if is_high {
            self.get_high_peak_klu()
        } else {
            self.get_low_peak_klu()
        }
    }

    pub fn get_high_peak_klu(&self) -> Option<&Handle<CKLineUnit>> {
        //if let Some(cached) = self.memoize_cache.get("high_peak") {
        //    return Ok(cached.clone());
        //}
        for kl in self.lst.iter().rev() {
            if kl.high == self.high {
                //self.memoize_cache
                //    .insert("high_peak".to_string(), kl.clone());
                return Some(kl);
            }
        }
        None
        //Err(CChanException::new(
        //    "can't find peak...".to_string(),
        //    ErrCode::CombinerErr,
        //))
    }

    pub fn get_low_peak_klu(&self) -> Option<&Handle<CKLineUnit>> {
        //if let Some(cached) = self.memoize_cache.get("low_peak") {
        //    return Ok(cached.clone());
        //}
        for kl in self.lst.iter().rev() {
            if kl.low == self.low {
                //self.memoize_cache
                //    .insert("low_peak".to_string(), kl.clone());
                return Some(kl);
            }
        }
        None
        //Err(CChanException::new(
        //    "can't find peak...".to_string(),
        //    ErrCode::CombinerErr,
        //))
    }

    pub fn update_fx(cur: &Handle<CKLine>, pre: &Handle<CKLine>, next: &Handle<CKLine>) {
        cur.as_mut().set_next(next.clone());
        cur.as_mut().set_pre(pre.clone());
        next.as_mut().set_pre(cur.clone());

        let pre = pre;
        let next = next;
        let cur = cur;
        if pre.high < cur.high && next.high < cur.high && pre.low < cur.low && next.low < cur.low {
            cur.as_mut().fx = FxType::Top;
        } else if pre.high > cur.high
            && next.high > cur.high
            && pre.low > cur.low
            && next.low > cur.low
        {
            cur.as_mut().fx = FxType::Bottom;
        }
        //self.clean_cache();
    }

    pub fn set_pre(&mut self, pre: Handle<CKLine>) {
        self.pre = Some(pre);
        //self.clean_cache();
    }

    pub fn set_next(&mut self, next: Handle<CKLine>) {
        self.next = Some(next);
        //self.clean_cache();
    }
}

impl std::fmt::Display for CKLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fx_token = match self.fx {
            FxType::Top => "^",
            FxType::Bottom => "_",
            FxType::Unknown => "",
        };
        write!(
            f,
            "{}th{}:{}~{}({}|{}) low={} high={}",
            self.index(),
            fx_token,
            self.time_begin,
            self.time_end,
            self.kl_type.as_deref().unwrap_or(""),
            self.lst.len(),
            self.low,
            self.high
        )
    }
}

// Add these implementations after the CKLine struct definition
impl PartialEq for CKLine {
    fn eq(&self, other: &Self) -> bool {
        self.index() == other.index()
    }
}

impl Eq for CKLine {}

impl PartialOrd for CKLine {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.index().partial_cmp(&other.index())
    }
}

impl Ord for CKLine {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.index().cmp(&other.index())
    }
}

impl_handle!(CKLine);
