use std::collections::HashMap;

use hashmap_macro::hashmap;

use crate::has_overlap;
use crate::AsHandle;
use crate::BspType;
use crate::CBSPointConfig;
use crate::CBspPoint;
use crate::CSeg;
use crate::CSegListChan;
use crate::CZs;
use crate::Handle;
use crate::IBspInfo;
use crate::ICalcMetric;
use crate::IParent;
use crate::Indexable;
use crate::LineType;
use crate::ToHandle;
use rustc_hash::FxHashMap;

// 基本思路：保存所有的历史买卖点
/// 买卖点列表,用于管理和计算各类买卖点
pub struct CBSPointList<T> {
    pub history: Box<Vec<CBspPoint<T>>>, // 历史买卖点记录
    pub lst: Vec<Handle<CBspPoint<T>>>,  // 当前有效的买卖点列表
    //bsp_dict: FxHashMap<usize, Handle<CBspPoint<T>>>,
    bsp1_lst: Vec<Handle<CBspPoint<T>>>, // 一类买卖点列表
    pub config: CBSPointConfig,          // 买卖点配置
    pub last_sure_pos: Option<usize>,    // 最后确定位置的索引
}

impl<T: LineType + IParent + IBspInfo + ToHandle + ICalcMetric> CBSPointList<T> {
    /// 创建新的买卖点列表实例
    ///
    /// # Arguments
    /// * `bs_point_config` - 买卖点配置参数
    ///
    /// # Returns
    /// 返回新的CBSPointList实例
    pub fn new(bs_point_config: CBSPointConfig) -> Self {
        CBSPointList {
            history: Box::new(Vec::with_capacity(10240)),
            lst: Vec::with_capacity(1024),
            //bsp_dict: FxHashMap::default(),
            bsp1_lst: Vec::with_capacity(1024),
            config: bs_point_config,
            last_sure_pos: None,
        }
    }

    /// 移除不确定的买卖点
    ///
    /// 根据last_sure_pos移除所有不确定的买卖点，包括lst和bsp1_lst中的点
    pub fn remove_unsure_bsp(&mut self) {
        self.lst.retain(|bsp| match self.last_sure_pos {
            Some(pos) => bsp.klu.index() <= pos,
            None => false,
        });

        //self.bsp_dict = self
        //    .lst
        //    .iter()
        //    .map(|bsp| (bsp.bi.get_end_klu().index(), *bsp))
        //    .collect();

        self.bsp1_lst.retain(|bsp| match self.last_sure_pos {
            Some(pos) => bsp.klu.index() <= pos,
            None => false,
        });
    }

    // 99% 完备
    /// 计算所有买卖点
    ///
    /// # Arguments
    /// * `bi_list` - 笔列表
    /// * `seg_list` - 线段列表
    ///
    /// 依次计算一类、二类、三类买卖点，并更新最后确定位置
    pub fn cal(&mut self, bi_list: &[T], seg_list: &CSegListChan<T>) {
        self.remove_unsure_bsp();

        self.cal_seg_bs1point(seg_list, bi_list);
        // 可以优化的地方，bsp1_bi_idx_dict，2类和3类都需要，因此可以仅仅计算一次
        self.cal_seg_bs2point(seg_list, bi_list);
        self.cal_seg_bs3point(seg_list, bi_list);
        self.update_last_pos(seg_list);
    }

    // 已完备
    /// 更新最后确定位置
    ///
    /// # Arguments
    /// * `seg_list` - 线段列表
    ///
    /// 从后向前查找第一个确定的线段，更新last_sure_pos
    pub fn update_last_pos(&mut self, seg_list: &CSegListChan<T>) {
        self.last_sure_pos = None;
        for seg in seg_list.iter().rev() {
            if seg.is_sure {
                self.last_sure_pos = Some(seg.end_bi.get_begin_klu().index());
                return;
            }
        }
    }

