use std::vec::Vec;

use crate::{
    CPivotAlgo, CSeg, CSegListChan, CZs, CZsConfig, Direction, Handle, ICalcMetric, IParent,
    LineType, ToHandle,
};

// 中枢
pub struct CZsList<T> {
    pub zs_lst: Box<Vec<CZs<T>>>,
    pub config: CZsConfig,
    pub free_item_lst: Vec<Handle<T>>,
    pub last_sure_pos: isize,
}

impl<T: LineType + IParent + ToHandle + ICalcMetric> CZsList<T> {
    pub fn new(zs_config: CZsConfig) -> Self {
        CZsList {
            zs_lst: Box::<Vec<CZs<T>>>::default(),
            config: zs_config,
            free_item_lst: Vec::new(),
            last_sure_pos: -1,
        }
    }

    // 已完备
    fn update_last_pos(&mut self, seg_list: &CSegListChan<T>) {
        self.last_sure_pos = -1;

        for seg in seg_list.iter().rev() {
            if seg.is_sure {
                self.last_sure_pos = seg.start_bi.index() as isize;
                return;
            }
        }
    }

    // 已完备
    fn seg_need_cal(&self, seg: &CSeg<T>) -> bool {
        seg.start_bi.index() as isize >= self.last_sure_pos
    }

    // 已完备
    fn add_to_free_lst(&mut self, item: Handle<T>, is_sure: bool, zs_algo: CPivotAlgo) {
        if let Some(last_item) = self.free_item_lst.last() {
            if item.index() == last_item.index() {
                // 防止笔新高或新低的更新带来bug
                self.free_item_lst.pop();
            }
        }

        self.free_item_lst.push(item);

        let res = self.try_construct_zs(is_sure, zs_algo);
        if let Some(res) = res {
            if res.begin_bi.index() > 0 {
                // 禁止第一笔就是中枢的起点
                self.zs_lst.push(res);
                self.clear_free_lst();
                self.try_combine();
            }
        }
    }

    // 已完备
    fn clear_free_lst(&mut self) {
        self.free_item_lst.clear();
    }

    // 已完备
    fn update(&mut self, bi: Handle<T>, is_sure: bool) {
        if self.free_item_lst.is_empty() && self.try_add_to_end(bi) {
            // zs_combine_mode=peak合并模式下会触发生效，=zs合并一定无效返回
            self.try_combine(); // 新形成的中枢尝试和之前的中枢合并
            return;
        }
        self.add_to_free_lst(bi, is_sure, CPivotAlgo::Normal);
    }

    // 已完备
    fn try_add_to_end(&mut self, bi: Handle<T>) -> bool {
        if self.zs_lst.is_empty() {
            return false;
        }
        self.zs_lst.last_mut().unwrap().try_add_to_end(bi)
    }

    // 已完备
    fn add_zs_from_bi_range(&mut self, seg_bi_lst: &[T], seg_dir: Direction, seg_is_sure: bool) {
        let mut deal_bi_cnt = 0;

        for bi in seg_bi_lst {
            if bi.direction() == seg_dir {
                continue;
            }
            if deal_bi_cnt < 1 {
                // 防止try_add_to_end执行到上一个线段的中枢里面去
                self.add_to_free_lst(bi.to_handle(), seg_is_sure, CPivotAlgo::Normal);
                deal_bi_cnt += 1;
            } else {
                self.update(bi.to_handle(), seg_is_sure);
            }
        }
    }

    // 已完备
    fn try_construct_zs(&mut self, is_sure: bool, zs_algo: CPivotAlgo) -> Option<CZs<T>> {
        let lst = &self.free_item_lst;

        let lst = match zs_algo {
            CPivotAlgo::Normal => {
                if !self.config.one_bi_zs {
                    if lst.len() == 1 {
                        return None;
                    }
                    &lst[lst.len() - 2..]
                } else {
                    lst
                }
            }
            CPivotAlgo::OverSeg => {
                if lst.len() < 3 {
                    return None;
                }
                let lst = &lst[lst.len() - 3..];
                if lst[0].direction() == lst[0].parent_seg_dir().unwrap() {
                    return None;
                }
                &lst[1..]
            }
            _ => return None,
        };

        let min_high = lst
            .iter()
            .map(|item| item.high())
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let max_low = lst
            .iter()
            .map(|item| item.low())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        if min_high > max_low {
            Some(CZs::new(&self.zs_lst, self.zs_lst.len(), lst, is_sure))
        } else {
            None
        }
    }

