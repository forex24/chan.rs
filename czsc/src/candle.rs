use std::fmt::Display;

use chrono::DateTime;
use chrono::Utc;

use crate::has_overlap;
use crate::AsHandle;
use crate::Bar;
use crate::Handle;
use crate::ICandlestick;
use crate::IHighLow;
use crate::{FxCheckMethod, FxType, KlineDir};

// 合并后的K线
#[derive(Debug)]
pub struct Candle {
    handle: Handle<Self>,
    pub time_begin: DateTime<Utc>,
    pub time_end: DateTime<Utc>,
    pub high: f64,
    pub low: f64,
    pub lst: Vec<Handle<Bar>>,
    pub dir: KlineDir,
    pub fx_type: FxType,
}

impl Candle {
    #[allow(clippy::borrowed_box)]
    pub fn new(box_vec: &Box<Vec<Self>>, bar: Handle<Bar>, idx: usize, dir: KlineDir) -> Candle {
        let c = Self {
            handle: Handle::new(box_vec, idx),
            time_begin: bar.time,
            time_end: bar.time,
            lst: vec![bar],
            high: bar.high,
            low: bar.low,
            dir,            fx_type: FxType::Unknown,
        };
        bar.as_mut().set_klc(&c);
        c
    }

    pub fn get_sub_klc(&self) -> impl Iterator<Item = Handle<Self>> + '_ {
        // 可能会出现相邻的两个KLC的子KLC会有重复
        // 因为子KLU合并时正好跨过了父KLC的结束时间边界
        let mut last_klc: Option<Handle<Self>> = None;
        self.lst.iter().flat_map(move |klu| {
            klu.get_children().filter_map(move |sub_klu| {
                let klc = sub_klu.klc;
                if klc.is_none()
                    || last_klc.is_none()
                    || klc.unwrap().index() != last_klc.unwrap().index()
                {
                    last_klc = sub_klu.klc;
                    sub_klu.klc
                } else {
                    None
                }
            })
        })
    }

    fn get_klu_max_high(&self) -> f64 {
        self.lst.iter().map(|x| x.high).reduce(f64::max).unwrap()
    }

    fn get_klu_min_low(&self) -> f64 {
        self.lst.iter().map(|x| x.low).reduce(f64::min).unwrap()
    }

    pub fn has_gap_with_next(&self) -> bool {
        let next = self.as_handle().next();
        assert!(next.is_some());
        //相同也算重叠，也就是没有gap
        !has_overlap(
            self.get_klu_min_low(),
            self.get_klu_max_high(),
            next.unwrap().get_klu_min_low(),
            next.unwrap().get_klu_max_high(),
            true,
        )
    }

    // 已完备
    pub fn check_fx_valid(
        lhs: Handle<Candle>,
        rhs: Handle<Candle>,
        method: FxCheckMethod,
        for_virtual: bool, // 虚笔时使用
    ) -> bool {
        assert!(lhs.next().is_some() && rhs.prev().is_some());
        assert!(lhs.prev().is_some());
        assert!(rhs.index() > lhs.index());
        match lhs.fx_type {
            FxType::Top => {
                assert!(for_virtual || rhs.fx_type == FxType::Bottom);
                if for_virtual && rhs.dir != KlineDir::Down {
                    return false;
                }
                let (item2_high, self_low) = match method {
                    FxCheckMethod::Half => (
                        // 检测前两KLC
                        rhs.prev().unwrap().high.max(rhs.high),
                        lhs.low.min(lhs.next().unwrap().low),
                    ),
                    FxCheckMethod::Loss => (rhs.high, lhs.low), // 只检测顶底分形KLC
                    FxCheckMethod::Strict | FxCheckMethod::Totally => {
                        if for_virtual {
                            (
                                rhs.prev().unwrap().high.max(rhs.high),
                                lhs.prev()
                                    .unwrap()
                                    .low
                                    .min(lhs.low)
                                    .min(lhs.next().unwrap().low),
                            )
                        } else {
                            assert!(rhs.next().is_some());
                            (
                                rhs.prev()
                                    .unwrap()
                                    .high
                                    .max(rhs.high)
                                    .max(rhs.next().unwrap().high),
                                lhs.prev()
                                    .unwrap()
                                    .low
                                    .min(lhs.low)
                                    .min(lhs.next().unwrap().low),
                            )
                        }
                    }
                };

                if method == FxCheckMethod::Totally {
                    lhs.low > item2_high
                } else {
                    lhs.high > item2_high && rhs.low < self_low
                }
            }
            FxType::Bottom => {
                assert!(for_virtual || rhs.fx_type == FxType::Top);
                if for_virtual && rhs.dir != KlineDir::Up {
                    return false;
                }
                let (item2_low, cur_high) = match method {
                    FxCheckMethod::Half => (
                        rhs.prev().unwrap().low.min(rhs.low),
                        lhs.high.max(lhs.next().unwrap().high),
                    ),
                    FxCheckMethod::Loss => (rhs.low, lhs.high),
                    FxCheckMethod::Strict | FxCheckMethod::Totally => {
                        if for_virtual {
                            (
                                rhs.prev().unwrap().low.min(rhs.low),
                                lhs.prev()
                                    .unwrap()
                                    .high
                                    .max(lhs.high)
                                    .max(lhs.next().unwrap().high),
                            )
                        } else {
                            assert!(rhs.next().is_some());
                            (
                                rhs.prev()
                                    .unwrap()
                                    .low
                                    .min(rhs.low)
                                    .min(rhs.next().unwrap().low),
                                lhs.prev()
                                    .unwrap()
                                    .high
                                    .max(lhs.high)
                                    .max(lhs.next().unwrap().high),
                            )
                        }
                    }
                };

                if method == FxCheckMethod::Totally {
                    lhs.high < item2_low
                } else {
                    lhs.low < item2_low && rhs.high > cur_high
                }
            }
            _ => panic!("only top/bottom fx can check_valid_top_button"),
        }
    }
}

