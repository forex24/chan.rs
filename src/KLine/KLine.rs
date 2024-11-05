use crate::Common::func_util::has_overlap;
use crate::Common::types::{Handle, StrongHandle, WeakHandle};
use crate::Common::CEnum::{FxCheckMethod, FxType, KLineDir};
use crate::Common::CTime::CTime;
use crate::Common::ChanException::{CChanException, ErrCode};
use crate::KLine::KLine_Unit::CKLineUnit;
use std::cell::RefCell;
use std::rc::Rc;

// 合并后的K线
pub struct CKLine {
    pub idx: usize,
    pub kl_type: Option<String>,
    pub fx: FxType,
    pub time_begin: CTime,
    pub time_end: CTime,
    pub low: f64,
    pub high: f64,
    pub dir: KLineDir,
    pub lst: Vec<StrongHandle<CKLineUnit>>,
    pub pre: Option<WeakHandle<CKLine>>,
    pub next: Option<WeakHandle<CKLine>>,
}

impl CKLine {
    pub fn new(
        kl_unit: StrongHandle<CKLineUnit>,
        idx: usize,
        dir: KLineDir, /*缺省值为KLINE_DIR.UP*/
    ) -> StrongHandle<Self> {
        let kline = Rc::new(RefCell::new(CKLine {
            idx,
            kl_type: kl_unit.borrow().kl_type.clone(),
            fx: FxType::Unknown,
            time_begin: kl_unit.borrow().time,
            time_end: kl_unit.borrow().time,
            low: kl_unit.borrow().low,
            high: kl_unit.borrow().high,
            dir,
            lst: vec![Rc::clone(&kl_unit)],
            pre: None,
            next: None,
        }));
        kline
    }

    //pub fn get_sub_klc(&self) -> impl Iterator<Item = &Handle<CKLine>> + '_ {
    //    let mut last_klc = None;
    //    self.lst.iter().flat_map(move |klu| {
    //        klu.borrow().get_children().filter_map(move |sub_klu| {
    //            let sub_klc = sub_klu.borrow().get_klc();
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
            if let Some(next) = next.upgrade() {
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
        } else {
            false
        }
    }

    pub fn check_fx_valid(
        &self,
        item2: &StrongHandle<CKLine>,
        method: FxCheckMethod,
        for_virtual: bool,
    ) -> bool {
        assert!(self.next.is_some() && self.pre.is_some());
        assert!(item2.borrow().pre.is_some());
        assert!(item2.borrow().idx > self.idx);

        match self.fx {
            FxType::Top => {
                assert!(for_virtual || item2.borrow().fx == FxType::Bottom);
                if for_virtual && item2.borrow().dir != KLineDir::Down {
                    return false;
                }

                let (item2_high, self_low) = match method {
                    FxCheckMethod::Half => (
                        item2
                            .borrow()
                            .pre
                            .as_ref()
                            .unwrap()
                            .upgrade()
                            .unwrap()
                            .borrow()
                            .high
                            .max(item2.borrow().high),
                        self.low
                            .min(self.next.as_ref().unwrap().upgrade().unwrap().borrow().low),
                    ),
                    FxCheckMethod::Loss => (item2.borrow().high, self.low),
                    FxCheckMethod::Strict | FxCheckMethod::Totally => {
                        let item2_high = if for_virtual {
                            item2
                                .borrow()
                                .pre
                                .as_ref()
                                .unwrap()
                                .upgrade()
                                .unwrap()
                                .borrow()
                                .high
                                .max(item2.borrow().high)
                        } else {
                            assert!(item2.borrow().next.is_some());
                            item2
                                .borrow()
                                .pre
                                .as_ref()
                                .unwrap()
                                .upgrade()
                                .unwrap()
                                .borrow()
                                .high
                                .max(item2.borrow().high)
                                .max(
                                    item2
                                        .borrow()
                                        .next
                                        .as_ref()
                                        .unwrap()
                                        .upgrade()
                                        .unwrap()
                                        .borrow()
                                        .high,
                                )
                        };
                        let self_low = self
                            .pre
                            .as_ref()
                            .unwrap()
                            .upgrade()
                            .unwrap()
                            .borrow()
                            .low
                            .min(self.low)
                            .min(self.next.as_ref().unwrap().upgrade().unwrap().borrow().low);
                        (item2_high, self_low)
                    }
                };

                if method == FxCheckMethod::Totally {
                    self.low > item2_high
                } else {
                    self.high > item2_high && item2.borrow().low < self_low
                }
            }
            FxType::Bottom => {
                assert!(for_virtual || item2.borrow().fx == FxType::Top);
                if for_virtual && item2.borrow().dir != KLineDir::Up {
                    return false;
                }

                let (item2_low, cur_high) = match method {
                    FxCheckMethod::Half => (
                        item2
                            .borrow()
                            .pre
                            .as_ref()
                            .unwrap()
                            .upgrade()
                            .unwrap()
                            .borrow()
                            .low
                            .min(item2.borrow().low),
                        self.high
                            .max(self.next.as_ref().unwrap().upgrade().unwrap().borrow().high),
                    ),
                    FxCheckMethod::Loss => (item2.borrow().low, self.high),
                    FxCheckMethod::Strict | FxCheckMethod::Totally => {
                        let item2_low = if for_virtual {
                            item2
                                .borrow()
                                .pre
                                .as_ref()
                                .unwrap()
                                .upgrade()
                                .unwrap()
                                .borrow()
                                .low
                                .min(item2.borrow().low)
                        } else {
                            assert!(item2.borrow().next.is_some());
                            item2
                                .borrow()
                                .pre
                                .as_ref()
                                .unwrap()
                                .upgrade()
                                .unwrap()
                                .borrow()
                                .low
                                .min(item2.borrow().low)
                                .min(
                                    item2
                                        .borrow()
                                        .next
                                        .as_ref()
                                        .unwrap()
                                        .upgrade()
                                        .unwrap()
                                        .borrow()
                                        .low,
                                )
                        };
                        let cur_high = self
                            .pre
                            .as_ref()
                            .unwrap()
                            .upgrade()
                            .unwrap()
                            .borrow()
                            .high
                            .max(self.high)
                            .max(self.next.as_ref().unwrap().upgrade().unwrap().borrow().high);
                        (item2_low, cur_high)
                    }
                };

                if method == FxCheckMethod::Totally {
                    self.high < item2_low
                } else {
                    self.low < item2_low && item2.borrow().high > cur_high
                }
            }
            _ => panic!("only top/bottom fx can check_valid_top_button"),
        }
    }
}