    // 已完备
    /// 判断线段是否需要计算买卖点
    ///
    /// # Arguments
    /// * `seg` - 待判断的线段
    ///
    /// # Returns
    /// 返回是否需要计算该线段的买卖点
    pub fn seg_need_cal(&self, seg: &CSeg<T>) -> bool {
        match self.last_sure_pos {
            Some(pos) => seg.end_bi.get_end_klu().index() > pos,
            None => true,
        }
    }

    // 80% 完备
    // TODO: 性能热点
    /// 添加买卖点
    ///
    /// # Arguments
    /// * `bs_type` - 买卖点类型
    /// * `bi` - 笔
    /// * `relate_bsp1` - 关联的一类买卖点
    /// * `is_target_bsp` - 是否为目标买卖点
    /// * `feature_dict` - 特征字典
    pub fn add_bs(
        &mut self,
        bs_type: BspType,
        bi: Handle<T>,
        relate_bsp1: Option<Handle<CBspPoint<T>>>,
        is_target_bsp: bool,
        feature_dict: Option<HashMap<String, Option<f64>>>,
    ) {
        let is_buy = bi.is_down();
        for exist_bsp in self.lst.iter() {
            if exist_bsp.klu.index() == bi.get_end_klu().index() {
                assert_eq!(exist_bsp.is_buy, is_buy);
                exist_bsp
                    .as_mut()
                    .add_another_bsp_prop(bs_type, relate_bsp1);
                return;
            }
        }

        //// 使哈希表O(1)查找替代遍历
        //let is_buy = bi.is_down();
        //let klu_index = bi.get_end_klu().index();
        //
        //// 使用哈希表O(1)查找替代遍历
        //if let Some(exist_bsp) = self.bsp_dict.get(&klu_index) {
        //    assert_eq!(exist_bsp.is_buy, is_buy);
        //    exist_bsp
        //        .as_mut()
        //        .add_another_bsp_prop(bs_type, relate_bsp1);
        //    return;
        //}

        let is_target_bsp = if !self
            .config
            .get_bs_config(is_buy)
            .target_types
            .contains(&bs_type)
        {
            false
        } else {
            is_target_bsp
        };

        if is_target_bsp || bs_type == BspType::T1 || bs_type == BspType::T1P {
            let bsp = CBspPoint::new(
                &self.history,
                self.history.len(),
                bi,
                is_buy,
                bs_type,
                relate_bsp1,
                feature_dict,
            );
            let bsp_handle = bsp.as_handle();
            self.history.push(bsp);

            if is_target_bsp {
                self.lst.push(bsp_handle);
                //self.bsp_dict.insert(klu_index, bsp_handle);
            }

            if bs_type == BspType::T1 || bs_type == BspType::T1P {
                self.bsp1_lst.push(bsp_handle);
            }
        }
    }

    // TODO: 性能热点
    /// 获取一类买卖点索引字典
    ///
    /// # Returns
    /// 返回一类买卖点的索引映射表
    fn bsp1_idx_dict(&self) -> FxHashMap<isize, Handle<CBspPoint<T>>> {
        self.bsp1_lst
            .iter()
            .map(|bsp| (bsp.bi.index() as isize, *bsp))
            .collect()
    }

    // 已完备
    /// 计算线段的一类买卖点
    ///
    /// # Arguments
    /// * `seg_list` - 线段列表
    /// * `bi_list` - 笔列表
    pub fn cal_seg_bs1point(&mut self, seg_list: &CSegListChan<T>, bi_list: &[T]) {
        for seg in seg_list.iter() {
            if !self.seg_need_cal(seg) {
                continue;
            }
            self.cal_single_bs1point(seg, bi_list);
        }
    }

