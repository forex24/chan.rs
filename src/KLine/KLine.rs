use crate::Common::func_util::has_overlap;
use crate::Common::types::Handle;
use crate::Common::CEnum::{FxCheckMethod, FxType, KlineDir};
use crate::Common::CTime::CTime;
use crate::Common::ChanException::{CChanException, ErrCode};
use crate::KLine::KLine_Unit::CKLineUnit;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct CKLine {
    pub idx: i32,
    pub kl_type: Option<String>,
    pub fx: FxType,
    pub time_begin: CTime,
    pub time_end: CTime,
    pub low: f64,
    pub high: f64,
    pub dir: KlineDir,
    pub lst: Vec<Handle<CKLineUnit>>,
    pub pre: Option<Handle<CKLine>>,
    pub next: Option<Handle<CKLine>>,
    memoize_cache: HashMap<String, f64>,
}

impl CKLine {
    pub fn new(kl_unit: Handle<CKLineUnit>, idx: i32, dir: KlineDir) -> Handle<Self> {
        let kline = Rc::new(RefCell::new(CKLine {
            idx,
            kl_type: kl_unit.borrow().kl_type.clone(),
            fx: FxType::Unknown,
            time_begin: kl_unit.borrow().time.clone(),
            time_end: kl_unit.borrow().time.clone(),
            low: kl_unit.borrow().low,
            high: kl_unit.borrow().high,
            dir,
            lst: vec![Rc::clone(&kl_unit)],
            pre: None,
            next: None,
            memoize_cache: HashMap::new(),
        }));

        kl_unit.borrow_mut().set_klc(Rc::clone(&kline));
        kline
    }

    //pub fn get_sub_klc(&self) -> impl Iterator<Item = Handle<CKLine>> + '_ {
    //    let mut last_klc = None;
    //    self.lst.iter().flat_map(move |klu| {
    //        klu.borrow().get_children().filter_map(move |sub_klu| {
    //            let sub_klc = sub_klu.borrow().get_klc();
    //            if sub_klc != last_klc {
    //                last_klc = sub_klc.clone();
    //                sub_klc
    //            } else {
    //                None
    //            }
    //        })
    //    })
    //}

    pub fn get_klu_max_high(&self) -> f64 {
        self.lst
            .iter()
            .map(|x| x.borrow().high)
            .fold(f64::NEG_INFINITY, f64::max)
    }

    pub fn get_klu_min_low(&self) -> f64 {
        self.lst
            .iter()
            .map(|x| x.borrow().low)
            .fold(f64::INFINITY, f64::min)
    }

