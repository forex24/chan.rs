use std::cell::RefCell;
use std::ops::{Index, IndexMut};
use std::rc::Rc;

use crate::Common::types::Handle;
use crate::Common::CEnum::{BiDir, LeftSegMethod, SegType};
use crate::Common::ChanException::{CChanException, ErrCode};

use super::linetype::Line;
use super::Seg::CSeg;
use super::SegConfig::CSegConfig;

pub struct CSegListComm<T: Line> {
    pub lst: Vec<Handle<CSeg<T>>>,
    pub lv: SegType,
    pub config: CSegConfig,
}

impl<T: Line> CSegListComm<T> {
    pub fn new(seg_config: CSegConfig, lv: SegType) -> Self {
        Self {
            lst: Vec::new(),
            lv,
            config: seg_config,
        }
    }

    pub fn do_init(&mut self) {
        self.lst.clear();
    }

    pub fn len(&self) -> usize {
        self.lst.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lst.is_empty()
    }

    pub fn left_bi_break(&self, bi_lst: &[Handle<T>]) -> bool {
        if self.lst.is_empty() {
            return false;
        }
        let last_seg_end_bi = &self.lst.last().unwrap().borrow().end_bi;
        for bi in bi_lst.iter().skip(last_seg_end_bi.borrow().idx() + 1) {
            if last_seg_end_bi.borrow().is_up()
                && bi.borrow().high() > last_seg_end_bi.borrow().high()
            {
                return true;
            } else if last_seg_end_bi.borrow().is_down()
                && bi.borrow().low() < last_seg_end_bi.borrow().low()
            {
                return true;
            }
        }
        false
    }

    pub fn collect_first_seg(&mut self, bi_lst: &[Handle<T>]) {
        if bi_lst.len() < 3 {
            return;
        }
        match self.config.left_method {
            LeftSegMethod::Peak => {
                let _high = bi_lst
                    .iter()
                    .map(|bi| bi.borrow().high())
                    .fold(f64::MIN, f64::max);
                let _low = bi_lst
                    .iter()
                    .map(|bi| bi.borrow().low())
                    .fold(f64::MAX, f64::min);
                let first_val = bi_lst.first().unwrap().borrow().get_begin_val();

                if (_high - first_val).abs() >= (_low - first_val).abs() {
                    if let Some(peak_bi) = find_peak_bi(bi_lst.iter(), true) {
                        self.add_new_seg(
                            bi_lst,
                            peak_bi.borrow().idx() ,
                            false,
                            Some(BiDir::Up),
                            false,
                            "0seg_find_high",
                        );
                    }
                } else if let Some(peak_bi) = find_peak_bi(bi_lst.iter(), false) {
                    self.add_new_seg(
                        bi_lst,
                        peak_bi.borrow().idx() ,
                        false,
                        Some(BiDir::Down),
                        false,
                        "0seg_find_low",
                    );
                }
                self.collect_left_as_seg(bi_lst);
            }
            LeftSegMethod::All => {
                let _dir = if bi_lst.last().unwrap().borrow().get_end_val()
                    >= bi_lst[0].borrow().get_begin_val()
                {
                    BiDir::Up
                } else {
                    BiDir::Down
                };
                self.add_new_seg(
                    bi_lst,
                    bi_lst.last().unwrap().borrow().idx() ,
                    false,
                    Some(_dir),
                    false,
                    "0seg_collect_all",
                );
            }
            _ => panic!("unknown seg left_method = {:?}", self.config.left_method),
        }
    }

    pub fn collect_left_seg_peak_method(
        &mut self,
        last_seg_end_bi: Handle<T>,
        bi_lst: &[Handle<T>],
    ) {
        if last_seg_end_bi.borrow().is_down() {
            if let Some(peak_bi) = find_peak_bi(
                bi_lst[last_seg_end_bi.borrow().idx()  + 3..].iter(),
                true,
            ) {
                if peak_bi.borrow().idx()  - last_seg_end_bi.borrow().idx() >= 3 {
                    self.add_new_seg(
                        bi_lst,
                        peak_bi.borrow().idx() ,
                        false,
                        Some(BiDir::Up),
                        true,
                        "collectleft_find_high",
                    );
                }
            }
        } else if let Some(peak_bi) = find_peak_bi(
            bi_lst[last_seg_end_bi.borrow().idx()  + 3..].iter(),
            false,
        ) {
            if peak_bi.borrow().idx()  - last_seg_end_bi.borrow().idx() >= 3 {
                self.add_new_seg(
                    bi_lst,
                    peak_bi.borrow().idx() ,
                    false,
                    Some(BiDir::Down),
                    true,
                    "collectleft_find_low",
                );
            }
        }
        self.collect_left_as_seg(bi_lst);
    }

