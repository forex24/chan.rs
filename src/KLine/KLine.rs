use crate::Common::func_util::has_overlap;
use crate::Common::types::SharedCell;
use crate::Common::CEnum::{FxCheckMethod, FxType, KlineDir};
use crate::Common::ChanException::{CChanException, ErrCode};
use crate::KLine::KLine_Unit::CKLineUnit;
use std::cell::RefCell;
use std::rc::Rc;

pub struct CKLine {
    pub idx: i32,
    pub kl_type: Option<String>,
    pub fx: Option<FxType>,
    pub time_begin: String,
    pub time_end: String,
    pub low: f64,
    pub high: f64,
    pub dir: KlineDir,
    pub lst: Vec<SharedCell<CKLineUnit>>,
    pub pre: Option<SharedCell<CKLine>>,
    pub next: Option<SharedCell<CKLine>>,
}

impl CKLine {
    pub fn new(kl_unit: SharedCell<CKLineUnit>, idx: i32, dir: KlineDir) -> Self {
        let mut kline = CKLine {
            idx,
            kl_type: kl_unit.borrow().kl_type.clone(),
            fx: None,
            time_begin: kl_unit.borrow().time.to_string(),
            time_end: kl_unit.borrow().time.to_string(),
            low: kl_unit.borrow().low,
            high: kl_unit.borrow().high,
            dir,
            lst: vec![Rc::clone(&kl_unit)],
            pre: None,
            next: None,
        };
        kl_unit
            .borrow_mut()
            .set_klc(Rc::new(RefCell::new(kline.clone())));
        kline
    }

    pub fn get_sub_klc(&self) -> impl Iterator<Item = SharedCell<CKLine>> + '_ {
        let mut last_klc = None;
        self.lst.iter().flat_map(move |klu| {
            klu.borrow().get_children().filter_map(move |sub_klu| {
                let sub_klc = sub_klu.borrow().get_klc();
                if sub_klc != last_klc {
                    last_klc = sub_klc.clone();
                    sub_klc
                } else {
                    None
                }
            })
        })
    }

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
        item2: &CKLine,
        method: FxCheckMethod,
        for_virtual: bool,
    ) -> Result<bool, CChanException> {
        if self.next.is_none() || self.pre.is_none() || item2.pre.is_none() || item2.idx <= self.idx
        {
            return Err(CChanException::new(
                "Invalid kline sequence".to_string(),
                ErrCode::BiErr,
            ));
        }

        match self.fx {
            Some(FxType::Top) => {
                if !for_virtual && item2.fx != Some(FxType::Bottom) {
                    return Err(CChanException::new(
                        "Invalid fx sequence".to_string(),
                        ErrCode::BiErr,
                    ));
                }
                if for_virtual && item2.dir != KlineDir::Down {
                    return Ok(false);
                }

                let (item2_high, self_low) = match method {
                    FxCheckMethod::Half => (
                        item2.pre.as_ref().unwrap().borrow().high.max(item2.high),
                        self.low.min(self.next.as_ref().unwrap().borrow().low),
                    ),
                    FxCheckMethod::Loss => (item2.high, self.low),
                    FxCheckMethod::Strict | FxCheckMethod::Totally => {
                        let item2_high = if for_virtual {
                            item2.pre.as_ref().unwrap().borrow().high.max(item2.high)
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
                                .borrow()
                                .high
                                .max(item2.high)
                                .max(item2.next.as_ref().unwrap().borrow().high)
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
                    self.high > item2_high && item2.low < self_low
                })
            }
            Some(FxType::Bottom) => {
                if !for_virtual && item2.fx != Some(FxType::Top) {
                    return Err(CChanException::new(
                        "Invalid fx sequence".to_string(),
                        ErrCode::BiErr,
                    ));
                }
                if for_virtual && item2.dir != KlineDir::Up {
                    return Ok(false);
                }

                let (item2_low, cur_high) = match method {
                    FxCheckMethod::Half => (
                        item2.pre.as_ref().unwrap().borrow().low.min(item2.low),
                        self.high.max(self.next.as_ref().unwrap().borrow().high),
                    ),
                    FxCheckMethod::Loss => (item2.low, self.high),
                    FxCheckMethod::Strict | FxCheckMethod::Totally => {
                        let item2_low = if for_virtual {
                            item2.pre.as_ref().unwrap().borrow().low.min(item2.low)
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
                                .borrow()
                                .low
                                .min(item2.low)
                                .min(item2.next.as_ref().unwrap().borrow().low)
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
                    self.low < item2_low && item2.high > cur_high
                })
            }
            None => Err(CChanException::new(
                "only top/bottom fx can check_valid_top_button".to_string(),
                ErrCode::BiErr,
            )),
        }
    }
}

impl std::fmt::Display for CKLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fx_token = match self.fx {
            Some(FxType::Top) => "^",
            Some(FxType::Bottom) => "_",
            None => "",
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