    pub fn has_gap_with_next(&self) -> bool {
        if let Some(next) = &self.next {
            let next = next.borrow();
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
            || item2.borrow().pre.is_none()
            || item2.borrow().idx <= self.idx
        {
            return Err(CChanException::new(
                "Invalid kline sequence".to_string(),
                ErrCode::BiErr,
            ));
        }

        match self.fx {
            FxType::Top => {
                if !for_virtual && item2.borrow().fx != FxType::Bottom {
                    return Err(CChanException::new(
                        "Invalid fx sequence".to_string(),
                        ErrCode::BiErr,
                    ));
                }
                if for_virtual && item2.borrow().dir != KlineDir::Down {
                    return Ok(false);
                }

                let (item2_high, self_low) = match method {
                    FxCheckMethod::Half => (
                        item2
                            .borrow()
                            .pre
                            .as_ref()
                            .unwrap()
                            .borrow()
                            .high
                            .max(item2.borrow().high),
                        self.low.min(self.next.as_ref().unwrap().borrow().low),
                    ),
                    FxCheckMethod::Loss => (item2.borrow().high, self.low),
                    FxCheckMethod::Strict | FxCheckMethod::Totally => {
                        let item2_high = if for_virtual {
                            item2
                                .borrow()
                                .pre
                                .as_ref()
                                .unwrap()
                                .borrow()
                                .high
                                .max(item2.borrow().high)
                        } else {
                            if item2.borrow().next.is_none() {
                                return Err(CChanException::new(
                                    "Invalid kline sequence".to_string(),
                                    ErrCode::BiErr,
                                ));
                            }
                            item2
                                .borrow()
                                .pre
                                .as_ref()
                                .unwrap()
                                .borrow()
                                .high
                                .max(item2.borrow().high)
                                .max(item2.borrow().next.as_ref().unwrap().borrow().high)
                        };
                        let self_low = self
                            .pre
                            .as_ref()
                            .unwrap()
                            .borrow()
                            .low
                            .min(self.low)
                            .min(self.next.as_ref().unwrap().borrow().low);
                        (item2_high, self_low)
                    }
                    _ => {
                        return Err(CChanException::new(
                            "bi_fx_check config error!".to_string(),
                            ErrCode::ConfigError,
                        ))
                    }
                };

                Ok(if method == FxCheckMethod::Totally {
                    self.low > item2_high
                } else {
                    self.high > item2_high && item2.borrow().low < self_low
                })
            }
            FxType::Bottom => {
                if !for_virtual && item2.borrow().fx != FxType::Top {
                    return Err(CChanException::new(
                        "Invalid fx sequence".to_string(),
                        ErrCode::BiErr,
                    ));
                }
                if for_virtual && item2.borrow().dir != KlineDir::Up {
                    return Ok(false);
                }

                let (item2_low, cur_high) = match method {
                    FxCheckMethod::Half => (
                        item2
                            .borrow()
                            .pre
                            .as_ref()
                            .unwrap()
                            .borrow()
                            .low
                            .min(item2.borrow().low),
                        self.high.max(self.next.as_ref().unwrap().borrow().high),
                    ),
                    FxCheckMethod::Loss => (item2.borrow().low, self.high),
                    FxCheckMethod::Strict | FxCheckMethod::Totally => {
                        let item2_low = if for_virtual {
                            item2
                                .borrow()
                                .pre
                                .as_ref()
                                .unwrap()
                                .borrow()
                                .low
                                .min(item2.borrow().low)
                        } else {
                            if item2.borrow().next.is_none() {
                                return Err(CChanException::new(
                                    "Invalid kline sequence".to_string(),
                                    ErrCode::BiErr,
                                ));
                            }
                            item2
                                .borrow()
                                .pre
                                .as_ref()
                                .unwrap()
                                .borrow()
                                .low
                                .min(item2.borrow().low)
                                .min(item2.borrow().next.as_ref().unwrap().borrow().low)
                        };
                        let cur_high = self
                            .pre
                            .as_ref()
                            .unwrap()
                            .borrow()
                            .high
                            .max(self.high)
                            .max(self.next.as_ref().unwrap().borrow().high);
                        (item2_low, cur_high)
                    }
                    _ => {
                        return Err(CChanException::new(
                            "bi_fx_check config error!".to_string(),
                            ErrCode::ConfigError,
                        ))
                    }
                };

                Ok(if method == FxCheckMethod::Totally {
                    self.high < item2_low
                } else {
                    self.low < item2_low && item2.borrow().high > cur_high
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

    pub fn test_combine(&self, item: &Handle<CKLineUnit>) -> KlineDir {
        if (self.high >= item.borrow().high && self.low <= item.borrow().low)
            || (self.high <= item.borrow().high && self.low >= item.borrow().low)
        {
            return KlineDir::Combine;
        }

        if self.high > item.borrow().high && self.low > item.borrow().low {
            return KlineDir::Down;
        }
        if self.high < item.borrow().high && self.low < item.borrow().low {
            return KlineDir::Up;
        }

        unreachable!();
    }

    pub fn add(&mut self, unit_kl: Handle<CKLineUnit>) {
        self.lst.push(unit_kl);
    }

    pub fn set_fx(&mut self, fx: FxType) {
        self.fx = fx;
    }

    pub fn try_add(
        klc: &Handle<CKLine>,
        unit_kl: &Handle<CKLineUnit>,
    ) -> Result<KlineDir, CChanException> {
        //let combine_item = CCombineItem::new(unit_kl.clone())?;
        let dir = klc.borrow().test_combine(&unit_kl); //, exclude_included, allow_top_equal)?;
        if dir == KlineDir::Combine {
            klc.borrow_mut().lst.push(Rc::clone(unit_kl));
            //if let Ok(kline_unit) = unit_kl.try_borrow_mut()
            //.unwrap()
            //.downcast_mut::<CKLineUnit>()
            //{
            unit_kl.borrow_mut().set_klc(Rc::clone(klc));
            //}

            let dir_ = klc.borrow().dir;
            match dir_ {
                KlineDir::Up => {
                    if unit_kl.borrow().high != unit_kl.borrow().low
                        || unit_kl.borrow().high != klc.borrow().high
                    {
                        let high_ = klc.borrow().high.max(unit_kl.borrow().high);
                        let low_ = klc.borrow().low.max(unit_kl.borrow().low);
                        klc.borrow_mut().high = high_;
                        klc.borrow_mut().low = low_;
                    }
                }
                KlineDir::Down => {
                    if unit_kl.borrow().high != unit_kl.borrow().low
                        || unit_kl.borrow().low != klc.borrow().low
                    {
                        let high_ = klc.borrow().high.min(unit_kl.borrow().high);
                        let low_ = klc.borrow().low.min(unit_kl.borrow().low);
                        klc.borrow_mut().high = high_;
                        klc.borrow_mut().low = low_;
                    }
                }
                _ => {
                    return Err(CChanException::new(
                        format!(
                            "KlineDir = {:?} err!!! must be {:?}/{:?}",
                            klc.borrow().dir,
                            KlineDir::Up,
                            KlineDir::Down
                        ),
                        ErrCode::CombinerErr,
                    ))
                }
            }
            klc.borrow_mut().time_end = unit_kl.borrow().time;
            //self.clean_cache();
        }
        Ok(dir)
    }

    pub fn get_peak_klu(&self, is_high: bool) -> Option<Handle<CKLineUnit>> {
        if is_high {
            self.get_high_peak_klu()
        } else {
            self.get_low_peak_klu()
        }
    }

    pub fn get_high_peak_klu(&self) -> Option<Handle<CKLineUnit>> {
        //if let Some(cached) = self.memoize_cache.get("high_peak") {
        //    return Ok(cached.clone());
        //}
        for kl in self.lst.iter().rev() {
            if kl.borrow().high == self.high {
                //self.memoize_cache
                //    .insert("high_peak".to_string(), kl.clone());
                return Some(Rc::clone(kl));
            }
        }
        None
        //Err(CChanException::new(
        //    "can't find peak...".to_string(),
        //    ErrCode::CombinerErr,
        //))
    }

    pub fn get_low_peak_klu(&self) -> Option<Handle<CKLineUnit>> {
        //if let Some(cached) = self.memoize_cache.get("low_peak") {
        //    return Ok(cached.clone());
        //}
        for kl in self.lst.iter().rev() {
            if kl.borrow().low == self.low {
                //self.memoize_cache
                //    .insert("low_peak".to_string(), kl.clone());
                return Some(Rc::clone(kl));
            }
        }
        None
        //Err(CChanException::new(
        //    "can't find peak...".to_string(),
        //    ErrCode::CombinerErr,
        //))
    }

    pub fn update_fx(cur: &Handle<CKLine>, pre: &Handle<CKLine>, next: &Handle<CKLine>) {
        cur.borrow_mut().set_next(next.clone());
        cur.borrow_mut().set_pre(pre.clone());
        next.borrow_mut().set_pre(cur.clone());

        let pre = pre.borrow();
        let next = next.borrow();
        let mut cur = cur.borrow_mut();
        if pre.high < cur.high && next.high < cur.high && pre.low < cur.low && next.low < cur.low {
            cur.fx = FxType::Top;
        } else if pre.high > cur.high
            && next.high > cur.high
            && pre.low > cur.low
            && next.low > cur.low
        {
            cur.fx = FxType::Bottom;
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
            self.idx,
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
