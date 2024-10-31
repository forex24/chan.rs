use crate::Common::types::Handle;
use crate::Common::CEnum::{BiDir, LeftSegMethod, SegType};
use crate::Common::ChanException::{CChanException, ErrCode};
use crate::Seg2::Seg::CSeg;
use crate::Seg2::SegConfig::CSegConfig;
use std::cell::RefCell;
use std::rc::Rc;

use super::linetype::Line;
use super::EigenFX::CEigenFX;

pub struct CSegListComm<T> {
    pub lst: Vec<Handle<CSeg<T>>>,
    pub lv: SegType,
    pub config: CSegConfig,
}

impl<T: Line<T>> CSegListComm<T> {
    pub fn new(seg_config: Option<CSegConfig>, lv: SegType) -> Self {
        let mut seg_list = CSegListComm {
            lst: Vec::new(),
            lv,
            config: seg_config.unwrap_or_default(),
            //_phantom: PhantomData,
        };
        seg_list.do_init();
        seg_list
    }

    pub fn do_init(&mut self) {
        self.lst.clear();
    }

    pub fn left_bi_break(&self, bi_lst: &[Handle<T>]) -> bool {
        if self.lst.is_empty() {
            return false;
        }
        let last_seg_end_bi = &self.lst.last().unwrap().borrow().end_bi;
        for bi in bi_lst
            .iter()
            .skip(last_seg_end_bi.borrow().idx() as usize + 1)
        {
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

    pub fn collect_first_seg(&mut self, bi_lst: &[Handle<T>]) -> Result<(), CChanException> {
        if bi_lst.len() < 3 {
            return Ok(());
        }
        match self.config.left_method {
            LeftSegMethod::Peak => {
                let _high = bi_lst
                    .iter()
                    .map(|bi| bi.borrow().high())
                    .fold(f64::NEG_INFINITY, f64::max);
                let _low = bi_lst
                    .iter()
                    .map(|bi| bi.borrow().low())
                    .fold(f64::INFINITY, f64::min);
                if (_high - bi_lst[0].borrow().get_begin_val()).abs()
                    >= (_low - bi_lst[0].borrow().get_begin_val()).abs()
                {
                    let peak_bi = find_peak_bi(bi_lst, true);
                    if let Some(peak_bi) = peak_bi {
                        self.add_new_seg(
                            bi_lst,
                            peak_bi.borrow().idx(),
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
                            peak_bi.borrow().idx(),
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
                    bi_lst.last().unwrap().borrow().idx(),
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
        last_seg_end_bi: Handle<T>,
        bi_lst: &[Handle<T>],
    ) -> Result<(), CChanException> {
        if last_seg_end_bi.borrow().is_down() {
            if let Some(peak_bi) =
                find_peak_bi(&bi_lst[last_seg_end_bi.borrow().idx() as usize + 3..], true)
            {
                if peak_bi.borrow().idx() - last_seg_end_bi.borrow().idx() >= 3 {
                    self.add_new_seg(
                        bi_lst,
                        peak_bi.borrow().idx(),
                        false,
                        Some(BiDir::Up),
                        true,
                        "collectleft_find_high",
                    )?;
                }
            }
        } else if let Some(peak_bi) = find_peak_bi(
            &bi_lst[last_seg_end_bi.borrow().idx() as usize + 3..],
            false,
        ) {
            if peak_bi.borrow().idx() - last_seg_end_bi.borrow().idx() >= 3 {
                self.add_new_seg(
                    bi_lst,
                    peak_bi.borrow().idx(),
                    false,
                    Some(BiDir::Down),
                    true,
                    "collectleft_find_low",
                )?;
            }
        }
        let last_seg_end_bi = self.lst.last().unwrap().borrow().end_bi.clone();
        self.collect_left_as_seg(bi_lst)?;
        Ok(())
    }

    pub fn collect_segs(&mut self, bi_lst: &[Handle<T>]) -> Result<(), CChanException> {
        let last_bi = bi_lst.last().unwrap();
        let last_seg_end_bi = self.lst.last().unwrap().borrow().end_bi.clone();
        if last_bi.borrow().idx() - last_seg_end_bi.borrow().idx() < 3 {
            return Ok(());
        }
        if last_seg_end_bi.borrow().is_down()
            && last_bi.borrow().get_end_val() <= last_seg_end_bi.borrow().get_end_val()
        {
            if let Some(peak_bi) =
                find_peak_bi(&bi_lst[last_seg_end_bi.borrow().idx() as usize + 3..], true)
            {
                self.add_new_seg(
                    bi_lst,
                    peak_bi.borrow().idx(),
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
            if let Some(peak_bi) = find_peak_bi(
                &bi_lst[last_seg_end_bi.borrow().idx() as usize + 3..],
                false,
            ) {
                self.add_new_seg(
                    bi_lst,
                    peak_bi.borrow().idx(),
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

    pub fn collect_left_seg(&mut self, bi_lst: &[Handle<T>]) -> Result<(), CChanException> {
        if self.lst.is_empty() {
            self.collect_first_seg(bi_lst)?;
        } else {
            self.collect_segs(bi_lst)?;
        }
        Ok(())
    }

    pub fn collect_left_as_seg(&mut self, bi_lst: &[Handle<T>]) -> Result<(), CChanException> {
        let last_bi = bi_lst.last().unwrap();
        let last_seg_end_bi = self.lst.last().unwrap().borrow().end_bi.clone();
        if last_seg_end_bi.borrow().idx() + 1 >= bi_lst.len() as i32 {
            return Ok(());
        }
        if last_seg_end_bi.borrow().dir() == last_bi.borrow().dir() {
            self.add_new_seg(
                bi_lst,
                last_bi.borrow().idx() - 1,
                false,
                None,
                true,
                "collect_left_1",
            )?;
        } else {
            self.add_new_seg(
                bi_lst,
                last_bi.borrow().idx(),
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
        bi_lst: &[Handle<T>],
        end_bi_idx: usize,
        is_sure: bool,
        seg_dir: Option<BiDir>,
        split_first_seg: bool,
        reason: &str,
    ) -> Result<(), CChanException> {
        if self.lst.is_empty() && split_first_seg && end_bi_idx >= 3 {
            if let Some(peak_bi) = find_peak_bi(
                &bi_lst[..=end_bi_idx - 3] // 确认是否是=
                    .iter()
                    .rev()
                    .cloned()
                    .collect::<Vec<_>>(),
                bi_lst[end_bi_idx as usize].borrow().is_down(),
            )
            //find_peak_bi(&bi_lst[0..=end_bi_idx - 3], bi_lst[end_bi_idx].is_down())
            {
                if (peak_bi.borrow().is_down()
                    && (peak_bi.borrow().low() < bi_lst[0].borrow().low()
                        || peak_bi.borrow().idx() == 0))
                    || (peak_bi.borrow().is_up()
                        && (peak_bi.borrow().high() > bi_lst[0].borrow().high()
                            || peak_bi.borrow().idx() == 0))
                {
                    self.add_new_seg(
                        bi_lst,
                        peak_bi.borrow().idx(),
                        false,
                        Some(peak_bi.borrow().dir()),
                        true,
                        "split_first_1st",
                    )?;
                    self.add_new_seg(
                        bi_lst,
                        end_bi_idx as i32,
                        false,
                        None,
                        true,
                        "split_first_2nd",
                    )?;
                    return Ok(());
                }
            }
        }
        let bi1_idx = if self.lst.is_empty() {
            0
        } else {
            self.lst.last().unwrap().borrow().end_bi.borrow().idx() + 1
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
        bi_lst: &[Handle<T>],
        end_bi_idx: i32,
        is_sure: bool,
        seg_dir: Option<BiDir>,
        split_first_seg: bool,
        reason: &str,
    ) -> Result<bool, CChanException> {
        match self.try_add_new_seg(
            bi_lst,
            end_bi_idx as usize,
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

/*impl<T: Line<T>> CSegListChan<T> {
    pub fn do_init(&mut self) {
        // 删除末尾不确定的线段
        while !self.lst.is_empty() && !self.lst.last().unwrap().borrow().is_sure {
            let _seg = self.lst.pop().unwrap();
            for bi in &_seg.borrow().bi_list {
                bi.borrow_mut().set_parent_seg(None);
            }
            if let Some(pre) = &_seg.borrow().pre {
                pre.borrow_mut().next = None;
            }
        }
        if !self.lst.is_empty() {
            assert!(
                self.lst.last().unwrap().borrow().eigen_fx.is_some()
                    && self
                        .lst
                        .last()
                        .unwrap()
                        .borrow()
                        .eigen_fx
                        .as_ref()
                        .unwrap()
                        .borrow()
                        .ele[2]
                        .is_some()
            );
            if !self
                .lst
                .last()
                .unwrap()
                .borrow()
                .eigen_fx
                .as_ref()
                .unwrap()
                .borrow()
                .ele[2]
                .as_ref()
                .unwrap()
                .borrow()
                .lst
                .last()
                .unwrap()
                .borrow()
                .is_sure()
            {
                // 如果确定线段的分形的第三元素包含不确定笔，也需要重新算，不然线段分形元素的高低点可能不对
                self.lst.pop();
            }
        }
    }

    pub fn update(&mut self, bi_lst: &[Handle<T>]) -> Result<(), CChanException> {
        self.do_init();
        if self.lst.is_empty() {
            self.cal_seg_sure(bi_lst, 0)?;
        } else {
            let last_end_bi_idx = self.lst.last().unwrap().borrow().end_bi.borrow().idx();
            self.cal_seg_sure(bi_lst, last_end_bi_idx + 1)?;
        }
        self.collect_left_seg(bi_lst)?;
        Ok(())
    }

    pub fn cal_seg_sure(
        &mut self,
        bi_lst: &[Handle<T>],
        begin_idx: i32,
    ) -> Result<(), CChanException> {
        let mut up_eigen = CEigenFX::<T>::new(BiDir::Up, false, self.lv);
        let mut down_eigen = CEigenFX::<T>::new(BiDir::Down, false, self.lv);
        let mut last_seg_dir = if self.lst.is_empty() {
            None
        } else {
            Some(self.lst.last().unwrap().borrow().dir)
        };

        for bi in bi_lst.iter().skip(begin_idx as usize) {
            let mut fx_eigen = None;
            if bi.borrow().is_down() && last_seg_dir != Some(BiDir::Up) {
                if up_eigen.add(bi.clone())? {
                    fx_eigen = Some(&mut up_eigen);
                }
            } else if bi.borrow().is_up() && last_seg_dir != Some(BiDir::Down) {
                if down_eigen.add(bi.clone())? {
                    fx_eigen = Some(&mut down_eigen);
                }
            }
            if self.lst.is_empty() {
                if up_eigen.ele[1].is_some() && bi.borrow().is_down() {
                    last_seg_dir = Some(BiDir::Down);
                    down_eigen.clear();
                } else if down_eigen.ele[1].is_some() && bi.borrow().is_up() {
                    up_eigen.clear();
                    last_seg_dir = Some(BiDir::Up);
                }
                if up_eigen.ele[1].is_none()
                    && last_seg_dir == Some(BiDir::Down)
                    && bi.borrow().dir() == BiDir::Down
                {
                    last_seg_dir = None;
                } else if down_eigen.ele[1].is_none()
                    && last_seg_dir == Some(BiDir::Up)
                    && bi.borrow().dir() == BiDir::Up
                {
                    last_seg_dir = None;
                }
            }

            if let Some(fx_eigen) = fx_eigen {
                self.treat_fx_eigen(fx_eigen, bi_lst)?;
                break;
            }
        }
        Ok(())
    }

    pub fn treat_fx_eigen(
        &mut self,
        fx_eigen: &Handle<CEigenFX<T>>,
        bi_lst: &[Handle<T>],
    ) -> Result<(), CChanException> {
        let _test = fx_eigen.borrow_mut().can_be_end(bi_lst);
        let end_bi_idx = fx_eigen.borrow().get_peak_bi_idx();
        match _test {
            Some(true) | None => {
                let is_true = _test.is_some();
                if !self.add_new_seg(
                    bi_lst,
                    end_bi_idx,
                    is_true && fx_eigen.borrow().all_bi_is_sure(),
                    None,
                    true,
                    "treat_fx_eigen",
                )? {
                    self.cal_seg_sure(bi_lst, end_bi_idx + 1)?;
                    return Ok(());
                }
                self.lst.last_mut().unwrap().borrow_mut().eigen_fx = Some(fx_eigen.clone());
                if is_true {
                    self.cal_seg_sure(bi_lst, end_bi_idx + 1)?;
                }
            }
            Some(false) => {
                self.cal_seg_sure(bi_lst, fx_eigen.borrow().lst[1].borrow().idx())?;
            }
        }
        Ok(())
    }
}*/

impl<T> std::ops::Index<usize> for CSegListComm<T> {
    type Output = Handle<CSeg<T>>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.lst[index]
    }
}

impl<T> std::ops::IndexMut<usize> for CSegListComm<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.lst[index]
    }
}

impl<T> std::ops::Deref for CSegListComm<T> {
    type Target = Vec<Handle<CSeg<T>>>;

    fn deref(&self) -> &Self::Target {
        &self.lst
    }
}

impl<T> std::ops::DerefMut for CSegListComm<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.lst
    }
}

pub fn find_peak_bi<T: Line<T>>(bi_lst: &[Handle<T>], is_high: bool) -> Option<Handle<T>> {
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