    // 已完备
    /// 计算单个线段的一类买卖点
    ///
    /// # Arguments
    /// * `seg` - 待计算的线段
    /// * `bi_list` - 笔列表
    pub fn cal_single_bs1point(&mut self, seg: &CSeg<T>, bi_list: &[T]) {
        let is_buy = seg.is_down();
        let bsp_conf = self.config.get_bs_config(is_buy);

        let zs_cnt = if bsp_conf.bsp1_only_multibi_zs {
            seg.get_multi_bi_zs_cnt()
        } else {
            seg.zs_lst.len()
        };

        let is_target_bsp = bsp_conf.min_zs_cnt == 0 || zs_cnt >= bsp_conf.min_zs_cnt;

        if !seg.zs_lst.is_empty() {
            let last_zs = &seg.zs_lst.back().unwrap();

            let valid_last_zs = !last_zs.is_one_bi_zs()
                && ((last_zs.bi_out.is_some()
                    && last_zs.bi_out.unwrap().index() >= seg.end_bi.index())
                    || last_zs.bi_lst.last().unwrap().index() >= seg.end_bi.index())
                && seg.end_bi.index() - last_zs.get_bi_in().to_handle().index() > 2;

            if valid_last_zs {
                self.treat_bsp1(seg, is_buy, is_target_bsp);
            } else {
                self.treat_pz_bsp1(seg, is_buy, bi_list, is_target_bsp);
            }
        }
    }

    /// 处理一类买卖点
    ///
    /// # Arguments
    /// * `seg` - 线段
    /// * `is_buy` - 是否为买点
    /// * `is_target_bsp` - 是否为目标买卖点
    pub fn treat_bsp1(&mut self, seg: &CSeg<T>, is_buy: bool, is_target_bsp: bool) {
        let mut is_target_bsp = is_target_bsp;
        let bsp_conf = self.config.get_bs_config(is_buy);

        let last_zs = seg.zs_lst.back().unwrap();

        let (break_peak, _) = last_zs.out_bi_is_peak(seg.end_bi.index());

        if bsp_conf.bs1_peak && !break_peak {
            is_target_bsp = false;
        }

        let (is_diver, divergence_rate) = last_zs.is_divergence(bsp_conf, Some(&seg.end_bi));
        if !is_diver {
            is_target_bsp = false;
        }

        // TODO: add custom feature
        let feature_dict = Some(hashmap! {
            "divergence_rate".to_string() => divergence_rate,
        });

        self.add_bs(BspType::T1, seg.end_bi, None, is_target_bsp, feature_dict);
    }

    /// 处理盘整一类买卖点
    ///
    /// # Arguments
    /// * `seg` - 线段
    /// * `is_buy` - 是否为买点
    /// * `bi_list` - 笔列表
    /// * `is_target_bsp` - 是否为目标买卖点
    pub fn treat_pz_bsp1(
        &mut self,
        seg: &CSeg<T>,
        is_buy: bool,
        bi_list: &[T],
        is_target_bsp: bool,
    ) {
        let mut is_target_bsp = is_target_bsp;
        let bsp_conf = self.config.get_bs_config(is_buy);

        let last_bi = &seg.end_bi;

        let pre_bi_index = last_bi.index() as isize - 2;
        let pre_bi_index = if pre_bi_index < 0 {
            (pre_bi_index + bi_list.len() as isize) as usize
        } else {
            pre_bi_index as usize
        };
        let pre_bi = &bi_list[pre_bi_index];

        if last_bi.seg_idx() != pre_bi.seg_idx() {
            return;
        }

        if last_bi.direction() != seg.dir {
            return;
        }

        // 创新低?
        if last_bi.is_down() && last_bi.low() > pre_bi.low() {
            return;
        }

        // 创新高?
        if last_bi.is_up() && last_bi.high() < pre_bi.high() {
            return;
        }

        let in_metric = pre_bi.cal_macd_metric(&bsp_conf.macd_algo, false);
        let out_metric = last_bi.cal_macd_metric(&bsp_conf.macd_algo, true);

        let (is_diver, divergence_rate) = (
            out_metric <= bsp_conf.divergence_rate * in_metric,
            out_metric / (in_metric + 1e-7),
        );

        if !is_diver {
            is_target_bsp = false;
        }

        let feature_dict = Some(hashmap! {
            "divergence_rate".to_string() => Some(divergence_rate),
            "bsp1_bi_amp".to_string() => Some(last_bi.amp()),
        });

        self.add_bs(
            BspType::T1P,
            last_bi.to_handle(),
            None,
            is_target_bsp,
            feature_dict,
        );
    }

