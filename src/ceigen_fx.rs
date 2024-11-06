use crate::CEigen;
use crate::EqualMode;
use crate::Handle;
use crate::LineType;
use crate::ToHandle;
use crate::{Direction, FxType, KlineDir, SegType};
use std::vec::Vec;

// 特征序列分型
#[derive(Debug)]
pub struct CEigenFx<T> {
    pub lv: SegType,
    pub dir: Direction, // 线段方向
    pub ele: [Option<CEigen<T>>; 3],
    pub lst: Vec<Handle<T>>,
    pub exclude_included: bool,
    pub kl_dir: KlineDir,
    pub last_evidence_bi: Option<Handle<T>>,
}

impl<T: LineType + ToHandle> CEigenFx<T> {
    pub fn new(dir: Direction, exclude_included: bool, lv: SegType) -> Self {
        CEigenFx {
            lv,
            dir,
            ele: [None, None, None],
            lst: vec![],
            exclude_included,
            kl_dir: if dir == Direction::Up {
                KlineDir::Up
            } else {
                KlineDir::Down
            },
            last_evidence_bi: None,
        }
    }

    fn treat_first_ele(&mut self, bi: Handle<T>) -> bool {
        self.ele[0] = Some(CEigen::new(bi, self.kl_dir));
        false
    }

    fn treat_second_ele(&mut self, bi: Handle<T>) -> bool {
        assert!(self.ele[0].is_some());
        let combine_dir = self.ele[0]
            .as_mut()
            .unwrap()
            .try_add(bi, self.exclude_included, None);
        if combine_dir != KlineDir::Combine {
            self.ele[1] = Some(CEigen::new(bi, self.kl_dir));
            if (self.is_up()
                && self.ele[1].as_ref().unwrap().high < self.ele[0].as_ref().unwrap().high)
                || (self.is_down()
                    && self.ele[1].as_ref().unwrap().low > self.ele[0].as_ref().unwrap().low)
            {
                return self.reset();
            }
        }
        false
    }

    fn treat_third_ele(&mut self, bi: Handle<T>) -> bool {
        assert!(self.ele[0].is_some());
        assert!(self.ele[1].is_some());

        self.last_evidence_bi = Some(bi);
        let allow_top_equal = if self.exclude_included {
            Some(if bi.is_down() {
                EqualMode::TopEqual
            } else {
                EqualMode::BottomEqual
            })
        } else {
            None
        };

        let combine_dir = self.ele[1]
            .as_mut()
            .unwrap()
            .try_add(bi, false, allow_top_equal);

        if combine_dir == KlineDir::Combine {
            return false;
        }

        self.ele[2] = Some(CEigen::new(bi, combine_dir));

        if !self.actual_break() {
            return self.reset();
        }

        self.update_fx(allow_top_equal);

        let fx = self.ele[1].as_ref().unwrap().fx_type;
        let is_fx = (self.is_up() && fx == FxType::Top) || (self.is_down() && fx == FxType::Bottom);
        if is_fx {
            true
        } else {
            self.reset()
        }
    }

    // 已完备
    fn check_fx(&self, exclude_include: bool, allow_top_equal: Option<EqualMode>) -> FxType {
        let k1 = self.ele[0].as_ref().unwrap();
        let k2 = self.ele[1].as_ref().unwrap();
        let k3 = self.ele[2].as_ref().unwrap();

        if exclude_include {
            if k1.high < k2.high && k3.high <= k2.high && k3.low < k2.low {
                if allow_top_equal == Some(EqualMode::TopEqual) || k3.high < k2.high {
                    return FxType::Top;
                }
            } else if k1.low > k2.low
                && k3.low >= k2.low
                && k3.high > k2.high
                && (allow_top_equal == Some(EqualMode::BottomEqual) || k3.low > k2.low)
            {
                return FxType::Bottom;
            }
        } else {
            if k1.high < k2.high && k3.high < k2.high && k1.low < k2.low && k3.low < k2.low {
                return FxType::Top;
            }
            if k1.high > k2.high && k3.high > k2.high && k1.low > k2.low && k3.low > k2.low {
                return FxType::Bottom;
            }
        }

        FxType::Unknown
    }

    // 已完备
    fn check_gap(&self) -> bool {
        let k1 = self.ele[0].as_ref().unwrap();
        let k2 = self.ele[1].as_ref().unwrap();

        (k2.fx_type == FxType::Top && k1.high < k2.low)
            || (k2.fx_type == FxType::Bottom && k1.low > k2.high)
    }
    // 已完备
    fn update_fx(&mut self, allow_top_equal: Option<EqualMode>) {
        let fx_type = self.check_fx(self.exclude_included, allow_top_equal);
        self.ele[1].as_mut().unwrap().fx_type = fx_type;
        let gap = self.check_gap();
        self.ele[1].as_mut().unwrap().gap = gap;
    }