    pub fn collect_segs(&mut self, bi_lst: &[Handle<T>]) {
        let last_bi = bi_lst.last().unwrap();
        let last_seg_end_bi = self.lst.last().unwrap().borrow().end_bi.clone();

        if last_bi.borrow().idx()  - last_seg_end_bi.borrow().idx() < 3 {
            return;
        }

        if last_seg_end_bi.borrow().is_down()
            && last_bi.borrow().get_end_val() <= last_seg_end_bi.borrow().get_end_val()
        {
            if let Some(peak_bi) =
                find_peak_bi(bi_lst[last_seg_end_bi.borrow().idx() + 3..].iter(), true)
            {
                self.add_new_seg(
                    bi_lst,
                    peak_bi.borrow().idx() ,
                    false,
                    Some(BiDir::Up),
                    true,
                    "collectleft_find_high_force",
                );
                self.collect_left_seg(bi_lst);
            }
        } else if last_seg_end_bi.borrow().is_up()
            && last_bi.borrow().get_end_val() >= last_seg_end_bi.borrow().get_end_val()
        {
            if let Some(peak_bi) =
                find_peak_bi(bi_lst[last_seg_end_bi.borrow().idx() + 3..].iter(), false)
            {
                self.add_new_seg(
                    bi_lst,
                    peak_bi.borrow().idx() ,
                    false,
                    Some(BiDir::Down),
                    true,
                    "collectleft_find_low_force",
                );
                self.collect_left_seg(bi_lst);
            }
        } else if self.config.left_method == LeftSegMethod::All {
            self.collect_left_as_seg(bi_lst);
        } else if self.config.left_method == LeftSegMethod::Peak {
            self.collect_left_seg_peak_method(last_seg_end_bi, bi_lst);
        } else {
            panic!("unknown seg left_method = {:?}", self.config.left_method);
        }
    }

    pub fn collect_left_seg(&mut self, bi_lst: &[Handle<T>]) {
        if self.lst.is_empty() {
            self.collect_first_seg(bi_lst);
        } else {
            self.collect_segs(bi_lst);
        }
    }

    pub fn collect_left_as_seg(&mut self, bi_lst: &[Handle<T>]) {
        let last_bi = bi_lst.last().unwrap();
        let last_seg_end_bi = self.lst.last().unwrap().borrow().end_bi.clone();

        if last_seg_end_bi.borrow().idx() + 1 >= bi_lst.len() {
            return;
        }

        if last_seg_end_bi.borrow().dir() == last_bi.borrow().dir() {
            self.add_new_seg(
                bi_lst,
                last_bi.borrow().idx()  - 1,
                false,
                None,
                true,
                "collect_left_1",
            );
        } else {
            self.add_new_seg(
                bi_lst,
                last_bi.borrow().idx() ,
                false,
                None,
                true,
                "collect_left_0",
            );
        }
    }