    // 已完备
    pub fn cal_bi_zs(&mut self, bi_lst: &[T], seg_lst: &CSegListChan<T>) {
        //TODO:
        //let last_sure_pos = self.last_sure_pos;
        //self.zs_lst
        //    .retain(|zs| (zs.begin_bi.index() as isize) < last_sure_pos);

        while let Some(last) = self.zs_lst.last() {
            if (last.begin_bi.index() as isize) >= self.last_sure_pos {
                self.zs_lst.pop();
            } else {
                break;
            }
        }

        match self.config.zs_algo {
            CPivotAlgo::Normal => {
                for seg in seg_lst.iter() {
                    if !self.seg_need_cal(seg) {
                        continue;
                    }
                    self.clear_free_lst();
                    let seg_bi_lst = &bi_lst[seg.start_bi.index()..=seg.end_bi.index()];
                    self.add_zs_from_bi_range(seg_bi_lst, seg.dir, seg.is_sure);
                }

                // 处理未生成新线段的部分
                if !seg_lst.is_empty() {
                    self.clear_free_lst();
                    self.add_zs_from_bi_range(
                        &bi_lst[seg_lst.last().unwrap().end_bi.index() + 1..],
                        seg_lst.last().unwrap().dir.flip(),
                        false,
                    );
                }
            }

            CPivotAlgo::OverSeg => {
                debug_assert!(!self.config.one_bi_zs);

                self.clear_free_lst();

                let begin_bi_idx = if let Some(zs) = self.zs_lst.last() {
                    zs.end_bi.unwrap().index() + 1
                } else {
                    0
                };

                for bi in &bi_lst[begin_bi_idx..] {
                    self.update_overseg_zs(bi.to_handle());
                }
            }

            CPivotAlgo::Auto => {
                let mut sure_seg_appear = false;

                let exist_sure_seg = seg_lst.iter().any(|seg| seg.is_sure); //seg_lst.exist_sure_seg()

                for seg in seg_lst.iter() {
                    if seg.is_sure {
                        sure_seg_appear = true;
                    }

                    if !self.seg_need_cal(seg) {
                        continue;
                    }

                    if seg.is_sure || (!sure_seg_appear && exist_sure_seg) {
                        self.clear_free_lst();

                        self.add_zs_from_bi_range(
                            &bi_lst[seg.start_bi.index()..=seg.end_bi.index()],
                            seg.dir,
                            seg.is_sure,
                        );
                    } else {
                        self.clear_free_lst();

                        for bi in &bi_lst[seg.start_bi.index()..] {
                            self.update_overseg_zs(bi.to_handle());
                        }
                        break;
                    }
                }
            }
        }
        self.update_last_pos(seg_lst);
    }

    // 已完备
    fn update_overseg_zs(&mut self, bi: Handle<T>) {
        if !self.zs_lst.is_empty() && self.free_item_lst.is_empty() {
            if bi.to_handle().next().is_none() {
                return;
            }

            if bi.to_handle().index() - self.zs_lst.last().unwrap().end_bi.unwrap().index() <= 1
                && self
                    .zs_lst
                    .last()
                    .unwrap()
                    .in_range(bi.to_handle().next().unwrap())
                && self.zs_lst.last_mut().unwrap().try_add_to_end(bi)
            {
                return;
            }
        }

        if !self.zs_lst.is_empty()
            && self.free_item_lst.is_empty()
            && self.zs_lst.last().unwrap().in_range(bi)
            && bi.to_handle().index() - self.zs_lst.last().unwrap().end_bi.unwrap().index() <= 1
        {
            return;
        }
        self.add_to_free_lst(bi.to_handle(), bi.is_sure(), CPivotAlgo::OverSeg);
    }

    // 已完备
    fn try_combine(&mut self) {
        if !self.config.need_combine {
            while self.zs_lst.len() >= 2
                && self.zs_lst[self.zs_lst.len() - 2]
                    .can_combine(self.zs_lst.last().unwrap(), self.config.zs_combine_mode)
            {
                // 合并后删除最后一个
                let last = self.zs_lst.pop().unwrap();
                self.zs_lst.last_mut().unwrap().do_combine(last);
            }
        }
        //if !self.config.need_combine {
        //    return;
        //}
        //while self.zs_lst.len() >= 2 {
        //    let last_idx = self.zs_lst.len() - 1;
        //    let combine_result = {
        //        let (first, second) = self.zs_lst.split_at_mut(last_idx);
        //        first.last_mut().unwrap().do_combine(&second[0]) //, &self.config.zs_combine_mode)
        //    };
        //
        //    if combine_result {
        //        self.zs_lst.pop();
        //    } else {
        //        break;
        //    }
        //}
    }
}

impl<T> std::ops::Deref for CZsList<T> {
    type Target = Box<Vec<CZs<T>>>;

    fn deref(&self) -> &Self::Target {
        &self.zs_lst
    }
}

impl<T> std::ops::DerefMut for CZsList<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.zs_lst
    }
}
