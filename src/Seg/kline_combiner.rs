// kline_combiner.rs
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::Bi::Bi::CBi;
use crate::Common::CEnum::{FxType, KLineDir};
use crate::Common::ChanException::{CChanException, ErrCode};

pub type Handle<T> = Rc<RefCell<T>>;

pub struct CCombineItem {
    pub time_begin: usize,
    pub time_end: usize,
    pub high: f64,
    pub low: f64,
}

impl CCombineItem {
    pub fn from_bi(bi: &Handle<CBi>) -> Result<Self, CChanException> {
        Ok(Self {
            time_begin: bi.borrow().begin_klc.borrow().idx as usize,
            time_end: bi.borrow().end_klc.borrow().idx as usize,
            high: bi.borrow()._high(),
            low: bi.borrow()._low(),
        })
    }
}

pub struct CKLineCombiner {
    pub(crate) time_begin: usize,
    pub(crate) time_end: usize,
    pub(crate) high: f64,
    pub(crate) low: f64,
    pub(crate) lst: Vec<Handle<CBi>>,
    pub(crate) dir: KLineDir,
    pub(crate) fx: FxType,
    pub(crate) pre: Option<Handle<Self>>,
    pub(crate) next: Option<Handle<Self>>,
    cache: RefCell<HashMap<String, Handle<CBi>>>,
}

impl CKLineCombiner {
    pub fn new(bi: Handle<CBi>, dir: KLineDir) -> Result<Self, CChanException> {
        let item = CCombineItem::from_bi(&bi)?;

        Ok(Self {
            time_begin: item.time_begin,
            time_end: item.time_end,
            high: item.high,
            low: item.low,
            lst: vec![bi],
            dir,
            fx: FxType::Unknown,
            pre: None,
            next: None,
            cache: RefCell::new(HashMap::new()),
        })
    }

    pub fn clean_cache(&self) {
        self.cache.borrow_mut().clear();
    }

    pub fn test_combine(
        &self,
        item: &CCombineItem,
        exclude_included: bool,
        allow_top_equal: Option<i32>,
    ) -> Result<KLineDir, CChanException> {
        if self.high >= item.high && self.low <= item.low {
            return Ok(KLineDir::Combine);
        }
        if self.high <= item.high && self.low >= item.low {
            match allow_top_equal {
                Some(1) if self.high == item.high && self.low > item.low => {
                    return Ok(KLineDir::Down)
                }
                Some(-1) if self.low == item.low && self.high < item.high => {
                    return Ok(KLineDir::Up)
                }
                _ => {
                    return Ok(if exclude_included {
                        KLineDir::Included
                    } else {
                        KLineDir::Combine
                    })
                }
            }
        }
        if self.high > item.high && self.low > item.low {
            return Ok(KLineDir::Down);
        }
        if self.high < item.high && self.low < item.low {
            return Ok(KLineDir::Up);
        }
        Err(CChanException::new(
            "combine type unknown".to_string(),
            ErrCode::CombinerErr,
        ))
    }

    pub fn try_add(
        &mut self,
        bi: Handle<CBi>,
        exclude_included: bool,
        allow_top_equal: Option<i32>,
    ) -> Result<KLineDir, CChanException> {
        let item = CCombineItem::from_bi(&bi)?;
        let dir = self.test_combine(&item, exclude_included, allow_top_equal)?;

        if dir == KLineDir::Combine {
            self.lst.push(bi);
            match self.dir {
                KLineDir::Up => {
                    if item.high != item.low || item.high != self.high {
                        self.high = self.high.max(item.high);
                        self.low = self.low.max(item.low);
                    }
                }
                KLineDir::Down => {
                    if item.high != item.low || item.low != self.low {
                        self.high = self.high.min(item.high);
                        self.low = self.low.min(item.low);
                    }
                }
                _ => {
                    return Err(CChanException::new(
                        format!("KLINE_DIR = {:?} err!!! must be Up/Down", self.dir).to_string(),
                        ErrCode::CombinerErr,
                    ))
                }
            }
            self.time_end = item.time_end;
            self.clean_cache();
        }
        Ok(dir)
    }

    pub fn add(&mut self, unit_kl: Handle<CBi>) {
        self.lst.push(unit_kl.clone());
        if let Ok(item) = CCombineItem::from_bi(&unit_kl) {
            //if let Ok(klu) = unit_kl.try_into::<CKLineUnit>() {
            //    klu.set_klc(self);
            //}
            match self.dir {
                KLineDir::Up => {
                    if item.high != item.low || item.high != self.high {
                        self.high = self.high.max(item.high);
                        self.low = self.low.max(item.low);
                    }
                }
                KLineDir::Down => {
                    if item.high != item.low || item.low != self.low {
                        self.high = self.high.min(item.high);
                        self.low = self.low.min(item.low);
                    }
                }
                _ => panic!("Invalid KLineDir"),
            }
            self.time_end = item.time_end;
            self.clean_cache();
        }
    }

    pub fn get_peak_klu(&self, is_high: bool) -> Result<Handle<CBi>, CChanException> {
        if is_high {
            self.get_high_peak_klu()
        } else {
            self.get_low_peak_klu()
        }
    }

    pub fn get_high_peak_klu(&self) -> Result<Handle<CBi>, CChanException> {
        let cache_key = "high_peak";
        if let Some(cached) = self.cache.borrow().get(cache_key) {
            return Ok(cached.clone());
        }

        for kl in self.lst.iter().rev() {
            let item = CCombineItem::from_bi(kl)?;
            if item.high == self.high {
                self.cache
                    .borrow_mut()
                    .insert(cache_key.to_string(), kl.clone());
                return Ok(kl.clone());
            }
        }
        Err(CChanException::new(
            "can't find peak...".to_string(),
            ErrCode::CombinerErr,
        ))
    }

    pub fn get_low_peak_klu(&self) -> Result<Handle<CBi>, CChanException> {
        let cache_key = "low_peak";
        if let Some(cached) = self.cache.borrow().get(cache_key) {
            return Ok(cached.clone());
        }

        for kl in self.lst.iter().rev() {
            let item = CCombineItem::from_bi(kl)?;
            if item.low == self.low {
                self.cache
                    .borrow_mut()
                    .insert(cache_key.to_string(), kl.clone());
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
        pre: &Self,
        next: &mut Self,
        exclude_included: bool,
        allow_top_equal: Option<i32>,
    ) {
        self.set_next(Some(Rc::new(RefCell::new(next.clone()))));
        self.set_pre(Some(Rc::new(RefCell::new(pre.clone()))));
        next.set_pre(Some(Rc::new(RefCell::new(self.clone()))));

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

    pub fn set_pre(&mut self, pre: Option<Handle<Self>>) {
        self.pre = pre;
        self.clean_cache();
    }

    pub fn set_next(&mut self, next: Option<Handle<Self>>) {
        self.next = next;
        self.clean_cache();
    }
}

impl Clone for CKLineCombiner {
    fn clone(&self) -> Self {
        Self {
            time_begin: self.time_begin,
            time_end: self.time_end,
            high: self.high,
            low: self.low,
            lst: self.lst.clone(),
            dir: self.dir,
            fx: self.fx,
            pre: self.pre.clone(),
            next: self.next.clone(),
            cache: RefCell::new(HashMap::new()),
        }
    }
}

impl std::fmt::Display for CKLineCombiner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}~{} {}->{}",
            self.time_begin, self.time_end, self.low, self.high
        )
    }
}
