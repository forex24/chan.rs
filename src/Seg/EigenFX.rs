// eigen_fx.rs

use std::cell::RefCell;
use std::rc::Rc;

use crate::Common::{
    func_util::revert_bi_dir,
    types::Handle,
    CEnum::{BiDir, EqualMode, FxType, KLineDir, SegType},
};

use super::{linetype::Line, Eigen::CEigen};

//#[derive(Debug, Clone)]
pub struct CEigenFX<T> {
    pub lv: SegType,
    pub dir: BiDir,
    pub ele: [Option<Handle<CEigen<T>>>; 3],
    pub lst: Vec<Handle<T>>,
    pub exclude_included: bool,
    pub kl_dir: KLineDir,
    pub last_evidence_bi: Option<Handle<T>>,
}

impl<T: Line> CEigenFX<T> {
    pub fn new(dir: BiDir, exclude_included: bool, lv: SegType) -> Self {
        let kl_dir = if dir == BiDir::Up {
            KLineDir::Up
        } else {
            KLineDir::Down
        };

        Self {
            lv,
            dir,
            ele: [None, None, None],
            lst: Vec::new(),
            exclude_included,
            kl_dir,
            last_evidence_bi: None,
        }
    }

    pub fn treat_first_ele(&mut self, bi: Handle<T>) -> bool {
        self.ele[0] = Some(Rc::new(RefCell::new(
            CEigen::new(&bi, self.kl_dir).unwrap(),
        )));
        false
    }

    pub fn treat_second_ele(&mut self, bi: Handle<T>) -> bool {
        assert!(self.ele[0].is_some());
        let combine_dir =
            self.ele[0]
                .as_ref()
                .unwrap()
                .borrow_mut()
                .try_add(&bi, self.exclude_included, None);

        if combine_dir != KLineDir::Combine {
            self.ele[1] = Some(Rc::new(RefCell::new(
                CEigen::new(&bi, self.kl_dir).unwrap(),
            )));
            let ele1 = self.ele[1].as_ref().unwrap();

            if (self.is_up() && ele1.borrow().high < self.ele[0].as_ref().unwrap().borrow().high)
                || (self.is_down()
                    && ele1.borrow().low > self.ele[0].as_ref().unwrap().borrow().low)
            {
                return self.reset();
            }
        }
        false
    }

    pub fn check_fx(&self, exclude_included: bool, allow_top_equal: Option<EqualMode>) -> FxType {
        let k1 = self.ele[0].as_ref().unwrap().borrow();
        let k2 = self.ele[1].as_ref().unwrap().borrow();
        let k3 = self.ele[2].as_ref().unwrap().borrow();
        if exclude_included {
            if k1.high < k2.high && k3.high <= k2.high && k3.low < k2.low {
                if allow_top_equal == Some(EqualMode::TopEqual) || k3.high < k2.high {
                    return FxType::Top;
                }
            } else if k3.high > k2.high
                && k1.low > k2.low
                && k3.low >= k2.low
                && (allow_top_equal == Some(EqualMode::BottomEqual) || k3.low > k2.low)
            {
                return FxType::Bottom;
            }
        } else if k1.high < k2.high && k3.high < k2.high && k1.low < k2.low && k3.low < k2.low {
            return FxType::Top;
        } else if k1.high > k2.high && k3.high > k2.high && k1.low > k2.low && k3.low > k2.low {
            return FxType::Bottom;
        }
        FxType::Unknown
    }

    // 已完备
    fn check_gap(&self) -> bool {
        let k1 = self.ele[0].as_ref().unwrap().borrow();
        let k2 = self.ele[1].as_ref().unwrap().borrow();

        (k2.fx == FxType::Top && k1.high < k2.low) || (k2.fx == FxType::Bottom && k1.low > k2.high)
    }

