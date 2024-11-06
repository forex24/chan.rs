use crate::Combiner::Combine_Item::CCombineItem;
use crate::Common::handle::Handle;
use crate::Common::CEnum::{FxType, KlineDir};
use crate::Common::ChanException::{CChanException, ErrCode};
use crate::KLine::KLine_Unit::CKLineUnit;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct CKLineCombiner<T> {
    pub time_begin: i64,
    pub time_end: i64,
    pub high: f64,
    pub low: f64,
    pub lst: Vec<Handle<T>>,
    pub dir: KlineDir,
    pub fx: FxType,
    pub pre: Option<Handle<CKLineCombiner<T>>>,
    pub next: Option<Handle<CKLineCombiner<T>>>,
    memoize_cache: HashMap<String, f64>,
}

impl<T> CKLineCombiner<T> {
    pub fn new(kl_unit: Handle<T>, dir: KlineDir) -> Result<Self, CChanException> {
        let item = CCombineItem::new(kl_unit.clone())?;
        Ok(CKLineCombiner {
            time_begin: item.time_begin,
            time_end: item.time_end,
            high: item.high,
            low: item.low,
            lst: vec![kl_unit],
            dir,
            fx: FxType::Unknown,
            pre: None,
            next: None,
            memoize_cache: HashMap::new(),
        })
    }

    pub fn clean_cache(&mut self) {
        self.memoize_cache.clear();
    }

    pub fn time_begin(&self) -> i64 {
        self.time_begin
    }
    pub fn time_end(&self) -> i64 {
        self.time_end
    }
    pub fn high(&self) -> f64 {
        self.high
    }
    pub fn low(&self) -> f64 {
        self.low
    }
    pub fn lst(&self) -> &Vec<Handle<T>> {
        &self.lst
    }
    pub fn dir(&self) -> KlineDir {
        self.dir
    }
    pub fn fx(&self) -> FxType {
        self.fx
    }

    pub fn pre(&self) -> Result<Handle<CKLineCombiner<T>>, CChanException> {
        self.pre.clone().ok_or_else(|| {
            CChanException::new("No previous combiner".to_string(), ErrCode::CombinerErr)
        })
    }

    pub fn next(&self) -> Option<Handle<CKLineCombiner<T>>> {
        self.next.clone()
    }

    pub fn get_next(&self) -> Result<Handle<CKLineCombiner<T>>, CChanException> {
        self.next.clone().ok_or_else(|| {
            CChanException::new("No next combiner".to_string(), ErrCode::CombinerErr)
        })
    }

    pub fn test_combine(
        &self,
        item: &CCombineItem,
        exclude_included: bool,
        allow_top_equal: Option<i32>,
    ) -> Result<KlineDir, CChanException> {
        if self.high >= item.high && self.low <= item.low {
            return Ok(KlineDir::Combine);
        }
        if self.high <= item.high && self.low >= item.low {
            match allow_top_equal {
                Some(1) if self.high == item.high && self.low > item.low => {
                    return Ok(KlineDir::Down)
                }
                Some(-1) if self.low == item.low && self.high < item.high => {
                    return Ok(KlineDir::Up)
                }
                _ => {
                    return Ok(if exclude_included {
                        KlineDir::Included
                    } else {
                        KlineDir::Combine
                    })
                }
            }
        }
        if self.high > item.high && self.low > item.low {
            return Ok(KlineDir::Down);
        }
        if self.high < item.high && self.low < item.low {
            return Ok(KlineDir::Up);
        }
        Err(CChanException::new(
            "combine type unknown".to_string(),
            ErrCode::CombinerErr,
        ))
    }

    pub fn add(&mut self, unit_kl: Handle<T>) {
        self.lst.push(unit_kl);
    }

    pub fn set_fx(&mut self, fx: FxType) {
        self.fx = fx;
    }