    // 已完备
    /// 计算线段的二类买卖点
    ///
    /// # Arguments
    /// * `seg_list` - 线段列表
    /// * `bi_list` - 笔列表
    pub fn cal_seg_bs2point(&mut self, seg_list: &CSegListChan<T>, bi_list: &[T]) {
        let bsp1_bi_idx_dict = self.bsp1_idx_dict();

        for seg in seg_list.iter() {
            let is_buy = seg.is_down();
            let config = self.config.get_bs_config(is_buy);
            if !config.target_types.contains(&BspType::T2)
                && !config.target_types.contains(&BspType::T2S)
            {
                continue;
            }

            self.treat_bsp2(seg, &bsp1_bi_idx_dict, seg_list, bi_list);
        }
    }

    // 已完备
    /// 处理二类买卖点
    ///
    /// # Arguments
    /// * `seg` - 当前线段
    /// * `bsp1_bi_idx_dict` - 一类买卖点索引字典
    /// * `seg_list` - 线段列表
    /// * `bi_list` - 笔列表
    pub fn treat_bsp2(
        &mut self,
        seg: &CSeg<T>,
        bsp1_bi_idx_dict: &FxHashMap<isize, Handle<CBspPoint<T>>>,
        seg_list: &CSegListChan<T>,
        bi_list: &[T],
    ) {
        if !self.seg_need_cal(seg) {
            return;
        }

        let (bsp1_bi_idx, bsp2_bi, break_bi, real_bsp1, bsp_conf, is_buy) = if seg_list.len() > 1 {
            let bsp_conf = self.config.get_bs_config(seg.is_down());
            let bsp1_bi = &seg.end_bi;
            let bsp1_bi_idx = Some(bsp1_bi.index());
            let real_bsp1 = bsp1_bi_idx_dict.get(&(bsp1_bi.index() as isize)).cloned();
            if bsp1_bi.index() + 2 >= bi_list.len() {
                return;
            }
            let break_bi = &bi_list[bsp1_bi.index() + 1];
            let bsp2_bi = &bi_list[bsp1_bi.index() + 2];
            (
                bsp1_bi_idx,
                bsp2_bi,
                break_bi,
                real_bsp1,
                bsp_conf,
                seg.is_down(),
            )
        } else {
            let bsp_conf = self.config.get_bs_config(seg.is_up());
            let bsp1_bi_idx = None;
            let real_bsp1 = None;
            if bi_list.len() == 1 {
                return;
            }
            let bsp2_bi = &bi_list[1];
            let break_bi = &bi_list[0];
            (
                bsp1_bi_idx,
                bsp2_bi,
                break_bi,
                real_bsp1,
                bsp_conf,
                seg.is_up(),
            )
        };

        if bsp_conf.bsp2_follow_1 //&& !bsp1_bi_idx_dict.contains_key(&bsp1_bi_idx)
        && bsp1_bi_idx.map_or(true, |idx| !bsp1_bi_idx_dict.contains_key(&(idx as isize)))
        {
            return;
        }

        let retrace_rate = bsp2_bi.amp() / break_bi.amp();
        let bsp2_flag = retrace_rate <= bsp_conf.max_bs2_rate;
        if bsp2_flag {
            let feature_dict = Some(hashmap! {
                "bsp2_retrace_rate".to_string() => Some(retrace_rate),
                "bsp2_break_bi_amp".to_string() => Some(break_bi.amp()),
                "bsp2_bi_amp".to_string() => Some(bsp2_bi.amp()),
            });

            self.add_bs(
                BspType::T2,
                bsp2_bi.to_handle(),
                real_bsp1,
                true,
                feature_dict,
            );
        } else if bsp_conf.bsp2s_follow_2 {
            return;
        }

        if !self
            .config
            .get_bs_config(seg.is_down())
            .target_types
            .contains(&BspType::T2S)
        {
            return;
        }

        self.treat_bsp2s(seg_list, bi_list, bsp2_bi, break_bi, real_bsp1, is_buy);
    }