impl CKLine {
    pub fn pre(&self) -> Option<StrongHandle<CKLine>> {
        self.pre.clone().and_then(|weak| weak.upgrade())
    }

    pub fn next(&self) -> Option<StrongHandle<CKLine>> {
        self.next.clone().and_then(|weak| weak.upgrade())
    }

    pub fn get_next(&self) -> Option<StrongHandle<CKLine>> {
        debug_assert!(self.next.is_some());
        self.next()
    }

    pub fn test_combine(&self, item: &Handle<CKLineUnit>) -> KLineDir {
        if (self.high >= item.borrow().high && self.low <= item.borrow().low)
            || (self.high <= item.borrow().high && self.low >= item.borrow().low)
        {
            return KLineDir::Combine;
        }

        if self.high > item.borrow().high && self.low > item.borrow().low {
            return KLineDir::Down;
        }
        if self.high < item.borrow().high && self.low < item.borrow().low {
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

    pub fn try_add(klc: &StrongHandle<CKLine>, bar: &StrongHandle<CKLineUnit>) -> KLineDir {
        let dir = klc.borrow().test_combine(&bar);
        if dir == KLineDir::Combine {
            klc.borrow_mut().lst.push(Rc::clone(bar));

            bar.borrow_mut().set_klc(Rc::downgrade(klc));

            let dir_ = klc.borrow().dir;
            match dir_ {
                KLineDir::Up => {
                    // 一字板不用处理
                    if bar.borrow().high != bar.borrow().low
                        || bar.borrow().high != klc.borrow().high
                    {
                        let high_ = klc.borrow().high.max(bar.borrow().high);
                        let low_ = klc.borrow().low.max(bar.borrow().low);
                        klc.borrow_mut().high = high_;
                        klc.borrow_mut().low = low_;
                    }
                }
                KLineDir::Down => {
                    // 一字板不用处理
                    if bar.borrow().high != bar.borrow().low || bar.borrow().low != klc.borrow().low
                    {
                        let high_ = klc.borrow().high.min(bar.borrow().high);
                        let low_ = klc.borrow().low.min(bar.borrow().low);
                        klc.borrow_mut().high = high_;
                        klc.borrow_mut().low = low_;
                    }
                }
                _ => {
                    panic!("KlineDir {} err!!! must be Up/Down", dir);
                }
            }
            klc.borrow_mut().time_end = bar.borrow().time;
        }
        dir
    }

    pub fn get_peak_klu(&self, is_high: bool) -> Option<StrongHandle<CKLineUnit>> {
        if is_high {
            self.get_high_peak_klu()
        } else {
            self.get_low_peak_klu()
        }
    }

    pub fn get_high_peak_klu(&self) -> Option<StrongHandle<CKLineUnit>> {
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

    pub fn get_low_peak_klu(&self) -> Option<StrongHandle<CKLineUnit>> {
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

    pub fn update_fx(
        cur: &StrongHandle<CKLine>,
        pre: &StrongHandle<CKLine>,
        next: &StrongHandle<CKLine>,
    ) {
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

    pub fn set_pre(&mut self, pre: StrongHandle<CKLine>) {
        self.pre = Some(Rc::downgrade(&pre));
    }

    pub fn set_next(&mut self, next: StrongHandle<CKLine>) {
        self.next = Some(Rc::downgrade(&next));
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

// Add these implementations after the CKLine struct definition
impl PartialEq for CKLine {
    fn eq(&self, other: &Self) -> bool {
        self.idx == other.idx
    }
}

impl Eq for CKLine {}

impl PartialOrd for CKLine {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.idx.partial_cmp(&other.idx)
    }
}

impl Ord for CKLine {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.idx.cmp(&other.idx)
    }
}
