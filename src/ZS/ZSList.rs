use crate::Common::func_util::revert_bi_dir;
use crate::Common::types::{Handle, WeakHandle};
use crate::Common::CEnum::{BiDir, ZsAlgo};
use crate::Seg::linetype::{Line, SegLine};
use crate::Seg::Seg::CSeg;
use crate::Seg::SegListChan::CSegListChan;
use crate::ZS::ZSConfig::CZSConfig;
use crate::ZS::ZS::CZS;
use std::cell::RefCell;
use std::rc::Rc;

pub struct CZSList<T> {
    zs_lst: Vec<Handle<CZS<T>>>,
    config: CZSConfig,
    free_item_lst: Vec<WeakHandle<T>>,
    last_sure_pos: Option<usize>,
}

impl<T: Line> CZSList<T> {
    pub fn new(zs_config: Option<CZSConfig>) -> Self {
        CZSList {
            zs_lst: Vec::new(),
            config: zs_config.unwrap_or_default(),
            free_item_lst: Vec::new(),
            last_sure_pos: None,
        }
    }

    pub fn update_last_pos(&mut self, seg_list: &CSegListChan<T>) {
        self.last_sure_pos = None;
        for seg in seg_list.iter().rev() {
            if seg.borrow().is_sure {
                self.last_sure_pos =
                    Some(seg.borrow().start_bi.upgrade().unwrap().borrow().line_idx());
                return;
            }
        }
    }

    pub fn seg_need_cal(&self, seg: &Handle<CSeg<T>>) -> bool {
        match self.last_sure_pos {
            None => true,
            Some(pos) => seg.borrow().start_bi.upgrade().unwrap().borrow().line_idx() >= pos,
        }
    }

    pub fn add_to_free_lst(&mut self, item: &WeakHandle<T>, is_sure: bool, zs_algo: ZsAlgo) {
        if !self.free_item_lst.is_empty()
            && item.upgrade().unwrap().borrow().line_idx()
                == self
                    .free_item_lst
                    .last()
                    .unwrap()
                    .upgrade()
                    .unwrap()
                    .borrow()
                    .line_idx()
        {
            // 防止笔新高或新低的更新带来bug
            self.free_item_lst.pop();
        }
        self.free_item_lst.push(item.clone());
        if let Some(res) = self.try_construct_zs(&self.free_item_lst, is_sure, zs_algo) {
            if let Some(ref begin_bi) = res.begin_bi {
                if begin_bi.upgrade().unwrap().borrow().line_idx() > 0 {
                    self.zs_lst.push(Rc::new(RefCell::new(res)));
                    self.clear_free_lst();
                    self.try_combine();
                }
            }
        }
    }

    pub fn clear_free_lst(&mut self) {
        self.free_item_lst.clear();
    }

    pub fn update(&mut self, bi: Handle<T>, is_sure: bool) {
        if self.free_item_lst.is_empty() && self.try_add_to_end(&Rc::downgrade(&bi)) {
            self.try_combine();
            return;
        }
        self.add_to_free_lst(&Rc::downgrade(&bi), is_sure, ZsAlgo::Normal);
    }

    pub fn try_add_to_end(&mut self, bi: &WeakHandle<T>) -> bool {
        if self.zs_lst.is_empty() {
            false
        } else {
            self.zs_lst
                .last_mut()
                .unwrap()
                .borrow_mut()
                .try_add_to_end(&bi.upgrade().unwrap())
        }
    }

    pub fn add_zs_from_bi_range(
        &mut self,
        seg_bi_lst: &[Handle<T>],
        seg_dir: BiDir,
        seg_is_sure: bool,
    ) {
        let mut deal_bi_cnt = 0;
        for bi in seg_bi_lst {
            if bi.borrow().line_dir() == seg_dir {
                continue;
            }
            if deal_bi_cnt < 1 {
                self.add_to_free_lst(&Rc::downgrade(bi), seg_is_sure, ZsAlgo::Normal);
                deal_bi_cnt += 1;
            } else {
                self.update(bi.clone(), seg_is_sure);
            }
        }
    }