    // 已完备
    /// 处理类二买卖点
    ///
    /// # Arguments
    /// * `seg_list` - 线段列表
    /// * `bi_list` - 笔列表
    /// * `bsp2_bi` - 二类买卖点笔
    /// * `break_bi` - 突破笔
    /// * `real_bsp1` - 实际的一类买卖点
    /// * `is_buy` - 是否为买点
    pub fn treat_bsp2s(
        &mut self,
        seg_list: &CSegListChan<T>,
        bi_list: &[T],
        bsp2_bi: &T,
        break_bi: &T,
        real_bsp1: Option<Handle<CBspPoint<T>>>,
        is_buy: bool,
    ) {
        // 1. 提前获取常用值
        let bsp_conf = self.config.get_bs_config(is_buy);
        let bsp2_bi_idx = bsp2_bi.to_handle().index();
        let bsp2_bi_seg_idx = bsp2_bi.seg_idx().unwrap();
        let break_bi_end_val = break_bi.get_end_val();
        let break_bi_amp = break_bi.amp();
        let max_bs2_rate = bsp_conf.max_bs2_rate;

        // 2. 提前计算bsp2_bi的high/low值
        let bsp2_bi_high = bsp2_bi.high();
        let bsp2_bi_low = bsp2_bi.low();

        let mut bias = 2;
        let mut overlap_low = None;
        let mut overlap_high = None;

        // 3. 提前计算循环终止条件
        let max_index = bi_list.len();
        let max_level = bsp_conf.max_bsp2s_lv.unwrap_or(usize::MAX);

        while bsp2_bi_idx + bias < max_index {
            // 4. 提前获取当前处理的笔
            let bsp2s_bi = &bi_list[bsp2_bi_idx + bias];
            let bsp2s_bi_seg_idx = match bsp2s_bi.seg_idx() {
                Some(idx) => idx,
                None => break,
            };

            // 5. 提前检查level限制
            if bias / 2 > max_level {
                break;
            }

            // 6. 合并段落索引检查条件
            if bsp2s_bi_seg_idx != bsp2_bi_seg_idx
                && (bsp2s_bi_seg_idx < seg_list.len() - 1
                    || bsp2s_bi_seg_idx - bsp2_bi_seg_idx >= 2
                    || seg_list[bsp2_bi_seg_idx].is_sure)
            {
                break;
            }

            // 7. 提前获bsp2s_bi的high/low值
            let bsp2s_bi_high = bsp2s_bi.high();
            let bsp2s_bi_low = bsp2s_bi.low();

            // 8. 优化重叠检查逻辑
            if bias == 2 {
                if !has_overlap(
                    bsp2_bi_low,
                    bsp2_bi_high,
                    bsp2s_bi_low,
                    bsp2s_bi_high,
                    false,
                ) {
                    break;
                }
                overlap_low = Some(bsp2_bi_low.max(bsp2s_bi_low));
                overlap_high = Some(bsp2_bi_high.min(bsp2s_bi_high));
            } else if !has_overlap(
                overlap_low.unwrap(),
                overlap_high.unwrap(),
                bsp2s_bi_low,
                bsp2s_bi_high,
                false,
            ) {
                break;
            }

            // 9. 提前计算回撤率
            let retrace_rate = (bsp2s_bi.get_end_val() - break_bi_end_val).abs() / break_bi_amp;

            if bsp2s_break_bsp1(bsp2s_bi, break_bi) || retrace_rate > max_bs2_rate {
                break;
            }

            // 10. 使用静态字符串避免重复分配
            let feature_dict = Some(hashmap! {
                "bsp2_retrace_rate".to_string() => Some(retrace_rate),
                "bsp2_break_bi_amp".to_string() => Some(break_bi_amp),
                "bsp2_bi_amp".to_string() => Some(bsp2_bi.amp()),
                "bsp2s_lv".to_string() => Some(bias as f64 / 2.0),
            });

            self.add_bs(
                BspType::T2S,
                bsp2s_bi.to_handle(),
                real_bsp1,
                true,
                feature_dict,
            );
            bias += 2;
        }
    }

