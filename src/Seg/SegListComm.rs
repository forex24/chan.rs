use crate::Bi::Bi::CBi;
use crate::Bi::BiList::CBiList;
use crate::Common::CEnum::{BiDir, LeftSegMethod, SegType};
use crate::Common::ChanException::{CChanException, ErrCode};
use crate::Seg::Seg::CSeg;
use crate::Seg::SegConfig::CSegConfig;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

pub struct CSegListComm<SUB_LINE_TYPE> {
    pub lst: Vec<Rc<RefCell<CSeg<SUB_LINE_TYPE>>>>,
    pub lv: SegType,
    pub config: CSegConfig,
    _phantom: PhantomData<SUB_LINE_TYPE>,
}

impl<SUB_LINE_TYPE> CSegListComm<SUB_LINE_TYPE> {
    pub fn new(seg_config: Option<CSegConfig>, lv: SegType) -> Self {
        let mut seg_list = CSegListComm {
            lst: Vec::new(),
            lv,
            config: seg_config.unwrap_or_default(),
            _phantom: PhantomData,
        };
        seg_list.do_init();
        seg_list
    }

    pub fn do_init(&mut self) {
        self.lst.clear();
    }

    pub fn left_bi_break(&self, bi_lst: &CBiList) -> bool {
        if self.lst.is_empty() {
            return false;
        }
        let last_seg_end_bi = &self.lst.last().unwrap().borrow().end_bi;
        for bi in bi_lst
            .iter()
            .skip(last_seg_end_bi.borrow().idx as usize + 1)
        {
            if last_seg_end_bi.borrow().is_up()
                && bi.borrow()._high() > last_seg_end_bi.borrow()._high()
            {
                return true;
            } else if last_seg_end_bi.borrow().is_down()
                && bi.borrow()._low() < last_seg_end_bi.borrow()._low()
            {
                return true;
            }
        }
        false
    }

