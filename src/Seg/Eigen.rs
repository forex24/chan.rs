use crate::Common::types::Handle;
use crate::Common::CEnum::{BiDir, EqualMode, FxType, KlineDir};

use super::linetype::Line;

pub struct CEigen<T> {
    pub high: f64,
    pub low: f64,
    pub lst: Vec<Handle<T>>,
    pub dir: KlineDir,
    pub fx: FxType,
    pub pre: Option<Handle<Self>>,
    pub next: Option<Handle<Self>>,
    pub gap: bool,
}

impl<T: Line<T>> CEigen<T> {
    pub fn new(bi: Handle<T>, dir: KlineDir) -> Self {
        let high = bi.borrow().high();
        let low = bi.borrow().low();
        CEigen {
            high,
            low,
            lst: vec![bi],
            dir,
            fx: FxType::Unknown,
            pre: None,
            next: None,
            gap: false,
        }
    }

    pub fn get_peak_bi_idx(&self) -> i32 {
        assert!(self.fx != FxType::Unknown);
        let bi_dir = self.lst[0].borrow().dir();
        if bi_dir == BiDir::Up {
            // 下降线段
            self.get_peak_klu(false).borrow().idx() - 1
        } else {
            self.get_peak_klu(true).borrow().idx() - 1
        }
    }

    pub fn try_add(
        &mut self,
        unit_kl: &Handle<T>,
        exclude_included: bool,
        allow_top_equal: Option<EqualMode>,
    ) -> KlineDir {
        let dir = self.test_combine(unit_kl, exclude_included, allow_top_equal);
        if dir == KlineDir::Combine {
            self.lst.push(unit_kl.clone());

            let unit_kl = unit_kl.borrow();
            match self.dir {
                KlineDir::Up => {
                    if unit_kl.high() != unit_kl.low() || unit_kl.high() != self.high {
                        self.high = self.high.max(unit_kl.high());
                        self.low = self.low.max(unit_kl.low());
                    }
                }
                KlineDir::Down => {
                    if unit_kl.high() != unit_kl.low() || unit_kl.low() != self.low {
                        self.high = self.high.min(unit_kl.high());
                        self.low = self.low.min(unit_kl.low());
                    }
                }
                _ => {
                    unreachable!()
                }
            }
        }
        dir
    }

    pub fn test_combine(
        &self,
        item: &Handle<T>,
        exclude_included: bool,
        allow_top_equal: Option<EqualMode>,
    ) -> KlineDir {
        let item = item.borrow();
        if self.high >= item.high() && self.low <= item.low() {
            return KlineDir::Combine;
        }
        if self.high <= item.high() && self.low >= item.low() {
            match allow_top_equal {
                Some(EqualMode::TopEqual) if self.high == item.high() && self.low > item.low() => {
                    return KlineDir::Down;
                }
                Some(EqualMode::BottomEqual)
                    if self.low == item.low() && self.high < item.high() =>
                {
                    return KlineDir::Up;
                }
                _ => {
                    return if exclude_included {
                        KlineDir::Included
                    } else {
                        KlineDir::Combine
                    };
                }
            }
        }
        if self.high > item.high() && self.low > item.low() {
            return KlineDir::Down;
        }
        if self.high < item.high() && self.low < item.low() {
            return KlineDir::Up;
        }
        unreachable!()
    }

    pub fn get_peak_klu(&self, is_high: bool) -> Handle<T> {
        if is_high {
            self.get_high_peak_klu()
        } else {
            self.get_low_peak_klu()
        }
    }

    pub fn get_high_peak_klu(&self) -> Handle<T> {
        for kl in self.lst.iter().rev() {
            if kl.borrow().high() == self.high {
                return kl.clone();
            }
        }
        unreachable!()
    }

    pub fn get_low_peak_klu(&self) -> Handle<T> {
        for kl in self.lst.iter().rev() {
            if kl.borrow().low() == self.low {
                return kl.clone();
            }
        }
        unreachable!()
    }
}

//impl<T> std::fmt::Display for CEigen<T> {
//    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//        write!(
//            f,
//            "{}~{} gap={} fx={:?}",
//            self.lst[0].borrow().idx,
//            self.lst.last().unwrap().borrow().idx,
//            self.gap,
//            self.fx
//        )
//    }
//}