    // 已完备
    /// 计算线段的三类买卖点
    ///
    /// # Arguments
    /// * `seg_list` - 线段列表
    /// * `bi_list` - 笔列表
    pub fn cal_seg_bs3point(&mut self, seg_list: &CSegListChan<T>, bi_list: &[T]) {
        // 1. 提前获取一类买卖点字典
        let bsp1_bi_idx_dict = self.bsp1_idx_dict();

        // 2. 提前判断列表长度
        let seg_list_len = seg_list.len();
        let is_multi_seg = seg_list_len > 1;

        for seg in seg_list.iter() {
            // 3. 提前检查是否需要计算
            if !self.seg_need_cal(seg) {
                continue;
            }

            // 4. 提前获取买卖方向和配置
            let is_buy = seg.is_down();
            let config = self.config.get_bs_config(is_buy);

            // 5. 提前检查目标类型
            let has_target_types = config.target_types.contains(&BspType::T3A)
                || config.target_types.contains(&BspType::T3B);
            if !has_target_types {
                continue;
            }

            // 6. 根据段落数量分别处理
            let (next_seg, next_seg_idx, bsp1_bi, real_bsp1, bsp1_bi_idx) = if is_multi_seg {
                // 多段情况
                let bsp1_bi = seg.end_bi;
                let bsp1_bi_idx = Some(bsp1_bi.index());
                let real_bsp1 = bsp1_bi_idx_dict.get(&(bsp1_bi.index() as isize)).cloned();
                let next_seg_idx = seg.index() + 1;
                let next_seg = seg.to_handle().next();
                (
                    next_seg,
                    next_seg_idx,
                    Some(bsp1_bi),
                    real_bsp1,
                    bsp1_bi_idx,
                )
            } else {
                // 单段情况
                let next_seg = Some(seg.to_handle());
                (next_seg, seg.index(), None, None, None)
            };

            // 7. 检查一类买卖点关联
            if config.bsp3_follow_1
                && bsp1_bi_idx.map_or(true, |idx| !bsp1_bi_idx_dict.contains_key(&(idx as isize)))
            {
                continue;
            }

            // 8. 处理后续买卖点
            if let Some(next_seg) = next_seg {
                self.treat_bsp3_after(
                    seg_list,
                    next_seg,
                    is_buy,
                    bi_list,
                    real_bsp1,
                    bsp1_bi_idx,
                    next_seg_idx,
                );
            }

            // 9. 处理之前买卖点
            self.treat_bsp3_before(
                seg_list,
                seg.to_handle(),
                next_seg,
                bsp1_bi,
                is_buy,
                bi_list,
                real_bsp1,
                next_seg_idx,
            );
        }
    }