    pub fn collect_first_seg(&mut self, bi_lst: &CBiList) -> Result<(), CChanException> {
        if bi_lst.len() < 3 {
            return Ok(());
        }
        match self.config.left_method {
            LeftSegMethod::Peak => {
                let _high = bi_lst
                    .iter()
                    .map(|bi| bi.borrow()._high())
                    .fold(f64::NEG_INFINITY, f64::max);
                let _low = bi_lst
                    .iter()
                    .map(|bi| bi.borrow()._low())
                    .fold(f64::INFINITY, f64::min);
                if (_high - bi_lst[0].borrow().get_begin_val()).abs()
                    >= (_low - bi_lst[0].borrow().get_begin_val()).abs()
                {
                    let peak_bi = find_peak_bi(bi_lst, true);
                    if let Some(peak_bi) = peak_bi {
                        self.add_new_seg(
                            bi_lst,
                            peak_bi.borrow().idx,
                            false,
                            Some(BiDir::Up),
                            false,
                            "0seg_find_high",
                        )?;
                    }
                } else {
                    let peak_bi = find_peak_bi(bi_lst, false);
                    if let Some(peak_bi) = peak_bi {
                        self.add_new_seg(
                            bi_lst,
                            peak_bi.borrow().idx,
                            false,
                            Some(BiDir::Down),
                            false,
                            "0seg_find_low",
                        )?;
                    }
                }
                self.collect_left_as_seg(bi_lst)?;
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
                    bi_lst.last().unwrap().borrow().idx,
                    false,
                    Some(_dir),
                    false,
                    "0seg_collect_all",
                )?;
            }
            _ => {
                return Err(CChanException::new(
                    format!("unknown seg left_method = {:?}", self.config.left_method),
                    ErrCode::ParaError,
                ))
            }
        }
        Ok(())
    }

    pub fn collect_left_seg_peak_method(
        &mut self,
        last_seg_end_bi: Rc<RefCell<SUB_LINE_TYPE>>,
        bi_lst: &CBiList,
    ) -> Result<(), CChanException> {
        if last_seg_end_bi.borrow().is_down() {
            if let Some(peak_bi) =
                find_peak_bi(&bi_lst[last_seg_end_bi.borrow().idx as usize + 3..], true)
            {
                if peak_bi.borrow().idx - last_seg_end_bi.borrow().idx >= 3 {
                    self.add_new_seg(
                        bi_lst,
                        peak_bi.borrow().idx,
                        false,
                        Some(BiDir::Up),
                        true,
                        "collectleft_find_high",
                    )?;
                }
            }
        } else {
            if let Some(peak_bi) =
                find_peak_bi(&bi_lst[last_seg_end_bi.borrow().idx as usize + 3..], false)
            {
                if peak_bi.borrow().idx - last_seg_end_bi.borrow().idx >= 3 {
                    self.add_new_seg(
                        bi_lst,
                        peak_bi.borrow().idx,
                        false,
                        Some(BiDir::Down),
                        true,
                        "collectleft_find_low",
                    )?;
                }
            }
        }
        let last_seg_end_bi = self.lst.last().unwrap().borrow().end_bi.clone();
        self.collect_left_as_seg(bi_lst)?;
        Ok(())
    }

    pub fn collect_segs(&mut self, bi_lst: &CBiList) -> Result<(), CChanException> {
        let last_bi = bi_lst.last().unwrap();
        let last_seg_end_bi = self.lst.last().unwrap().borrow().end_bi.clone();
        if last_bi.borrow().idx - last_seg_end_bi.borrow().idx < 3 {
            return Ok(());
        }
        if last_seg_end_bi.borrow().is_down()
            && last_bi.borrow().get_end_val() <= last_seg_end_bi.borrow().get_end_val()
        {
            if let Some(peak_bi) =
                find_peak_bi(&bi_lst[last_seg_end_bi.borrow().idx as usize + 3..], true)
            {
                self.add_new_seg(
                    bi_lst,
                    peak_bi.borrow().idx,
                    false,
                    Some(BiDir::Up),
                    true,
                    "collectleft_find_high_force",
                )?;
                self.collect_left_seg(bi_lst)?;
            }
        } else if last_seg_end_bi.borrow().is_up()
            && last_bi.borrow().get_end_val() >= last_seg_end_bi.borrow().get_end_val()
        {
            if let Some(peak_bi) =
                find_peak_bi(&bi_lst[last_seg_end_bi.borrow().idx as usize + 3..], false)
            {
                self.add_new_seg(
                    bi_lst,
                    peak_bi.borrow().idx,
                    false,
                    Some(BiDir::Down),
                    true,
                    "collectleft_find_low_force",
                )?;
                self.collect_left_seg(bi_lst)?;
            }
        } else if self.config.left_method == LeftSegMethod::All {
            self.collect_left_as_seg(bi_lst)?;
        } else if self.config.left_method == LeftSegMethod::Peak {
            self.collect_left_seg_peak_method(last_seg_end_bi, bi_lst)?;
        } else {
            return Err(CChanException::new(
                format!("unknown seg left_method = {:?}", self.config.left_method),
                ErrCode::ParaError,
            ));
        }
        Ok(())
    }

    pub fn collect_left_seg(&mut self, bi_lst: &CBiList) -> Result<(), CChanException> {
        if self.lst.is_empty() {
            self.collect_first_seg(bi_lst)?;
        } else {
            self.collect_segs(bi_lst)?;
        }
        Ok(())
    }

    pub fn collect_left_as_seg(&mut self, bi_lst: &CBiList) -> Result<(), CChanException> {
        let last_bi = bi_lst.last().unwrap();
        let last_seg_end_bi = self.lst.last().unwrap().borrow().end_bi.clone();
        if last_seg_end_bi.borrow().idx + 1 >= bi_lst.len() as i32 {
            return Ok(());
        }
        if last_seg_end_bi.borrow().dir == last_bi.borrow().dir {
            self.add_new_seg(
                bi_lst,
                last_bi.borrow().idx - 1,
                false,
                None,
                true,
                "collect_left_1",
            )?;
        } else {
            self.add_new_seg(
                bi_lst,
                last_bi.borrow().idx,
                false,
                None,
                true,
                "collect_left_0",
            )?;
        }
        Ok(())
    }

    pub fn try_add_new_seg(
        &mut self,
        bi_lst: &CBiList,
        end_bi_idx: i32,
        is_sure: bool,
        seg_dir: Option<BiDir>,
        split_first_seg: bool,
        reason: &str,
    ) -> Result<(), CChanException> {
        if self.lst.is_empty() && split_first_seg && end_bi_idx >= 3 {
            if let Some(peak_bi) = find_peak_bi(
                &bi_lst[..end_bi_idx as usize]
                    .iter()
                    .rev()
                    .cloned()
                    .collect::<Vec<_>>(),
                bi_lst[end_bi_idx as usize].borrow().is_down(),
            ) {
                if (peak_bi.borrow().is_down()
                    && (peak_bi.borrow()._low() < bi_lst[0].borrow()._low()
                        || peak_bi.borrow().idx == 0))
                    || (peak_bi.borrow().is_up()
                        && (peak_bi.borrow()._high() > bi_lst[0].borrow()._high()
                            || peak_bi.borrow().idx == 0))
                {
                    self.add_new_seg(
                        bi_lst,
                        peak_bi.borrow().idx,
                        false,
                        Some(peak_bi.borrow().dir),
                        true,
                        "split_first_1st",
                    )?;
                    self.add_new_seg(bi_lst, end_bi_idx, false, None, true, "split_first_2nd")?;
                    return Ok(());
                }
            }
        }
        let bi1_idx = if self.lst.is_empty() {
            0
        } else {
            self.lst.last().unwrap().borrow().end_bi.borrow().idx + 1
        };
        let bi1 = bi_lst[bi1_idx as usize].clone();
        let bi2 = bi_lst[end_bi_idx as usize].clone();
        let new_seg = Rc::new(RefCell::new(CSeg::new(
            self.lst.len() as i32,
            bi1,
            bi2,
            is_sure,
            seg_dir,
            reason,
        )?));

        if self.lst.len() >= 2 {
            let last_seg = self.lst.last().unwrap().clone();
            last_seg.borrow_mut().next = Some(new_seg.clone());
            new_seg.borrow_mut().pre = Some(last_seg);
        }
        new_seg
            .borrow_mut()
            .update_bi_list(bi_lst, bi1_idx as usize, end_bi_idx as usize);
        self.lst.push(new_seg);
        Ok(())
    }

    pub fn add_new_seg(
        &mut self,
        bi_lst: &CBiList,
        end_bi_idx: i32,
        is_sure: bool,
        seg_dir: Option<BiDir>,
        split_first_seg: bool,
        reason: &str,
    ) -> Result<bool, CChanException> {
        match self.try_add_new_seg(
            bi_lst,
            end_bi_idx,
            is_sure,
            seg_dir,
            split_first_seg,
            reason,
        ) {
            Ok(_) => Ok(true),
            Err(e) => {
                if e.errcode == ErrCode::SegEndValueErr && self.lst.is_empty() {
                    Ok(false)
                } else {
                    Err(e)
                }
            }
        }
    }

    pub fn exist_sure_seg(&self) -> bool {
        self.lst.iter().any(|seg| seg.borrow().is_sure)
    }
}