    pub fn try_add_new_seg(
        &mut self,
        bi_lst: &[Handle<T>],
        end_bi_idx: usize,
        is_sure: bool,
        seg_dir: Option<BiDir>,
        split_first_seg: bool,
        reason: &str,
    ) -> Result<(), CChanException> {
        if self.lst.is_empty() && split_first_seg && end_bi_idx >= 3 {
            if let Some(peak_bi) = find_peak_bi(
                bi_lst[..end_bi_idx - 2].iter().rev(),
                bi_lst[end_bi_idx].borrow().is_down(),
            ) {
                let peak_bi_ref = peak_bi.borrow();
                if (peak_bi_ref.is_down()
                    && (peak_bi_ref.low() < bi_lst[0].borrow().low() || peak_bi_ref.idx() == 0))
                    || (peak_bi_ref.is_up()
                        && (peak_bi_ref.high() > bi_lst[0].borrow().high()
                            || peak_bi_ref.idx() == 0))
                {
                    // 要比第一笔开头还高/低（因为没有比较到）
                    self.add_new_seg(
                        bi_lst,
                        peak_bi_ref.idx() ,
                        false,
                        Some(peak_bi_ref.dir()),
                        true,
                        "split_first_1st",
                    );
                    self.add_new_seg(bi_lst, end_bi_idx, false, None, true, "split_first_2nd");
                    return Ok(());
                }
            }
        }

        let bi1_idx = if self.lst.is_empty() {
            0
        } else {
            self.lst.last().unwrap().borrow().end_bi.borrow().idx() + 1
        };
        let bi1 = bi_lst[bi1_idx].clone();
        let bi2 = bi_lst[end_bi_idx].clone();

        let new_seg = Rc::new(RefCell::new(CSeg::new(
            self.lst.len(),
            bi1,
            bi2,
            is_sure,
            seg_dir,
            reason,
        )?));

        if self.lst.len() >= 2 {
            let last_seg = self.lst.last().unwrap().clone();
            last_seg.borrow_mut().next = Some(Rc::clone(&new_seg));
            new_seg.borrow_mut().pre = Some(last_seg);
        }

        new_seg
            .borrow_mut()
            .update_bi_list(bi_lst, bi1_idx, end_bi_idx);
        self.lst.push(new_seg);

        Ok(())
    }

    pub fn add_new_seg(
        &mut self,
        bi_lst: &[Handle<T>],
        end_bi_idx: usize,
        is_sure: bool,
        seg_dir: Option<BiDir>,
        split_first_seg: bool,
        reason: &str,
    ) -> bool {
        match self.try_add_new_seg(
            bi_lst,
            end_bi_idx,
            is_sure,
            seg_dir,
            split_first_seg,
            reason,
        ) {
            Ok(_) => true,
            Err(e) => {
                if e.errcode == ErrCode::SegEndValueErr && self.lst.is_empty() {
                    false
                } else {
                    panic!("{}", e)
                }
            }
        }
    }

    pub fn exist_sure_seg(&self) -> bool {
        self.lst.iter().any(|seg| seg.borrow().is_sure)
    }

    pub fn update(&mut self, _bi_lst: &[Handle<T>]) {
        unimplemented!("update method must be implemented by derived struct");
    }
}

pub fn find_peak_bi<'a, T: Line + 'a, I>(bi_lst: I, is_high: bool) -> Option<Handle<T>>
where
    I: Iterator<Item = &'a Handle<T>>,
{
    let mut peak_val = if is_high { f64::MIN } else { f64::MAX };
    let mut peak_bi = None;

    for bi in bi_lst {
        let bi_ref = bi.borrow();
        if (is_high && bi_ref.get_end_val() >= peak_val && bi_ref.is_up())
            || (!is_high && bi_ref.get_end_val() <= peak_val && bi_ref.is_down())
        {
            if let Some(pre) = &bi_ref.pre() {
                if let Some(pre_pre) = &pre.borrow().pre() {
                    if (is_high && pre_pre.borrow().get_end_val() > bi_ref.get_end_val())
                        || (!is_high && pre_pre.borrow().get_end_val() < bi_ref.get_end_val())
                    {
                        continue;
                    }
                }
            }
            peak_val = bi_ref.get_end_val();
            peak_bi = Some(bi.clone());
        }
    }
    peak_bi
}

// 实现 Index 和 IndexMut traits 以支持索引访问
impl<T: Line> Index<usize> for CSegListComm<T> {
    type Output = Handle<CSeg<T>>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.lst[index]
    }
}

impl<T: Line> IndexMut<usize> for CSegListComm<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.lst[index]
    }
}

// 实现 IntoIterator 以支持迭代
impl<T: Line> IntoIterator for CSegListComm<T> {
    type Item = Handle<CSeg<T>>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.lst.into_iter()
    }
}
