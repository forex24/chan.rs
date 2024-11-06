use crate::{Direction, EqualMode, FxType, IHighLow, LineType, ToHandle};
use crate::{Handle, KlineDir};

// 特征序列的特征
#[derive(Debug)]
pub struct CEigen<T> {
    pub high: f64,
    pub low: f64,
    pub lst: Vec<Handle<T>>,
    pub dir: KlineDir,
    pub fx_type: FxType,
    pub gap: bool,
}

impl<T: LineType + ToHandle> CEigen<T> {
    pub fn new(bi: Handle<T>, dir: KlineDir) -> Self {
        Self {
            high: bi.high(),
            low: bi.low(),
            lst: vec![bi],
            dir,
            fx_type: FxType::Unknown,
            gap: false,
        }
    }

    pub fn get_peak_bi_idx(&self) -> usize {
        assert!(self.fx_type != FxType::Unknown);
        let bi_dir = self.lst.first().unwrap().direction();
        match bi_dir {
            Direction::Up => self.get_peak_klu(false).to_handle().index() - 1,
            Direction::Down => self.get_peak_klu(true).to_handle().index() - 1,
        }
    }

    pub fn try_add(
        &mut self,
        unit_kl: Handle<T>,
        exclude_included: bool,
        allow_top_equal: Option<EqualMode>,
    ) -> KlineDir {
        let _dir = self.test_combine(&unit_kl, exclude_included, allow_top_equal);
        if _dir == KlineDir::Combine {
            self.lst.push(unit_kl);

            match self.dir {
                KlineDir::Up => {
                    if unit_kl.high() != unit_kl.low() || unit_kl.high() != self.high {
                        self.high = f64::max(self.high, unit_kl.high());
                        self.low = f64::max(self.low, unit_kl.low());
                    }
                }
                KlineDir::Down => {
                    if unit_kl.high() != unit_kl.low() || unit_kl.low() != self.low {
                        self.high = f64::min(self.high, unit_kl.high());
                        self.low = f64::min(self.low, unit_kl.low());
                    }
                }
                _ => panic!(
                    "KlineDir = {} err!!! must be {}/{}",
                    self.dir,
                    KlineDir::Up,
                    KlineDir::Down
                ),
            }
        }
        _dir
    }

    fn test_combine(
        &self,
        item: &T,
        exclude_included: bool,
        allow_top_equal: Option<EqualMode>,
    ) -> KlineDir {
        if self.high >= item.high() && self.low <= item.low() {
            return KlineDir::Combine;
        }
        if self.high <= item.high() && self.low >= item.low() {
            if allow_top_equal == Some(EqualMode::TopEqual)
                && self.high == item.high()
                && self.low > item.low()
            {
                return KlineDir::Down;
            }
            if allow_top_equal == Some(EqualMode::BottomEqual)
                && self.low == item.low()
                && self.high < item.high()
            {
                return KlineDir::Up;
            }
            return if exclude_included {
                KlineDir::Included
            } else {
                KlineDir::Combine
            };
        }
        if self.high > item.high() && self.low > item.low() {
            return KlineDir::Down;
        }
        if self.high < item.high() && self.low < item.low() {
            KlineDir::Up
        } else {
            panic!("combine type unknown");
        }
    }

    fn get_peak_klu(&self, is_high: bool) -> &T {
        if is_high {
            self.get_high_peak_klu()
        } else {
            self.get_low_peak_klu()
        }
    }

    fn get_high_peak_klu(&self) -> &T {
        for kl in self.lst.iter().rev() {
            if kl.high() == self.high {
                return kl;
            }
        }
        panic!("can't find peak...");
    }

    fn get_low_peak_klu(&self) -> &T {
        for kl in self.lst.iter().rev() {
            if kl.low() == self.low {
                return kl;
            }
        }
        panic!("can't find peak...");
    }
}

impl<T> IHighLow for CEigen<T> {
    fn high(&self) -> f64 {
        self.high
    }

    fn low(&self) -> f64 {
        self.low
    }
}
