use crate::Bi::Bi::CBi;
use crate::Bi::BiList::CBiList;
use crate::Common::CEnum::{BiDir, FxType, KlineDir, SegType};
use crate::Common::ChanException::{CChanException, ErrCode};
use crate::Common::FuncUtil::revert_bi_dir;
use crate::Seg::Eigen::CEigen;
use std::cell::RefCell;
use std::rc::Rc;

pub struct CEigenFX {
    lv: SegType,
    dir: BiDir,
    ele: [Option<Rc<RefCell<CEigen>>>; 3],
    lst: Vec<Rc<RefCell<CBi>>>,
    exclude_included: bool,
    kl_dir: KlineDir,
    last_evidence_bi: Option<Rc<RefCell<CBi>>>,
}

impl CEigenFX {
    pub fn new(dir: BiDir, exclude_included: bool, lv: SegType) -> Self {
        CEigenFX {
            lv,
            dir,
            ele: [None, None, None],
            lst: Vec::new(),
            exclude_included,
            kl_dir: if dir == BiDir::Up {
                KlineDir::Up
            } else {
                KlineDir::Down
            },
            last_evidence_bi: None,
        }
    }

    fn treat_first_ele(&mut self, bi: Rc<RefCell<CBi>>) -> bool {
        self.ele[0] = Some(Rc::new(RefCell::new(CEigen::new(bi, self.kl_dir))));
        false
    }

    fn treat_second_ele(&mut self, bi: Rc<RefCell<CBi>>) -> bool {
        let ele0 = self.ele[0].as_ref().unwrap();
        let combine_dir = ele0
            .borrow_mut()
            .try_add(&bi, Some(self.exclude_included), None);
        if combine_dir != KlineDir::Combine {
            self.ele[1] = Some(Rc::new(RefCell::new(CEigen::new(bi, self.kl_dir))));
            if (self.is_up()
                && self.ele[1].as_ref().unwrap().borrow().high() < ele0.borrow().high())
                || (self.is_down()
                    && self.ele[1].as_ref().unwrap().borrow().low() > ele0.borrow().low())
            {
                return self.reset();
            }
        }
        false
    }

    fn treat_third_ele(&mut self, bi: Rc<RefCell<CBi>>) -> bool {
        self.last_evidence_bi = Some(bi.clone());
        let allow_top_equal = if self.exclude_included {
            Some(if bi.borrow().is_down() { 1 } else { -1 })
        } else {
            None
        };
        let combine_dir =
            self.ele[1]
                .as_ref()
                .unwrap()
                .borrow_mut()
                .try_add(&bi, None, allow_top_equal);
        if combine_dir == KlineDir::Combine {
            return false;
        }
        self.ele[2] = Some(Rc::new(RefCell::new(CEigen::new(bi, combine_dir))));
        if !self.actual_break() {
            return self.reset();
        }
        self.ele[1].as_ref().unwrap().borrow_mut().update_fx(
            &self.ele[0].as_ref().unwrap().borrow(),
            &self.ele[2].as_ref().unwrap().borrow(),
            self.exclude_included,
            allow_top_equal,
        );
        let fx = self.ele[1].as_ref().unwrap().borrow().fx();
        let is_fx = (self.is_up() && fx == FxType::Top) || (self.is_down() && fx == FxType::Bottom);
        if is_fx {
            true
        } else {
            self.reset()
        }
    }

    pub fn add(&mut self, bi: Rc<RefCell<CBi>>) -> bool {
        assert!(bi.borrow().dir != self.dir);
        self.lst.push(bi.clone());
        if self.ele[0].is_none() {
            self.treat_first_ele(bi)
        } else if self.ele[1].is_none() {
            self.treat_second_ele(bi)
        } else if self.ele[2].is_none() {
            self.treat_third_ele(bi)
        } else {
            panic!(
                "特征序列3个都找齐了还没处理!! 当前笔:{},当前:{}",
                bi.borrow().idx,
                self.to_string()
            );
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
            let ele2_begin_idx = self.ele[1].as_ref().unwrap().borrow().lst()[0].borrow().idx;
            self.ele[0] = self.ele[1].take();
            self.ele[1] = self.ele[2].take();
            self.ele[2] = None;
            self.lst = bi_tmp_list
                .into_iter()
                .filter(|bi| bi.borrow().idx >= ele2_begin_idx)
                .collect();
        }
        false
    }

