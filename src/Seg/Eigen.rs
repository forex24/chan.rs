// eigen.rs
// 已完备
use std::rc::Rc;
use std::rc::Weak;

use crate::Common::types::Handle;
use crate::Common::types::WeakHandle;
use crate::Common::CEnum::{BiDir, EqualMode, FxType, KLineDir};

use super::linetype::Line;

//#[derive(Debug, Clone)]
pub struct CEigen<T> {
    //begin_klc: usize,
    //end_klc: usize,
    pub high: f64,
    pub low: f64,
    pub lst: Vec<WeakHandle<T>>,
    pub dir: KLineDir,
    pub fx: FxType,
    //pre: Option<Handle<Self>>,
    //next: Option<Handle<Self>>,
    pub gap: bool,
}

impl<T: Line> CEigen<T> {
    pub fn new(bi: &WeakHandle<T>, dir: KLineDir) -> Self {
        Self {
            //begin_klc: bi.borrow().line_get_begin_klc().borrow().idx ,
            //end_klc: bi.borrow().line_get_end_klc().borrow().idx ,
            high: bi.upgrade().unwrap().borrow().line_high(),
            low: bi.upgrade().unwrap().borrow().line_low(),
            lst: vec![Rc::downgrade(&bi.upgrade().unwrap())],
            dir,
            fx: FxType::Unknown,
            //pre: None,
            //next: None,
            gap: false,
        }
    }

    // 已完备
    pub fn test_combine(
        &self,
        item: &WeakHandle<T>,
        exclude_included: bool,
        allow_top_equal: Option<EqualMode>,
    ) -> KLineDir {
        let item = item.upgrade().unwrap();
        if self.high >= item.borrow().line_high() && self.low <= item.borrow().line_low() {
            return KLineDir::Combine;
        }
        if self.high <= item.borrow().line_high() && self.low >= item.borrow().line_low() {
            match allow_top_equal {
                Some(EqualMode::TopEqual)
                    if self.high == item.borrow().line_high()
                        && self.low > item.borrow().line_low() =>
                {
                    return KLineDir::Down
                }
                Some(EqualMode::BottomEqual)
                    if self.low == item.borrow().line_low()
                        && self.high < item.borrow().line_high() =>
                {
                    return KLineDir::Up
                }
                _ => {
                    return if exclude_included {
                        KLineDir::Included
                    } else {
                        KLineDir::Combine
                    }
                }
            }
        }
        if self.high > item.borrow().line_high() && self.low > item.borrow().line_low() {
            return KLineDir::Down;
        }
        if self.high < item.borrow().line_high() && self.low < item.borrow().line_low() {
            return KLineDir::Up;
        }
        unreachable!()
    }

    // 已完备
    pub fn try_add(
        &mut self,
        bi: &WeakHandle<T>,
        exclude_included: bool,
        allow_top_equal: Option<EqualMode>,
    ) -> KLineDir {
        let dir = self.test_combine(bi, exclude_included, allow_top_equal);
        let bi_strong = bi.upgrade().unwrap();
        let high = bi_strong.borrow().line_high();
        let low = bi_strong.borrow().line_low();

        if dir == KLineDir::Combine {
            self.lst.push(Rc::downgrade(&bi_strong));

            match self.dir {
                KLineDir::Up => {
                    if high != low || high != self.high {
                        self.high = self.high.max(high);
                        self.low = self.low.max(low);
                    }
                }
                KLineDir::Down => {
                    if high != low || low != self.low {
                        self.high = self.high.min(high);
                        self.low = self.low.min(low);
                    }
                }
                _ => unreachable!("KLINE_DIR = {:?} err!!! must be Up/Down", self.dir),
            }
            //self.end_klc = item.line_get_end_klc().borrow().idx ;
        }
        dir
    }

    //pub fn add(&mut self, unit_kl: Handle<T>) {
    //    self.lst.push(Rc::clone(&unit_kl));
    //
    //    let item = unit_kl.borrow();
    //    match self.dir {
    //        KLineDir::Up => {
    //            if item.high() != item.low() || item.high() != self.high {
    //                self.high = self.high.max(item.high());
    //                self.low = self.low.max(item.low());
    //            }
    //        }
    //        KLineDir::Down => {
    //            if item.high() != item.low() || item.low() != self.low {
    //                self.high = self.high.min(item.high());
    //                self.low = self.low.min(item.low());
    //            }
    //        }
    //        _ => panic!("Invalid KLineDir"),
    //    }
    //    self.end_klc = item.get_end_klc().borrow().idx ;
    //}