impl<SUB_LINE_TYPE> std::ops::Index<usize> for CSegListComm<SUB_LINE_TYPE> {
    type Output = Rc<RefCell<CSeg<SUB_LINE_TYPE>>>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.lst[index]
    }
}

impl<SUB_LINE_TYPE> std::ops::IndexMut<usize> for CSegListComm<SUB_LINE_TYPE> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.lst[index]
    }
}

impl<SUB_LINE_TYPE> std::ops::Deref for CSegListComm<SUB_LINE_TYPE> {
    type Target = Vec<Rc<RefCell<CSeg<SUB_LINE_TYPE>>>>;

    fn deref(&self) -> &Self::Target {
        &self.lst
    }
}

impl<SUB_LINE_TYPE> std::ops::DerefMut for CSegListComm<SUB_LINE_TYPE> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.lst
    }
}

pub fn find_peak_bi(bi_lst: &[Rc<RefCell<CBi>>], is_high: bool) -> Option<Rc<RefCell<CBi>>> {
    let mut peak_val = if is_high {
        f64::NEG_INFINITY
    } else {
        f64::INFINITY
    };
    let mut peak_bi = None;
    for bi in bi_lst {
        let bi_ref = bi.borrow();
        if (is_high && bi_ref.get_end_val() >= peak_val && bi_ref.is_up())
            || (!is_high && bi_ref.get_end_val() <= peak_val && bi_ref.is_down())
        {
            if let Some(pre) = &bi_ref.pre {
                if let Some(pre_pre) = &pre.borrow().pre {
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