    pub fn can_be_end(&mut self, bi_lst: &CBiList) -> Option<bool> {
        if self.ele[1].as_ref().unwrap().borrow().gap {
            let end_bi_idx = self.get_peak_bi_idx();
            let thred_value = bi_lst[end_bi_idx].borrow().get_end_val();
            let break_thred = if self.is_up() {
                self.ele[0].as_ref().unwrap().borrow().low()
            } else {
                self.ele[0].as_ref().unwrap().borrow().high()
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

    pub fn get_peak_bi_idx(&self) -> i32 {
        self.ele[1].as_ref().unwrap().borrow().get_peak_bi_idx()
    }

    pub fn all_bi_is_sure(&self) -> bool {
        self.lst.iter().all(|bi| bi.borrow().is_sure)
            && self.last_evidence_bi.as_ref().unwrap().borrow().is_sure
    }

    pub fn clear(&mut self) {
        self.ele = [None, None, None];
        self.lst.clear();
    }

    fn actual_break(&mut self) -> bool {
        if !self.exclude_included {
            return true;
        }
        let ele1 = self.ele[1].as_ref().unwrap().borrow();
        let ele2 = self.ele[2].as_ref().unwrap().borrow();
        if (self.is_up() && ele2.low() < ele1.lst().last().unwrap().borrow()._low())
            || (self.is_down() && ele2.high() > ele1.lst().last().unwrap().borrow()._high())
        {
            return true;
        }
        assert_eq!(ele2.lst().len(), 1);
        let ele2_bi = &ele2.lst()[0];
        if let Some(next) = &ele2_bi.borrow().next {
            if let Some(next_next) = &next.borrow().next {
                if ele2_bi.borrow().is_down() && next_next.borrow()._low() < ele2_bi.borrow()._low()
                {
                    self.last_evidence_bi = Some(next_next.clone());
                    return true;
                } else if ele2_bi.borrow().is_up()
                    && next_next.borrow()._high() > ele2_bi.borrow()._high()
                {
                    self.last_evidence_bi = Some(next_next.clone());
                    return true;
                }
            }
        }
        false
    }

    fn find_revert_fx(
        &mut self,
        bi_list: &CBiList,
        begin_idx: i32,
        thred_value: f64,
        break_thred: f64,
    ) -> Option<bool> {
        const COMMON_COMBINE: bool = true;
        let first_bi_dir = bi_list[begin_idx as usize].borrow().dir;
        let mut eigen_fx = CEigenFX::new(revert_bi_dir(first_bi_dir), !COMMON_COMBINE, self.lv);
        for bi in bi_list.iter().skip(begin_idx as usize).step_by(2) {
            if eigen_fx.add(bi.clone()) {
                if COMMON_COMBINE {
                    return Some(true);
                }
                loop {
                    match eigen_fx.can_be_end(bi_list) {
                        Some(true) | None => {
                            self.last_evidence_bi = Some(bi.clone());
                            return eigen_fx.can_be_end(bi_list);
                        }
                        _ if !eigen_fx.reset() => break,
                        _ => {}
                    }
                }
            }
            if (bi.borrow().is_down() && bi.borrow()._low() < thred_value)
                || (bi.borrow().is_up() && bi.borrow()._high() > thred_value)
            {
                return Some(false);
            }
            if let Some(ele1) = &eigen_fx.ele[1] {
                if (bi.borrow().is_down() && ele1.borrow().high() > break_thred)
                    || (bi.borrow().is_up() && ele1.borrow().low() < break_thred)
                {
                    return Some(true);
                }
            }
        }
        None
    }
}

impl std::fmt::Display for CEigenFX {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let t: Vec<String> = self
            .ele
            .iter()
            .map(|ele| {
                if let Some(e) = ele {
                    e.borrow()
                        .lst()
                        .iter()
                        .map(|b| b.borrow().idx.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                } else {
                    String::from("[]")
                }
            })
            .collect();
        write!(f, "{}", t.join(" | "))
    }
}