    /// 处理三类买卖点（后续）
    ///
    /// # Arguments
    /// * `seg_list` - 线段列表
    /// * `next_seg` - 下一个线段
    /// * `is_buy` - 是否为买点
    /// * `bi_list` - 笔列表
    /// * `real_bsp1` - 实际的一类买卖点
    /// * `bsp1_bi_idx` - 一类买卖点索引
    /// * `next_seg_idx` - 下一个线段索引
    #[allow(clippy::too_many_arguments)]
    pub fn treat_bsp3_after(
        &mut self,
        seg_list: &CSegListChan<T>,
        next_seg: Handle<CSeg<T>>,
        is_buy: bool,
        bi_list: &[T],
        real_bsp1: Option<Handle<CBspPoint<T>>>,
        bsp1_bi_idx: Option<usize>,
        next_seg_idx: usize,
    ) {
        // 1. 提前获取并检查first_zs
        let first_zs = match next_seg.get_first_multi_bi_zs() {
            Some(zs) => zs,
            None => return,
        };

        // 2. 提前获取配置
        let bsp_conf = self.config.get_bs_config(is_buy);

        // 3. 检查strict_bsp3条件
        if bsp_conf.strict_bsp3 {
            let bi_in_index = first_zs.get_bi_in().to_handle().index();
            if bi_in_index != bsp1_bi_idx.unwrap_or(0) + 1 {
                return;
            }
        }

        // 4. 获取并检查bi_out
        let bi_out_index = match first_zs.bi_out {
            Some(bi_out) => bi_out.index(),
            None => return,
        };

        if bi_out_index + 1 >= bi_list.len() {
            return;
        }

        // 5. 获取bsp3_bi并提前计算常用值
        let bsp3_bi = &bi_list[bi_out_index + 1];
        let bsp3_parent_seg_idx = bsp3_bi.parent_seg_idx();
        let bsp3_seg_idx = bsp3_bi.seg_idx();
        let seg_list_len = seg_list.len();
        let next_seg_index = next_seg.index();

        // 6. 检查parent_seg_idx条件
        match bsp3_parent_seg_idx {
            None => {
                if next_seg_index != seg_list_len - 1 {
                    return;
                }
            }
            Some(parent_idx) if parent_idx != next_seg_index && parent_idx < seg_list_len => {
                let parent_seg = &seg_list[parent_idx];
                if parent_seg.bi_list.len() >= 3 {
                    return;
                }
            }
            _ => {}
        }

        // 7. 检查其他条件
        if bsp3_bi.direction() == next_seg.dir {
            return;
        }

        if bsp3_seg_idx != Some(next_seg_idx) && next_seg_idx < seg_list_len - 2 {
            return;
        }

        if bsp3_back2zs(bsp3_bi, first_zs) {
            return;
        }

        // 8. 检查peak条件
        if bsp_conf.bsp3_peak && !bsp3_break_zspeak(bsp3_bi, first_zs) {
            return;
        }

        // 9. 计算中枢高度和特征字典
        let zs_height = (first_zs.high - first_zs.low) / first_zs.low;
        let bsp3_bi_amp = bsp3_bi.amp();

        let feature_dict = Some(hashmap! {
            "bsp3_zs_height".to_string() => Some(zs_height),
            "bsp3_bi_amp".to_string() => Some(bsp3_bi_amp),
        });

        // 10. 添加买卖点
        self.add_bs(
            BspType::T3A,
            bsp3_bi.to_handle(),
            real_bsp1,
            true,
            feature_dict,
        );
    }

    /// 处理三类买卖点（之前）
    ///
    /// # Arguments
    /// * `seg_list` - 线段列表
    /// * `seg` - 当前线段
    /// * `next_seg` - 下一个线段
    /// * `bsp1_bi` - 一类买卖点笔
    /// * `is_buy` - 是否为买点
    /// * `bi_list` - 笔列表
    /// * `real_bsp1` - 实际的一类买卖点
    /// * `next_seg_idx` - 下一个线段索引
    #[allow(clippy::too_many_arguments)]
    pub fn treat_bsp3_before(
        &mut self,
        seg_list: &CSegListChan<T>,
        seg: Handle<CSeg<T>>,
        next_seg: Option<Handle<CSeg<T>>>,
        bsp1_bi: Option<Handle<T>>,
        is_buy: bool,
        bi_list: &[T],
        real_bsp1: Option<Handle<CBspPoint<T>>>,
        next_seg_idx: usize,
    ) {
        let cmp_zs = seg.get_final_multi_bi_zs();
        if cmp_zs.is_none() {
            return;
        }
        if bsp1_bi.is_none() {
            return;
        }
        let bsp_conf = self.config.get_bs_config(is_buy);
        if bsp_conf.strict_bsp3
            && (cmp_zs.unwrap().bi_out.is_none()
                || cmp_zs.unwrap().bi_out.unwrap().index() != bsp1_bi.unwrap().to_handle().index())
        {
            return;
        }
        let end_bi_idx = cal_bsp3_bi_end_idx(next_seg);
        for bsp3_bi in bi_list
            .iter()
            .skip(bsp1_bi.unwrap().to_handle().index() + 2)
            .step_by(2)
        {
            if bsp3_bi.to_handle().index() > end_bi_idx {
                break;
            }
            if let Some(bsp3_bi_seg_idx) = bsp3_bi.seg_idx() {
                if bsp3_bi_seg_idx != next_seg_idx && bsp3_bi_seg_idx < seg_list.len() - 1 {
                    break;
                }
            }
            if bsp3_back2zs(bsp3_bi, cmp_zs.unwrap()) {
                continue;
            }

            let feature_dict = Some(hashmap! {
                "bsp3_zs_height".to_string() => Some((cmp_zs.unwrap().high - cmp_zs.unwrap().low)/cmp_zs.unwrap().low),
                "bsp3_bi_amp".to_string() => Some(bsp3_bi.amp()),
            });
            self.add_bs(
                BspType::T3B,
                bsp3_bi.to_handle(),
                real_bsp1,
                true,
                feature_dict,
            );
            break;
        }
    }