    pub fn add(&mut self, bi: Handle<T>) -> bool {
        assert_ne!(bi.direction(), self.dir);
        self.lst.push(bi);
        if self.ele[0].is_none() {
            self.treat_first_ele(bi)
        } else if self.ele[1].is_none() {
            self.treat_second_ele(bi)
        } else if self.ele[2].is_none() {
            self.treat_third_ele(bi)
        } else {
            panic!("特征序列3个都找齐了还没处理!! ")
        }
    }

    fn reset(&mut self) -> bool {
        // TODO:
        let bi_tmp_list = self.lst[1..].to_vec();
        if self.exclude_included {
            self.clear();
            for bi in bi_tmp_list {
                if self.add(bi) {
                    return true;
                }
            }
        } else {
            assert!(self.ele[1].is_some());
            let ele2_begin_idx = self.ele[1].as_ref().unwrap().lst[0].index();
            self.ele[0] = self.ele[1].take();
            self.ele[1] = self.ele[2].take();
            self.ele[2] = None;
            self.lst = bi_tmp_list
                .into_iter()
                .filter(|bi| bi.index() >= ele2_begin_idx)
                .collect();
        }
        false
    }

    pub fn can_be_end(&mut self, bi_lst: &[T]) -> Option<bool> {
        assert!(self.ele[1].is_some());
        if self.ele[1].as_ref().unwrap().gap {
            assert!(self.ele[0].is_some());
            let end_bi_idx = self.get_peak_bi_idx();
            let thred_value = bi_lst[end_bi_idx].get_end_val();
            let break_thred = if self.is_up() {
                self.ele[0].as_ref().unwrap().low
            } else {
                self.ele[0].as_ref().unwrap().high
            };
            self.find_revert_fx(bi_lst, end_bi_idx + 2, thred_value, break_thred)
        } else {
            Some(true)
        }
    }

    fn is_down(&self) -> bool {
        self.dir == Direction::Down
    }

    fn is_up(&self) -> bool {
        self.dir == Direction::Up
    }

    pub fn get_peak_bi_idx(&self) -> usize {
        assert!(self.ele[1].is_some());
        self.ele[1].as_ref().unwrap().get_peak_bi_idx()
    }

    pub fn all_bi_is_sure(&self) -> bool {
        assert!(self.last_evidence_bi.is_some());
        for bi in &self.lst {
            if !bi.is_sure() {
                return false;
            }
        }
        self.last_evidence_bi.unwrap().is_sure()
    }

    pub fn clear(&mut self) {
        self.ele = [None, None, None];
        self.lst = vec![];
    }

    fn actual_break(&mut self) -> bool {
        if !self.exclude_included {
            return true;
        }
        assert!(self.ele[2].is_some() && self.ele[1].is_some());
        if (self.is_up()
            && self.ele[2].as_ref().unwrap().low
                < self.ele[1].as_ref().unwrap().lst.last().unwrap().low())
            || (self.is_down()
                && self.ele[2].as_ref().unwrap().high
                    > self.ele[1].as_ref().unwrap().lst.last().unwrap().high())
        {
            return true;
        }
        assert_eq!(self.ele[2].as_ref().unwrap().lst.len(), 1);
        let ele2_bi = &self.ele[2].as_ref().unwrap().lst[0];
        if let Some(ref next) = ele2_bi.next() {
            if let Some(ref next_next) = next.next() {
                if ele2_bi.is_down() && next_next.low() < ele2_bi.low() {
                    self.last_evidence_bi = Some(*next_next);
                    return true;
                }
                if ele2_bi.is_up() && next_next.high() > ele2_bi.high() {
                    self.last_evidence_bi = Some(*next_next);
                    return true;
                }
            }
        }
        false
    }

    fn find_revert_fx(
        &mut self,
        bi_list: &[T],
        begin_idx: usize,
        thred_value: f64,
        break_thred: f64,
    ) -> Option<bool> {
        const COMMON_Combine: bool = true;

        let first_bi_dir = bi_list[begin_idx].direction();
        let mut egien_fx = CEigenFx::new(first_bi_dir.flip(), !COMMON_Combine, self.lv);
        for bi in bi_list.iter().skip(begin_idx).step_by(2) {
            if egien_fx.add(bi.to_handle()) {
                if COMMON_Combine {
                    return Some(true);
                }
                loop {
                    let _test = egien_fx.can_be_end(bi_list);
                    if _test.is_none() || _test == Some(true) {
                        self.last_evidence_bi = Some(bi.to_handle());
                        return _test;
                    }
                    if !egien_fx.reset() {
                        break;
                    }
                }
            }
            if (bi.is_down() && bi.low() < thred_value) || (bi.is_up() && bi.high() > thred_value) {
                return Some(false);
            }

            if egien_fx.ele[1].is_some()
                && ((bi.is_down() && egien_fx.ele[1].as_ref().unwrap().high > break_thred)
                    || (bi.is_up() && egien_fx.ele[1].as_ref().unwrap().low < break_thred))
            {
                return Some(true);
            }
        }
        None
    }
}
