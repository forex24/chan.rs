use super::eigen_fx::CEigenFX;
use super::seg_config::CSegConfig;
use super::seg_list_comm::CSegListComm;
use crate::bi::bi_list::CBiList;
use crate::common::c_enum::{BiDir, SegType};
use crate::common::chan_exception::CChanException;
use std::cell::RefCell;
use std::rc::Rc;

pub struct CSegListChan<SUB_LINE_TYPE> {
    inner: CSegListComm<SUB_LINE_TYPE>,
}

impl<SUB_LINE_TYPE> CSegListChan<SUB_LINE_TYPE> {
    pub fn new(seg_config: Option<CSegConfig>, lv: SegType) -> Self {
        CSegListChan {
            inner: CSegListComm::new(seg_config, lv),
        }
    }

    pub fn do_init(&mut self) {
        // 删除末尾不确定的线段
        while !self.inner.lst.is_empty() && !self.inner.lst.last().unwrap().borrow().is_sure {
            let _seg = self.inner.lst.pop().unwrap();
            for bi in &_seg.borrow().bi_list {
                bi.borrow_mut().parent_seg = None;
            }
            if let Some(pre) = &_seg.borrow().pre {
                pre.borrow_mut().next = None;
            }
        }
        if !self.inner.lst.is_empty() {
            assert!(
                self.inner.lst.last().unwrap().borrow().eigen_fx.is_some()
                    && self
                        .inner
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
                .inner
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
                .is_sure
            {
                // 如果确定线段的分形的第三元素包含不确定笔，也需要重新算，不然线段分形元素的高低点可能不对
                self.inner.lst.pop();
            }
        }
    }

    pub fn update(&mut self, bi_lst: &CBiList) -> Result<(), CChanException> {
        self.do_init();
        if self.inner.lst.is_empty() {
            self.cal_seg_sure(bi_lst, 0)?;
        } else {
            let last_end_bi_idx = self.inner.lst.last().unwrap().borrow().end_bi.borrow().idx;
            self.cal_seg_sure(bi_lst, last_end_bi_idx + 1)?;
        }
        self.inner.collect_left_seg(bi_lst)?;
        Ok(())
    }

    pub fn cal_seg_sure(&mut self, bi_lst: &CBiList, begin_idx: i32) -> Result<(), CChanException> {
        let mut up_eigen = CEigenFX::new(BiDir::Up, false, self.inner.lv);
        let mut down_eigen = CEigenFX::new(BiDir::Down, false, self.inner.lv);
        let mut last_seg_dir = if self.inner.lst.is_empty() {
            None
        } else {
            Some(self.inner.lst.last().unwrap().borrow().dir)
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
            if self.inner.lst.is_empty() {
                if up_eigen.ele[1].is_some() && bi.borrow().is_down() {
                    last_seg_dir = Some(BiDir::Down);
                    down_eigen.clear();
                } else if down_eigen.ele[1].is_some() && bi.borrow().is_up() {
                    up_eigen.clear();
                    last_seg_dir = Some(BiDir::Up);
                }
                if up_eigen.ele[1].is_none()
                    && last_seg_dir == Some(BiDir::Down)
                    && bi.borrow().dir == BiDir::Down
                {
                    last_seg_dir = None;
                } else if down_eigen.ele[1].is_none()
                    && last_seg_dir == Some(BiDir::Up)
                    && bi.borrow().dir == BiDir::Up
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
        fx_eigen: &mut CEigenFX,
        bi_lst: &CBiList,
    ) -> Result<(), CChanException> {
        let _test = fx_eigen.can_be_end(bi_lst);
        let end_bi_idx = fx_eigen.get_peak_bi_idx();
        match _test {
            Some(true) | None => {
                let is_true = _test.is_some();
                if !self.inner.add_new_seg(
                    bi_lst,
                    end_bi_idx,
                    is_true && fx_eigen.all_bi_is_sure(),
                    None,
                    true,
                    "treat_fx_eigen",
                )? {
                    self.cal_seg_sure(bi_lst, end_bi_idx + 1)?;
                    return Ok(());
                }
                self.inner.lst.last_mut().unwrap().borrow_mut().eigen_fx =
                    Some(Rc::new(RefCell::new(fx_eigen.clone())));
                if is_true {
                    self.cal_seg_sure(bi_lst, end_bi_idx + 1)?;
                }
            }
            Some(false) => {
                self.cal_seg_sure(bi_lst, fx_eigen.lst[1].borrow().idx)?;
            }
        }
        Ok(())
    }
}

impl<SUB_LINE_TYPE> std::ops::Deref for CSegListChan<SUB_LINE_TYPE> {
    type Target = CSegListComm<SUB_LINE_TYPE>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<SUB_LINE_TYPE> std::ops::DerefMut for CSegListChan<SUB_LINE_TYPE> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