    /*pub fn getLastestBspList(&self) -> Vec<Handle<BspPoint<T>>> {
        if self.lst.is_empty() {
            return Vec::new();
        }
        let mut lst = self.lst.iter().map(|x| x.as_handle()).collect::<Vec<_>>();
        lst.sort_by(|a, b| b.bi.index().cmp(&a.bi.index()));
        lst
    }*/
}

/// 判断类二买卖点是否突破一类买卖点
fn bsp2s_break_bsp1<T: LineType>(bsp2s_bi: &T, bsp2_break_bi: &T) -> bool {
    // 提前获取方向,避免重复调用
    let is_down = bsp2s_bi.is_down();

    match is_down {
        true => bsp2s_bi.low() < bsp2_break_bi.low(),
        false => bsp2s_bi.high() > bsp2_break_bi.high(),
    }
}

/// 判断三类买卖点是否回中枢
fn bsp3_back2zs<T: LineType>(bsp3_bi: &T, zs: Handle<CZs<T>>) -> bool {
    // 提前获取方向和关键值
    let is_down = bsp3_bi.is_down();
    let zs_high = zs.high;
    let zs_low = zs.low;

    match is_down {
        true => bsp3_bi.low() < zs_high,
        false => bsp3_bi.high() > zs_low,
    }
}

/// 判断三类买卖点是否突破中枢顶底
fn bsp3_break_zspeak<T: LineType>(bsp3_bi: &T, zs: Handle<CZs<T>>) -> bool {
    // 提前获取方向和关键值
    let is_down = bsp3_bi.is_down();
    let peak_high = zs.peak_high;
    let peak_low = zs.peak_low;

    match is_down {
        true => bsp3_bi.high() >= peak_high,
        false => bsp3_bi.low() <= peak_low,
    }
}

/// 计算三类买卖点的笔结束索引
fn cal_bsp3_bi_end_idx<T: LineType>(seg: Option<Handle<CSeg<T>>>) -> usize {
    // 使用match处理Option
    let seg = match seg {
        Some(s) => s,
        None => return usize::MAX,
    };

    // 提前检查条件
    if seg.get_multi_bi_zs_cnt() == 0 && seg.next().is_none() {
        return usize::MAX;
    }

    // 获取初始end_bi_idx
    let mut end_bi_idx = seg.end_bi.index() - 1;

    // 使用迭代器方法查找合适的中枢
    if let Some(valid_zs) = seg
        .zs_lst
        .iter()
        .filter(|zs| !zs.is_one_bi_zs())
        .find(|zs| zs.bi_out.is_some())
    {
        end_bi_idx = valid_zs.bi_out.unwrap().index();
    }

    end_bi_idx
}

// 可选：为常用的组合条件创建新的工具函数
#[allow(dead_code)]
fn is_valid_zs<T: LineType>(zs: &CZs<T>) -> bool {
    !zs.is_one_bi_zs() && zs.bi_out.is_some()
}

/// 实现 Deref trait,允许直接访问内部的买卖点列表
impl<T> std::ops::Deref for CBSPointList<T> {
    type Target = Vec<Handle<CBspPoint<T>>>;

    fn deref(&self) -> &Self::Target {
        &self.lst
    }
}

/// 实现 DerefMut trait,允许直接修改内部的买卖点列表
impl<T> std::ops::DerefMut for CBSPointList<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.lst
    }
}