    pub fn try_add(
        &mut self,
        unit_kl: Handle<T>,
        exclude_included: bool,
        allow_top_equal: Option<i32>,
    ) -> Result<KlineDir, CChanException> {
        let combine_item = CCombineItem::new(unit_kl.clone())?;
        let dir = self.test_combine(&combine_item, exclude_included, allow_top_equal)?;
        if dir == KlineDir::Combine {
            self.lst.push(unit_kl.clone());
            if let Ok(kline_unit) = unit_kl
                .try_borrow_mut()
                .unwrap()
                .downcast_mut::<CKLineUnit>()
            {
                kline_unit.set_klc(Rc::new(RefCell::new(self.clone())));
            }
            match self.dir {
                KlineDir::Up => {
                    if combine_item.high != combine_item.low || combine_item.high != self.high {
                        self.high = self.high.max(combine_item.high);
                        self.low = self.low.max(combine_item.low);
                    }
                }
                KlineDir::Down => {
                    if combine_item.high != combine_item.low || combine_item.low != self.low {
                        self.high = self.high.min(combine_item.high);
                        self.low = self.low.min(combine_item.low);
                    }
                }
                _ => {
                    return Err(CChanException::new(
                        format!(
                            "KlineDir = {:?} err!!! must be {:?}/{:?}",
                            self.dir,
                            KlineDir::Up,
                            KlineDir::Down
                        ),
                        ErrCode::CombinerErr,
                    ))
                }
            }
            self.time_end = combine_item.time_end;
            self.clean_cache();
        }
        Ok(dir)
    }

    pub fn get_peak_klu(&self, is_high: bool) -> Result<Handle<T>, CChanException> {
        if is_high {
            self.get_high_peak_klu()
        } else {
            self.get_low_peak_klu()
        }
    }

    pub fn get_high_peak_klu(&mut self) -> Result<Handle<T>, CChanException> {
        if let Some(cached) = self.memoize_cache.get("high_peak") {
            return Ok(cached.clone());
        }
        for kl in self.lst.iter().rev() {
            let item = CCombineItem::new(kl.clone())?;
            if item.high == self.high {
                self.memoize_cache
                    .insert("high_peak".to_string(), kl.clone());
                return Ok(kl.clone());
            }
        }
        Err(CChanException::new(
            "can't find peak...".to_string(),
            ErrCode::CombinerErr,
        ))
    }

    pub fn get_low_peak_klu(&mut self) -> Result<Handle<T>, CChanException> {
        if let Some(cached) = self.memoize_cache.get("low_peak") {
            return Ok(cached.clone());
        }
        for kl in self.lst.iter().rev() {
            let item = CCombineItem::new(kl.clone())?;
            if item.low == self.low {
                self.memoize_cache
                    .insert("low_peak".to_string(), kl.clone());
                return Ok(kl.clone());
            }
        }
        Err(CChanException::new(
            "can't find peak...".to_string(),
            ErrCode::CombinerErr,
        ))
    }

    pub fn update_fx(
        &mut self,
        pre: Handle<CKLineCombiner<T>>,
        next: Handle<CKLineCombiner<T>>,
        exclude_included: bool,
        allow_top_equal: Option<i32>,
    ) {
        self.set_next(next.clone());
        self.set_pre(pre.clone());
        next.borrow_mut()
            .set_pre(Rc::new(RefCell::new(self.clone())));
        let pre = pre.borrow();
        let next = next.borrow();
        if exclude_included {
            if pre.high < self.high && next.high <= self.high && next.low < self.low {
                if allow_top_equal == Some(1) || next.high < self.high {
                    self.fx = FxType::Top;
                }
            } else if next.high > self.high && pre.low > self.low && next.low >= self.low {
                if allow_top_equal == Some(-1) || next.low > self.low {
                    self.fx = FxType::Bottom;
                }
            }
        } else if pre.high < self.high
            && next.high < self.high
            && pre.low < self.low
            && next.low < self.low
        {
            self.fx = FxType::Top;
        } else if pre.high > self.high
            && next.high > self.high
            && pre.low > self.low
            && next.low > self.low
        {
            self.fx = FxType::Bottom;
        }
        self.clean_cache();
    }

    pub fn set_pre(&mut self, pre: Handle<CKLineCombiner<T>>) {
        self.pre = Some(pre);
        self.clean_cache();
    }

    pub fn set_next(&mut self, next: Handle<CKLineCombiner<T>>) {
        self.next = Some(next);
        self.clean_cache();
    }
}

impl<T> std::fmt::Display for CKLineCombiner<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}~{} {}->{}",
            self.time_begin, self.time_end, self.low, self.high
        )
    }
}

impl<T> std::ops::Index<usize> for CKLineCombiner<T> {
    type Output = Handle<T>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.lst[index]
    }
}

impl<T> std::ops::Index<std::ops::Range<usize>> for CKLineCombiner<T> {
    type Output = [Handle<T>];

    fn index(&self, range: std::ops::Range<usize>) -> &Self::Output {
        &self.lst[range]
    }
}

impl<T> std::iter::IntoIterator for CKLineCombiner<T> {
    type Item = Handle<T>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.lst.into_iter()
    }
}
