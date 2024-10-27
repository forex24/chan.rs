use crate::Bi::Bi::CBi;
use crate::Bi::BiList::CBiList;
use crate::Common::func_util::revert_BiDir;
use crate::Common::types::SharedCell;
use crate::Seg::Seg::CSeg;
use crate::Seg::SegListChan::CSegListChan;
use crate::ZS::ZSConfig::CZSConfig;
use crate::ZS::ZS::CZS;
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

pub struct CZSList {
    zs_lst: Vec<SharedCell<CZS>>,
    config: CZSConfig,
    free_item_lst: Vec<SharedCell<dyn Any>>,
    last_sure_pos: i32,
}

impl CZSList {
    pub fn new(zs_config: Option<CZSConfig>) -> Self {
        CZSList {
            zs_lst: Vec::new(),
            config: zs_config.unwrap_or_default(),
            free_item_lst: Vec::new(),
            last_sure_pos: -1,
        }
    }

    pub fn update_last_pos<T>(&mut self, seg_list: &CSegListChan<T>) {
        self.last_sure_pos = -1;
        for seg in seg_list.iter().rev() {
            if seg.is_sure {
                self.last_sure_pos = seg.start_bi.borrow().idx;
                return;
            }
        }
    }

    pub fn seg_need_cal<T>(&self, seg: &CSeg<T>) -> bool {
        seg.start_bi.borrow().idx >= self.last_sure_pos
    }

    pub fn add_to_free_lst(&mut self, item: SharedCell<dyn Any>, is_sure: bool, zs_algo: &str) {
        if !self.free_item_lst.is_empty()
            && item.borrow().idx == self.free_item_lst.last().unwrap().borrow().idx
        {
            self.free_item_lst.pop();
        }
        self.free_item_lst.push(item.clone());
        if let Some(res) = self.try_construct_zs(&self.free_item_lst, is_sure, zs_algo) {
            if res.begin_bi.borrow().idx > 0 {
                self.zs_lst.push(Rc::new(RefCell::new(res)));
                self.clear_free_lst();
                self.try_combine();
            }
        }
    }

    pub fn clear_free_lst(&mut self) {
        self.free_item_lst.clear();
    }

    pub fn update(&mut self, bi: SharedCell<CBi>, is_sure: bool) {
        if self.free_item_lst.is_empty() && self.try_add_to_end(&bi) {
            self.try_combine();
            return;
        }
        self.add_to_free_lst(bi, is_sure, "normal");
    }

    pub fn try_add_to_end(&mut self, bi: &SharedCell<CBi>) -> bool {
        if self.zs_lst.is_empty() {
            false
        } else {
            self.zs_lst
                .last_mut()
                .unwrap()
                .borrow_mut()
                .try_add_to_end(bi)
        }
    }

    pub fn add_zs_from_bi_range(
        &mut self,
        seg_bi_lst: &[SharedCell<CBi>],
        seg_dir: i32,
        seg_is_sure: bool,
    ) {
        let mut deal_bi_cnt = 0;
        for bi in seg_bi_lst {
            if bi.borrow().dir == seg_dir {
                continue;
            }
            if deal_bi_cnt < 1 {
                self.add_to_free_lst(bi.clone(), seg_is_sure, "normal");
                deal_bi_cnt += 1;
            } else {
                self.update(bi.clone(), seg_is_sure);
            }
        }
    }

    pub fn try_construct_zs(
        &self,
        lst: &[SharedCell<dyn Any>],
        is_sure: bool,
        zs_algo: &str,
    ) -> Option<CZS> {
        let lst = match zs_algo {
            "normal" => {
                if !self.config.one_bi_zs {
                    if lst.len() == 1 {
                        return None;
                    } else {
                        &lst[lst.len() - 2..]
                    }
                } else {
                    lst
                }
            }
            "over_seg" => {
                if lst.len() < 3 {
                    return None;
                }
                let lst = &lst[lst.len() - 3..];
                if lst[0].borrow().dir == lst[0].borrow().parent_seg.borrow().dir {
                    &lst[1..]
                } else {
                    lst
                }
            }
            _ => lst,
        };

        let min_high = lst
            .iter()
            .map(|item| item.borrow()._high())
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let max_low = lst
            .iter()
            .map(|item| item.borrow()._low())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();

        if min_high > max_low {
            Some(CZS::new(Some(lst.to_vec()), is_sure))
        } else {
            None
        }
    }