    pub fn try_construct_zs(
        &self,
        lst: &[WeakHandle<T>],
        is_sure: bool,
        zs_algo: ZsAlgo,
    ) -> Option<CZS<T>> {
        let lst = match zs_algo {
            ZsAlgo::Normal => {
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
            ZsAlgo::OverSeg => {
                if lst.len() < 3 {
                    return None;
                }
                let lst = &lst[lst.len() - 3..];
                if lst[0].upgrade().unwrap().borrow().line_dir()
                    == lst[0]
                        .upgrade()
                        .unwrap()
                        .borrow()
                        .line_get_parent_seg()
                        .as_ref()
                        .unwrap()
                        .borrow()
                        .dir
                {
                    &lst[1..]
                } else {
                    lst
                }
            }
            _ => lst,
        };

        let min_high = lst
            .iter()
            .map(|item| item.upgrade().unwrap().borrow().line_high())
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let max_low = lst
            .iter()
            .map(|item| item.upgrade().unwrap().borrow().line_low())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();

        if min_high > max_low {
            Some(CZS::new(
                Some(lst.iter().map(|x| x.upgrade().unwrap()).collect()),
                is_sure,
            ))
        } else {
            None
        }
    }

    pub fn cal_bi_zs(&mut self, bi_lst: &[Handle<T>], seg_lst: &CSegListChan<T>) {
        // 移除不确定的中枢
        while !self.zs_lst.is_empty() {
            let last_zs = self.zs_lst.last().unwrap();
            let begin_idx = last_zs
                .borrow()
                .begin_bi
                .as_ref()
                .unwrap()
                .upgrade()
                .unwrap()
                .borrow()
                .line_idx();

            match self.last_sure_pos {
                Some(pos) if begin_idx >= pos => {
                    self.zs_lst.pop();
                }
                _ => break,
            }
        }

        match self.config.zs_algo {
            ZsAlgo::Normal => {
                for seg in seg_lst.iter() {
                    if !self.seg_need_cal(seg) {
                        continue;
                    }
                    self.clear_free_lst();
                    let seg_bi_lst =
                        &bi_lst[seg.borrow().start_bi.upgrade().unwrap().borrow().line_idx()
                            ..seg.borrow().end_bi.upgrade().unwrap().borrow().line_idx() + 1];
                    self.add_zs_from_bi_range(&seg_bi_lst, seg.borrow().dir, seg.borrow().is_sure);
                }

                if !seg_lst.is_empty() {
                    self.clear_free_lst();
                    let last_seg = seg_lst.last().unwrap();
                    let remaining_bi_lst = &bi_lst[last_seg
                        .borrow()
                        .end_bi
                        .upgrade()
                        .unwrap()
                        .borrow()
                        .line_idx()
                        + 1..];
                    self.add_zs_from_bi_range(
                        &remaining_bi_lst,
                        revert_bi_dir(&last_seg.borrow().dir),
                        false,
                    );
                }
            }
            ZsAlgo::OverSeg => {
                assert!(!self.config.one_bi_zs);
                self.clear_free_lst();
                let begin_bi_idx = if !self.zs_lst.is_empty() {
                    self.zs_lst
                        .last()
                        .unwrap()
                        .borrow()
                        .end_bi
                        .as_ref()
                        .unwrap()
                        .upgrade()
                        .unwrap()
                        .borrow()
                        .line_idx()
                        + 1
                } else {
                    0
                };
                for bi in &bi_lst[begin_bi_idx..] {
                    self.update_overseg_zs(bi);
                }
            }
            ZsAlgo::Auto => {
                let mut sure_seg_appear = false;
                let exist_sure_seg = seg_lst.exist_sure_seg();
                for seg in seg_lst.iter() {
                    let seg_ref = seg.borrow();
                    if seg_ref.is_sure {
                        sure_seg_appear = true;
                    }
                    if !self.seg_need_cal(seg) {
                        continue;
                    }
                    if seg_ref.is_sure || (!sure_seg_appear && exist_sure_seg) {
                        self.clear_free_lst();
                        let seg_bi_lst =
                            &bi_lst[seg_ref.start_bi.upgrade().unwrap().borrow().line_idx()
                                ..seg_ref.end_bi.upgrade().unwrap().borrow().line_idx() + 1];
                        self.add_zs_from_bi_range(&seg_bi_lst, seg_ref.dir, seg_ref.is_sure);
                    } else {
                        self.clear_free_lst();
                        for bi in &bi_lst[seg_ref.start_bi.upgrade().unwrap().borrow().line_idx()..]
                        {
                            self.update_overseg_zs(bi);
                        }
                        break;
                    }
                }
            }
        }

        self.update_last_pos(seg_lst);
    }

    pub fn update_overseg_zs(&mut self, bi: &Handle<T>) {
        if !self.zs_lst.is_empty() && self.free_item_lst.is_empty() {
            if bi.borrow().line_next().is_none() {
                return;
            }
            let last_zs = self.zs_lst.last().unwrap();
            if bi.borrow().line_idx()
                - last_zs
                    .borrow()
                    .end_bi
                    .as_ref()
                    .unwrap()
                    .upgrade()
                    .unwrap()
                    .borrow()
                    .line_idx()
                <= 1
                && last_zs
                    .borrow()
                    .in_range(&bi.borrow().line_next().as_ref().unwrap())
                && last_zs.borrow_mut().try_add_to_end(&bi)
            {
                return;
            }
        }
        if !self.zs_lst.is_empty()
            && self.free_item_lst.is_empty()
            && self.zs_lst.last().unwrap().borrow().in_range(&bi)
            && bi.borrow().line_idx()
                - self
                    .zs_lst
                    .last()
                    .unwrap()
                    .borrow()
                    .end_bi
                    .as_ref()
                    .unwrap()
                    .upgrade()
                    .unwrap()
                    .borrow()
                    .line_idx()
                <= 1
        {
            return;
        }
        self.add_to_free_lst(
            &Rc::downgrade(bi),
            bi.borrow().line_is_sure(),
            ZsAlgo::Normal,
        );
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
                .combine(&last.borrow(), self.config.zs_combine_mode)
            {
                continue;
            }
            self.zs_lst.push(last);
            break;
        }
    }
}

impl<T: Line> std::ops::Deref for CZSList<T> {
    type Target = Vec<Handle<CZS<T>>>;

    fn deref(&self) -> &Self::Target {
        &self.zs_lst
    }
}

impl<T: Line> std::ops::DerefMut for CZSList<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.zs_lst
    }
}

impl<T: Line> std::iter::IntoIterator for CZSList<T> {
    type Item = Handle<CZS<T>>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.zs_lst.into_iter()
    }
}