    // 已完备
    fn update_fx(&mut self, allow_top_equal: Option<EqualMode>) {
        let fx_type = self.check_fx(self.exclude_included, allow_top_equal);
        self.ele[1].as_mut().unwrap().borrow_mut().fx = fx_type;
        let gap = self.check_gap();
        self.ele[1].as_mut().unwrap().borrow_mut().gap = gap;
    }

    pub fn treat_third_ele(&mut self, bi: Handle<T>) -> bool {
        assert!(self.ele[0].is_some());
        assert!(self.ele[1].is_some());

        self.last_evidence_bi = Some(Rc::clone(&bi));

        let allow_top_equal = if self.exclude_included {
            if bi.borrow().line_is_down() {
                Some(EqualMode::TopEqual)
            } else {
                Some(EqualMode::BottomEqual)
            }
        } else {
            None
        };

        let combine_dir = self.ele[1].as_ref().unwrap().borrow_mut().try_add(
            &bi,
            self.exclude_included,
            allow_top_equal,
        );

        if combine_dir == KLineDir::Combine {
            return false;
        }

        self.ele[2] = Some(Rc::new(RefCell::new(
            CEigen::new(&bi, combine_dir).unwrap(),
        )));

        if !self.actual_break() {
            return self.reset();
        }

        self.update_fx(allow_top_equal);

        //let ele1 = self.ele[1].as_ref().unwrap();
        //let ele0 = self.ele[0].as_ref().unwrap();
        //let ele2 = self.ele[2].as_ref().unwrap();
        //ele1.borrow_mut()
        //    .update_fx(&ele0, &ele2, self.exclude_included, allow_top_equal);

        let fx = self.ele[1].as_ref().unwrap().borrow().fx;
        let is_fx = (self.is_up() && fx == FxType::Top) || (self.is_down() && fx == FxType::Bottom);

        if is_fx {
            true
        } else {
            self.reset()
        }
    }

    pub fn add(&mut self, bi: Handle<T>) -> bool {
        assert!(bi.borrow().line_dir() != self.dir);
        self.lst.push(bi.clone());

        if self.ele[0].is_none() {
            self.treat_first_ele(bi)
        } else if self.ele[1].is_none() {
            self.treat_second_ele(bi)
        } else if self.ele[2].is_none() {
            self.treat_third_ele(bi)
        } else {
            panic!("特征序列3个都找齐了还没处理!!");
        }
    }

    pub fn reset(&mut self) -> bool {
        let bi_tmp_list: Vec<_> = self.lst[1..].to_vec();

        if self.exclude_included {
            self.clear();
            for bi in bi_tmp_list {
                if self.add(bi) {
                    return true;
                }
            }
        } else {
            assert!(self.ele[1].is_some());

            let ele1_begin_idx = self.ele[1].as_ref().unwrap().borrow().lst[0]
                .borrow()
                .line_idx();

            self.ele[0] = self.ele[1].take();
            self.ele[1] = self.ele[2].take();
            self.ele[2] = None;

            self.lst = bi_tmp_list
                .into_iter()
                .filter(|bi| bi.borrow().line_idx() >= ele1_begin_idx)
                .collect();
        }
        false
    }

    pub fn can_be_end(&mut self, bi_lst: &[Handle<T>]) -> Option<bool> {
        assert!(self.ele[1].is_some());
        let ele1_gap = self.ele[1].as_ref().unwrap().borrow().gap;
        if ele1_gap {
            assert!(self.ele[0].is_some());
            let end_bi_idx = self.get_peak_bi_idx();
            let thred_value = bi_lst[end_bi_idx].borrow().line_get_end_val();
            let break_thred = if self.is_up() {
                self.ele[0].as_ref().unwrap().borrow().low
            } else {
                self.ele[0].as_ref().unwrap().borrow().high
            };
            self.find_revert_fx(bi_lst, end_bi_idx + 2, thred_value, break_thred)
        } else {
            Some(true)
        }
    }

    pub fn is_down(&self) -> bool {
        self.dir == BiDir::Down
    }

    pub fn is_up(&self) -> bool {
        self.dir == BiDir::Up
    }