    pub fn cal_bi_zs<T>(&mut self, bi_lst: &dyn Any, seg_lst: &CSegListChan<T>) {
        while !self.zs_lst.is_empty()
            && self.zs_lst.last().unwrap().borrow().begin_bi.borrow().idx >= self.last_sure_pos
        {
            self.zs_lst.pop();
        }

        match self.config.zs_algo.as_str() {
            "normal" => {
                for seg in seg_lst.iter() {
                    if !self.seg_need_cal(seg) {
                        continue;
                    }
                    self.clear_free_lst();
                    let seg_bi_lst = bi_lst
                        .downcast_ref::<CBiList>()
                        .unwrap()
                        .slice(seg.start_bi.borrow().idx, seg.end_bi.borrow().idx + 1);
                    self.add_zs_from_bi_range(&seg_bi_lst, seg.dir, seg.is_sure);
                }

                if !seg_lst.is_empty() {
                    self.clear_free_lst();
                    let last_seg = seg_lst.last().unwrap();
                    let remaining_bi_lst = bi_lst
                        .downcast_ref::<CBiList>()
                        .unwrap()
                        .slice(last_seg.end_bi.borrow().idx + 1, bi_lst.len());
                    self.add_zs_from_bi_range(&remaining_bi_lst, revert_BiDir(last_seg.dir), false);
                }
            }
            "over_seg" => {
                assert!(!self.config.one_bi_zs);
                self.clear_free_lst();
                let begin_bi_idx = if !self.zs_lst.is_empty() {
                    self.zs_lst.last().unwrap().borrow().end_bi.borrow().idx + 1
                } else {
                    0
                };
                for bi in bi_lst
                    .downcast_ref::<CBiList>()
                    .unwrap()
                    .slice(begin_bi_idx, bi_lst.len())
                {
                    self.update_overseg_zs(bi);
                }
            }
            "auto" => {
                let mut sure_seg_appear = false;
                let exist_sure_seg = seg_lst.exist_sure_seg();
                for seg in seg_lst.iter() {
                    if seg.is_sure {
                        sure_seg_appear = true;
                    }
                    if !self.seg_need_cal(seg) {
                        continue;
                    }
                    if seg.is_sure || (!sure_seg_appear && exist_sure_seg) {
                        self.clear_free_lst();
                        let seg_bi_lst = bi_lst
                            .downcast_ref::<CBiList>()
                            .unwrap()
                            .slice(seg.start_bi.borrow().idx, seg.end_bi.borrow().idx + 1);
                        self.add_zs_from_bi_range(&seg_bi_lst, seg.dir, seg.is_sure);
                    } else {
                        self.clear_free_lst();
                        for bi in bi_lst
                            .downcast_ref::<CBiList>()
                            .unwrap()
                            .slice(seg.start_bi.borrow().idx, bi_lst.len())
                        {
                            self.update_overseg_zs(bi);
                        }
                        break;
                    }
                }
            }
            _ => panic!("unknown zs_algo {}", self.config.zs_algo),
        }

        self.update_last_pos(seg_lst);
    }

    pub fn update_overseg_zs(&mut self, bi: SharedCell<dyn Any>) {
        if !self.zs_lst.is_empty() && self.free_item_lst.is_empty() {
            if bi.borrow().next.is_none() {
                return;
            }
            let last_zs = self.zs_lst.last().unwrap();
            if bi.borrow().idx - last_zs.borrow().end_bi.borrow().idx <= 1
                && last_zs
                    .borrow()
                    .in_range(&bi.borrow().next.as_ref().unwrap())
                && last_zs.borrow_mut().try_add_to_end(bi)
            {
                return;
            }
        }
        if !self.zs_lst.is_empty()
            && self.free_item_lst.is_empty()
            && self.zs_lst.last().unwrap().borrow().in_range(&bi)
            && bi.borrow().idx - self.zs_lst.last().unwrap().borrow().end_bi.borrow().idx <= 1
        {
            return;
        }
        self.add_to_free_lst(bi, bi.borrow().is_sure, "over_seg");
    }

    pub fn try_combine(&mut self) {
        if !self.config.need_combine {
            return;
        }
        while self.zs_lst.len() >= 2 {
            let last = self.zs_lst.pop().unwrap();
            let second_last = self.zs_lst.last_mut().unwrap();
            if second_last
                .borrow_mut()
                .combine(&last.borrow(), &self.config.zs_combine_mode)
            {
                continue;
            }
            self.zs_lst.push(last);
            break;
        }
    }
}

impl std::ops::Deref for CZSList {
    type Target = Vec<SharedCell<CZS>>;

    fn deref(&self) -> &Self::Target {
        &self.zs_lst
    }
}

impl std::ops::DerefMut for CZSList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.zs_lst
    }
}

impl std::iter::IntoIterator for CZSList {
    type Item = SharedCell<CZS>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.zs_lst.into_iter()
    }
}