impl Candle {
    fn test_combine(&self, item: &Bar) -> KlineDir {
        /*let high_cmp = f64::total_cmp(&self.high, &item.high);
        let low_cmp = f64::total_cmp(&self.low, &item.low);
        match (high_cmp, low_cmp) {
            (Ordering::Greater | Ordering::Equal, Ordering::Less | Ordering::Equal) => {
                KlineDir::Combine
            }
            (Ordering::Less | Ordering::Equal, Ordering::Greater | Ordering::Equal) => {
                KlineDir::Combine
            }
            (Ordering::Greater, Ordering::Greater) => KlineDir::Down,
            (Ordering::Less, Ordering::Less) => KlineDir::Up,
        }*/
        if self.high >= item.high && self.low <= item.low {
            return KlineDir::Combine;
        }
        if self.high <= item.high && self.low >= item.low {
            return KlineDir::Combine;
        }
        if self.high > item.high && self.low > item.low {
            return KlineDir::Down;
        }
        if self.high < item.high && self.low < item.low {
            KlineDir::Up
        } else {
            panic!("combine type unknown");
        }
    }

    pub(crate) fn try_add(&mut self, bar: Handle<Bar>) -> KlineDir {
        let _dir = self.test_combine(&bar);
        if _dir == KlineDir::Combine {
            self.lst.push(bar);

            bar.as_mut().set_klc(self);

            if self.dir == KlineDir::Up {
                // 一字板不用处理
                if bar.high != bar.low || bar.high != self.high {
                    self.high = f64::max(self.high, bar.high);
                    self.low = f64::max(self.low, bar.low);
                }
            } else if self.dir == KlineDir::Down {
                if bar.high != bar.low || bar.low != self.low {
                    self.high = f64::min(self.high, bar.high);
                    self.low = f64::min(self.low, bar.low);
                }
            } else {
                panic!("KlineDir {} err!!! must be Up/Down", _dir);
            }
            self.time_end = bar.time;
        }
        _dir
    }

    pub fn get_begin_klu(&self) -> &Bar {
        &self.lst[0]
    }

    pub fn get_end_klu(&self) -> &Bar {
        self.lst.last().unwrap()
    }

    pub fn get_peak_val(&self, is_high: bool) -> f64 {
        if is_high {
            self.get_high_peak_klu().high
        } else {
            self.get_low_peak_klu().low
        }
    }

    pub fn get_peak_klu(&self, is_high: bool) -> &Bar {
        // 获取最大值 or 最小值所在klu/bi
        if is_high {
            self.get_high_peak_klu()
        } else {
            self.get_low_peak_klu()
        }
    }

    fn get_high_peak_klu(&self) -> &Bar {
        for kl in self.lst.iter().rev() {
            if kl.high == self.high {
                return kl;
            }
        }
        panic!("can't find peak...");
    }

    fn get_low_peak_klu(&self) -> &Bar {
        for kl in self.lst.iter().rev() {
            if kl.low == self.low {
                return kl;
            }
        }
        panic!("can't find peak...");
    }
}

impl Display for Candle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "index {} dir {} fx_tyep {} time_begin:{} time_end:{} high:{} low:{}",
            self.as_handle().index(),
            self.dir,
            self.fx_type,
            self.time_begin,
            self.time_end,
            self.high,
            self.low
        )
    }
}

impl IHighLow for Candle {
    fn high(&self) -> f64 {
        self.high
    }

    fn low(&self) -> f64 {
        self.low
    }
}

impl ICandlestick for Candle {
    fn unix_time(&self) -> i64 {
        self.time_begin.timestamp_millis()
    }

    fn open(&self) -> f64 {
        self.high
    }

    fn close(&self) -> f64 {
        self.low
    }
}

impl_handle!(Candle);
