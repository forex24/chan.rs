use crate::{
    Bi::BiList::CBiList,
    Common::CEnum::{BiDir, SegType},
};

use super::{linetype::Line, EigenFX::CEigenFX, SegConfig::CSegConfig, SegListComm::CSegListComm};

pub struct CSegListChan<T: Line> {
    inner: CSegListComm<T>,
}

impl<T: Line> CSegListChan<T> {
    pub fn new(seg_config: CSegConfig, lv: SegType) -> Self {
        let mut instance = Self {
            inner: CSegListComm::new(seg_config, lv),
        };
        instance.do_init();
        instance
    }

    pub fn do_init(&mut self) {
        // 删除末尾不确定的线段
        while !self.inner.lst.is_empty() && !self.inner.lst.last().unwrap().borrow().is_sure {
            let _seg = self.inner.lst.last().unwrap().clone();
            let _seg_ref = _seg.borrow();
            for bi in &_seg_ref.bi_list {
                bi.borrow_mut().parent_seg = None;
            }
            if let Some(pre) = &_seg_ref.pre {
                pre.borrow_mut().next = None;
            }
            self.inner.lst.pop();
        }

        if !self.inner.lst.is_empty() {
            let last_seg = self.inner.lst.last().unwrap();
            let last_seg_ref = last_seg.borrow();
            assert!(
                last_seg_ref.eigen_fx.is_some()
                    && last_seg_ref.eigen_fx.as_ref().unwrap().ele[2].is_some()
            );
            if !last_seg_ref.eigen_fx.as_ref().unwrap().ele[2]
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

    pub fn update(&mut self, bi_lst: &CBiList) {
        self.do_init();
        if self.inner.lst.is_empty() {
            self.cal_seg_sure(bi_lst, 0);
        } else {
            let begin_idx =
                self.inner.lst.last().unwrap().borrow().end_bi.borrow().idx as usize + 1;
            self.cal_seg_sure(bi_lst, begin_idx);
        }
        self.inner.collect_left_seg(bi_lst);
    }

    fn cal_seg_sure(&mut self, bi_lst: &CBiList, begin_idx: usize) {
        let mut up_eigen = CEigenFX::new(BiDir::Up, false, self.inner.lv); // 上升线段下降笔
        let mut down_eigen = CEigenFX::new(BiDir::Down, false, self.inner.lv); // 下降线段上升笔
        let mut last_seg_dir = if self.inner.lst.is_empty() {
            None
        } else {
            Some(self.inner.lst.last().unwrap().borrow().dir)
        };

        for bi in bi_lst.iter().skip(begin_idx) {
            let mut fx_eigen = None;
            if bi.borrow().is_down() && last_seg_dir != Some(BiDir::Up) {
                if up_eigen.add(bi.clone()) {
                    fx_eigen = Some(up_eigen.clone());
                }
            } else if bi.borrow().is_up() && last_seg_dir != Some(BiDir::Down) {
                if down_eigen.add(bi.clone()) {
                    fx_eigen = Some(down_eigen.clone());
                }
            }

            if self.inner.lst.is_empty() {
                // 尝试确定第一段方向，不要以谁先成为分形来决定
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
                self.treat_fx_eigen(&fx_eigen, bi_lst);
                break;
            }
        }
    }

    fn treat_fx_eigen(&mut self, fx_eigen: &CEigenFX, bi_lst: &CBiList) {
        let _test = fx_eigen.can_be_end(bi_lst);
        let end_bi_idx = fx_eigen.get_peak_bi_idx();

        match _test {
            Some(true) | None => {
                // None表示反向分型找到尾部也没找到
                let is_true = _test.is_some(); // 如果是正常结束
                if !self.inner.add_new_seg(
                    bi_lst,
                    end_bi_idx,
                    is_true && fx_eigen.all_bi_is_sure(),
                    None,
                    true,
                    "normal",
                ) {
                    self.cal_seg_sure(bi_lst, end_bi_idx + 1);
                    return;
                }
                self.inner.lst.last_mut().unwrap().borrow_mut().eigen_fx = Some(fx_eigen.clone());
                if is_true {
                    self.cal_seg_sure(bi_lst, end_bi_idx + 1);
                }
            }
            Some(false) => {
                self.cal_seg_sure(bi_lst, fx_eigen.lst[1].borrow().idx);
            }
        }
    }
}

// 实现 Deref 和 DerefMut 以便直接访问 inner 的方法
impl<T: Line> std::ops::Deref for CSegListChan<T> {
    type Target = CSegListComm<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: Line> std::ops::DerefMut for CSegListChan<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