    pub fn get_peak_bi_idx(&self) -> usize {
        assert!(self.ele[1].is_some());
        self.ele[1].as_ref().unwrap().borrow().get_peak_bi_idx()
    }

    pub fn all_bi_is_sure(&self) -> bool {
        assert!(self.last_evidence_bi.is_some());

        for bi in &self.lst {
            if !bi.borrow().line_is_sure() {
                return false;
            }
        }

        self.last_evidence_bi
            .as_ref()
            .unwrap()
            .borrow()
            .line_is_sure()
    }

    pub fn clear(&mut self) {
        self.ele = [None, None, None];
        self.lst.clear();
    }

    pub fn actual_break(&mut self) -> bool {
        if !self.exclude_included {
            return true;
        }

        assert!(self.ele[2].is_some() && self.ele[1].is_some());

        let ele2 = self.ele[2].as_ref().unwrap().borrow();
        let ele1 = self.ele[1].as_ref().unwrap().borrow();

        if (self.is_up() && ele2.low < ele1.lst.last().unwrap().borrow().line_low())
            || (self.is_down() && ele2.high > ele1.lst.last().unwrap().borrow().line_high())
        {
            return true;
        }

        assert_eq!(ele2.lst.len(), 1);

        let ele2_bi = &ele2.lst[0];
        let ele2_bi_ref = ele2_bi.borrow();

        if let Some(next) = &ele2_bi_ref.line_next() {
            if let Some(next_next) = &next.borrow().line_next() {
                if ele2_bi_ref.line_is_down()
                    && next_next.borrow().line_low() < ele2_bi_ref.line_low()
                {
                    self.last_evidence_bi = Some(next_next.clone());
                    return true;
                } else if ele2_bi_ref.line_is_up()
                    && next_next.borrow().line_high() > ele2_bi_ref.line_high()
                {
                    self.last_evidence_bi = Some(next_next.clone());
                    return true;
                }
            }
        }
        false
    }

    pub fn find_revert_fx(
        &mut self,
        bi_list: &[Handle<T>],
        begin_idx: usize,
        thred_value: f64,
        break_thred: f64,
    ) -> Option<bool> {
        const COMMON_COMBINE: bool = true;

        let first_bi_dir = bi_list[begin_idx].borrow().line_dir();
        let mut eigen_fx =
            CEigenFX::<T>::new(revert_bi_dir(&first_bi_dir), !COMMON_COMBINE, self.lv);

        for bi in bi_list.iter().skip(begin_idx).step_by(2) {
            if eigen_fx.add(Rc::clone(bi)) {
                if COMMON_COMBINE {
                    return Some(true);
                }

                loop {
                    match eigen_fx.can_be_end(bi_list) {
                        Some(true) | None => {
                            self.last_evidence_bi = Some(Rc::clone(bi));
                            return eigen_fx.can_be_end(bi_list);
                        }
                        Some(false) => {
                            if !eigen_fx.reset() {
                                break;
                            }
                        }
                    }
                }
            }

            let bi_ref = bi.borrow();

            if (bi_ref.line_is_down() && bi_ref.line_low() < thred_value)
                || (bi_ref.line_is_up() && bi_ref.line_high() > thred_value)
            {
                return Some(false);
            }

            if eigen_fx.ele[1].is_some() {
                let ele1 = eigen_fx.ele[1].as_ref().unwrap().borrow();
                if (bi_ref.line_is_down() && ele1.high > break_thred)
                    || (bi_ref.line_is_up() && ele1.low < break_thred)
                {
                    return Some(true);
                }
            }
        }
        None
    }
}

impl<T: Line> std::fmt::Display for CEigenFX<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let elements: Vec<String> = self
            .ele
            .iter()
            .map(|ele| {
                if let Some(e) = ele {
                    e.borrow()
                        .lst
                        .iter()
                        .map(|b| b.borrow().line_idx().to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                } else {
                    String::from("[]")
                }
            })
            .collect();

        write!(f, "{}", elements.join(" | "))
    }
}