    pub fn get_peak_klu(&self, is_high: bool) -> WeakHandle<T> {
        if is_high {
            self.get_high_peak_klu()
        } else {
            self.get_low_peak_klu()
        }
    }

    pub fn get_high_peak_klu(&self) -> WeakHandle<T> {
        for kl in self.lst.iter().rev() {
            if kl.upgrade().unwrap().borrow().line_high() == self.high {
                return Rc::downgrade(&kl.upgrade().unwrap());
            }
        }
        unreachable!("can't find peak high...")
    }

    pub fn get_low_peak_klu(&self) -> WeakHandle<T> {
        for kl in self.lst.iter().rev() {
            if kl.upgrade().unwrap().borrow().line_low() == self.low {
                return Rc::downgrade(&kl.upgrade().unwrap());
            }
        }
        unreachable!("can't find peak low...")
    }
    //pub fn check_gap(&self, k1: &Handle<Self>) -> bool {
    //    //检查是否有gap
    //    if (self.fx == FxType::Top && k1.borrow().high < self.low)
    //        || (self.fx == FxType::Bottom && k1.borrow().low > self.high)
    //    {
    //        true
    //    } else {
    //        false
    //    }
    //}
    //
    //pub fn update_fx2(
    //    &mut self,
    //    pre: &Self,
    //    next: &mut Self,
    //    exclude_included: bool,
    //    allow_top_equal: Option<EqualMode>,
    //) {
    //    //self.set_next(Some(Rc::new(RefCell::new(next.clone()))));
    //    //self.set_pre(Some(Rc::new(RefCell::new(pre.clone()))));
    //    //next.set_pre(Some(Rc::new(RefCell::new(self.clone()))));
    //
    //    if exclude_included {
    //        if pre.high < self.high && next.high <= self.high && next.low < self.low {
    //            if allow_top_equal == Some(EqualMode::TopEqual) || next.high < self.high {
    //                self.fx = FxType::Top;
    //            }
    //        } else if next.high > self.high && pre.low > self.low && next.low >= self.low {
    //            if allow_top_equal == Some(EqualMode::BottomEqual) || next.low > self.low {
    //                self.fx = FxType::Bottom;
    //            }
    //        }
    //    } else if pre.high < self.high
    //        && next.high < self.high
    //        && pre.low < self.low
    //        && next.low < self.low
    //    {
    //        self.fx = FxType::Top;
    //    } else if pre.high > self.high
    //        && next.high > self.high
    //        && pre.low > self.low
    //        && next.low > self.low
    //    {
    //        self.fx = FxType::Bottom;
    //    }
    //}
    //
    //pub fn set_pre(&mut self, pre: Option<Handle<Self>>) {
    //    self.pre = pre;
    //}
    //
    //pub fn set_next(&mut self, next: Option<Handle<Self>>) {
    //    self.next = next;
    //}

    //pub fn update_fx(
    //    &mut self,
    //    pre: &Handle<CEigen<T>>,
    //    next: &Handle<CEigen<T>>,
    //    exclude_included: bool,
    //    allow_top_equal: Option<EqualMode>,
    //) {
    //    // 先调用父类的update_fx
    //    self.update_fx(
    //        &pre.borrow().inner,
    //        &mut next.borrow_mut().inner,
    //        exclude_included,
    //        allow_top_equal,
    //    );
    //
    //    // 检查是否有gap
    //    if (self.fx == FxType::Top && pre.borrow().high < self.low)
    //        || (self.fx == FxType::Bottom && pre.borrow().low > self.high)
    //    {
    //        self.gap = true;
    //    }
    //}

    // 已完备
    pub fn get_peak_bi_idx(&self) -> usize {
        assert!(self.fx != FxType::Unknown);
        let bi_dir = self.lst[0].upgrade().unwrap().borrow().line_dir();
        match bi_dir {
            // 下降线段
            BiDir::Up => {
                self.get_peak_klu(false)
                    .upgrade()
                    .unwrap()
                    .borrow()
                    .line_idx()
                    - 1
            }
            BiDir::Down => {
                self.get_peak_klu(true)
                    .upgrade()
                    .unwrap()
                    .borrow()
                    .line_idx()
                    - 1
            }
        }
    }
}

impl<T: Line> std::fmt::Display for CEigen<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}~{} gap={} fx={:?}",
            self.lst[0].upgrade().unwrap().borrow().line_idx(),
            self.lst
                .last()
                .unwrap()
                .upgrade()
                .unwrap()
                .borrow()
                .line_idx(),
            self.gap,
            self.fx
        )
    }
}
