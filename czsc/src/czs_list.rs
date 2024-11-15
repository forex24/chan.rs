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
    pub last_sure_pos: Option<usize>,
}

impl<T: LineType + IParent + ToHandle + ICalcMetric> CZsList<T> {
    /// 创建新的中枢列表实例
    ///
    /// # Arguments
    /// * `zs_config` - 中枢配置参数
    ///
    /// # Returns
    /// 返回新的CZsList实例
    pub fn new(zs_config: CZsConfig) -> Self {
        CZsList {
            zs_lst: Box::<Vec<CZs<T>>>::default(),
            config: zs_config,
            free_item_lst: Vec::new(),
            last_sure_pos: None,
        }
    }

    // 已完备
    /// 更新最后确定位置
    ///
    /// # Arguments
    /// * `seg_list` - 线段列表
    ///
    /// 从后向前查找第一个确定的线段，更新last_sure_pos
    fn update_last_pos(&mut self, seg_list: &CSegListChan<T>) {
        self.last_sure_pos = None;

        for seg in seg_list.iter().rev() {
            if seg.is_sure {
                self.last_sure_pos = Some(seg.start_bi.index());
                return;
            }
        }
    }

    // 已完备
    /// 判断线段是否需要计算中枢
    ///
    /// # Arguments
    /// * `seg` - 待判断的线段
    ///
    /// # Returns
    /// 返回是否需要计算该线段的中枢
    fn seg_need_cal(&self, seg: &CSeg<T>) -> bool {
        match self.last_sure_pos {
            Some(pos) => seg.start_bi.index() >= pos,
            None => true,
        }
    }

    // 已完备
    /// 添加元素到自由列表并尝试构建中枢
    ///
    /// # Arguments
    /// * `item` - 待添加的元素
    /// * `is_sure` - 是否确定
    /// * `zs_algo` - 中枢算法
    fn add_to_free_lst(&mut self, item: Handle<T>, is_sure: bool, zs_algo: CPivotAlgo) {
        if !self.free_item_lst.is_empty()
            && item.index() == self.free_item_lst.last().unwrap().index()
        {
            // 防止笔新高或新低的更新带来bug
            self.free_item_lst.pop();
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
    /// 清空自由列表
    fn clear_free_lst(&mut self) {
        self.free_item_lst.clear();
    }

    // 已完备
    /// 更新中枢列表
    ///
    /// # Arguments
    /// * `bi` - 笔
    /// * `is_sure` - 是否确定
    fn update(&mut self, bi: Handle<T>, is_sure: bool) {
        if self.free_item_lst.is_empty() && self.try_add_to_end(bi) {
            // zs_combine_mode=peak合并模式下会触发生效，=zs合并一定无效返回
            self.try_combine(); // 新形成的中枢尝试和之前的中枢合并
            return;
        }
        self.add_to_free_lst(bi, is_sure, CPivotAlgo::Normal);
    }

    /// 尝试将笔添加到最后一个中枢
    ///
    /// # Arguments
    /// * `bi` - 待添加的笔
    ///
    /// # Returns
    /// 返回是否成功添加
    fn try_add_to_end(&mut self, bi: Handle<T>) -> bool {
        if self.zs_lst.is_empty() {
            return false;
        }
        self.zs_lst.last_mut().unwrap().try_add_to_end(bi)
    }

    /// 从笔范围添加中枢
    ///
    /// # Arguments
    /// * `seg_bi_lst` - 线段包含的笔列表
    /// * `seg_dir` - 线段方向
    /// * `seg_is_sure` - 线段是否确定
    fn add_zs_from_bi_range(&mut self, seg_bi_lst: &[T], seg_dir: Direction, seg_is_sure: bool) {
        let mut deal_bi_cnt = 0;
        for bi in seg_bi_lst {
            if bi.direction() == seg_dir {
                continue;
            }

            if deal_bi_cnt < 1 {
                // 防止try_add_to_end执行到上一个线段中枢里面去
                self.add_to_free_lst(bi.to_handle(), seg_is_sure, CPivotAlgo::Normal);
                deal_bi_cnt += 1;
            } else {
                self.update(bi.to_handle(), seg_is_sure);
            }
        }
    }

    /// 尝试构建中枢
    ///
    /// # Arguments
    /// * `is_sure` - 是否确定
    /// * `zs_algo` - 中枢算法
    ///
    /// # Returns
    /// 返回可能构建的中枢
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
                    //self.free_item_lst = lst[1..].to_vec();
                    return None;
                } else {
                    lst
                }
            }
            _ => lst,
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
    /// 计算笔中枢
    ///
    /// # Arguments
    /// * `bi_lst` - 笔列表
    /// * `seg_lst` - 线段列表
    pub fn cal_bi_zs(&mut self, bi_lst: &[T], seg_lst: &CSegListChan<T>) {
        while let Some(last) = self.zs_lst.last() {
            // 检查 last_sure_pos 是否有值
            if let Some(last_sure_pos) = self.last_sure_pos {
                // 比较 last.begin_bi.idx 和 last_sure_pos
                if last.begin_bi.index() >= last_sure_pos {
                    self.zs_lst.pop();
                } else {
                    break; // 如果条件不满足，退出循环
                }
            } else {
                self.zs_lst.clear(); // 如果 last_sure_pos 没有值，退出循环
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
                assert!(!self.config.one_bi_zs);
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
    /// 更新跨线段中枢
    ///
    /// # Arguments
    /// * `bi` - 笔
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
    /// 尝试合并中枢
    ///
    /// 根据配置的合并模式，尝试合并相邻的中枢
    fn try_combine(&mut self) {
        if self.config.need_combine {
            while self.zs_lst.len() >= 2
                && self.zs_lst[self.zs_lst.len() - 2]
                    .can_combine(self.zs_lst.last().unwrap(), self.config.zs_combine_mode)
            {
                // 合并后删除最后一个
                let last = self.zs_lst.pop().unwrap();
                self.zs_lst.last_mut().unwrap().do_combine(last);
            }
        }
    }
}

impl<T> std::ops::Deref for CZsList<T> {
    type Target = Box<Vec<CZs<T>>>;

    /// 实现Deref trait，允许直接访问内部的中枢列表
    fn deref(&self) -> &Self::Target {
        &self.zs_lst
    }
}

impl<T> std::ops::DerefMut for CZsList<T> {
    /// 实现DerefMut trait，允许直接修改内部的中枢列表
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.zs_lst
    }
}
