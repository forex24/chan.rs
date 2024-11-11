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

// 基本思路：保存所有的历史买卖点
/// 买卖点列表,用于管理和计算各类买卖点
pub struct CBSPointList<T> {
    pub history: Box<Vec<CBspPoint<T>>>, // 历史买卖点记录
    pub lst: Vec<Handle<CBspPoint<T>>>,  // 当前有效的买卖点列表
    //bsp_dict: HashMap<usize, Handle<CBspPoint<T>>>,
    bsp1_lst: Vec<Handle<CBspPoint<T>>>, // 一类买卖点列表
    pub config: CBSPointConfig,          // 买卖点配置
    pub last_sure_pos: Option<usize>,    // 最后确定位置的索引
}

impl<T: LineType + IParent + IBspInfo + ToHandle + ICalcMetric> CBSPointList<T> {
    /// 创建新的买卖点列表实例
    pub fn new(bs_point_config: CBSPointConfig) -> Self {
        CBSPointList {
            history: Box::new(Vec::with_capacity(10240)),
            lst: Vec::with_capacity(1024),
            //bsp_dict: HashMap::new(),
            bsp1_lst: Vec::with_capacity(1024),
            config: bs_point_config,
            last_sure_pos: None,
        }
    }

    pub fn remove_unsure_bsp(&mut self) {
        self.lst.retain(|bsp| match self.last_sure_pos {
            Some(pos) => bsp.klu.index() <= pos,
            None => false,
        });
        //self.bsp_dict = self
        //    .lst
        //    .iter()
        //    .map(|bsp| (bsp.bi.get_end_klu().index(), bsp.clone()))
        //    .collect();
        self.bsp1_lst.retain(|bsp| match self.last_sure_pos {
            Some(pos) => bsp.klu.index() <= pos,
            None => false,
        });
    }

    // 99% 完备
    /// 计算所有买卖点
    pub fn cal(&mut self, bi_list: &[T], seg_list: &CSegListChan<T>) {
        self.remove_unsure_bsp();

        self.cal_seg_bs1point(seg_list, bi_list);
        self.cal_seg_bs2point(seg_list, bi_list);
        self.cal_seg_bs3point(seg_list, bi_list);
        self.update_last_pos(seg_list);
    }

    // 已完备
    /// 更新最后确定位置
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
    pub fn seg_need_cal(&self, seg: &CSeg<T>) -> bool {
        match self.last_sure_pos {
            Some(pos) => seg.end_bi.get_end_klu().index() > pos,
            None => true,
        }
    }

    // 80% 完备
    // TODO: 性能热点
    /// 添加买卖点
    pub fn add_bs(
        &mut self,
        bs_type: BspType,                                   // 买卖点类型
        bi: Handle<T>,                                      // 笔
        relate_bsp1: Option<Handle<CBspPoint<T>>>,          // 关联的一类买卖点
        is_target_bsp: bool,                                // 是否为目标买卖点
        feature_dict: Option<HashMap<String, Option<f64>>>, // 特征字典
    ) {
        //let is_buy = bi.is_down();
        //let end_klu_idx = bi.get_end_klu().index();
        //
        //if let Some(exist_bsp) = self.bsp_dict.get(&end_klu_idx) {
        //    assert_eq!(exist_bsp.is_buy, is_buy);
        //    exist_bsp
        //        .borrow_mut()
        //        .add_another_bsp_prop(bs_type, relate_bsp1);
        //    //if let Some(features) = feature_dict {
        //    //    exist_bsp.borrow_mut().add_feat(features, None);
        //    //}
        //    return;
        //}
        //
        //let is_target_bsp = if !self
        //    .config
        //    .get_bs_config(is_buy)
        //    .target_types
        //    .contains(&bs_type)
        //{
        //    false
        //} else {
        //    is_target_bsp
        //};
        //
        //if is_target_bsp || matches!(bs_type, BspType::T1 | BspType::T1P) {
        //    let bsp = CBspPoint::new(bi, is_buy, bs_type, relate_bsp1, feature_dict);
        //
        //    if is_target_bsp {
        //        self.lst.push(bsp.clone());
        //        self.bsp_dict.insert(end_klu_idx, bsp.clone());
        //    }
        //    if matches!(bs_type, BspType::T1 | BspType::T1P) {
        //        self.bsp1_lst.push(bsp.clone());
        //    }
        //}

        //for exist_bsp in self.lst.iter() {
        //    if exist_bsp.klu.index() == bi.get_end_klu().index() {
        //        assert_eq!(exist_bsp.is_buy, is_buy);
        //        exist_bsp
        //            .borrow_mut()
        //            .add_another_bsp_prop(bs_type, relate_bsp1);
        //        return;
        //    }
        //}

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
                //self.bsp_dict.insert(bi.get_end_klu().index(), bsp.clone());
            }

            if bs_type == BspType::T1 || bs_type == BspType::T1P {
                self.bsp1_lst.push(bsp_handle);
            }
        }
    }

    // TODO: 性能热点
    /// 获取一类买卖点索引字典
    fn bsp1_idx_dict(&self) -> HashMap<isize, Handle<CBspPoint<T>>> {
        self.bsp1_lst
            .iter()
            .map(|bsp| (bsp.bi.index() as isize, *bsp))
            .collect()
    }

    // 已完备
    /// 计算线段的一类买卖点
    pub fn cal_seg_bs1point(&mut self, seg_list: &CSegListChan<T>, bi_list: &[T]) {
        for seg in seg_list.iter() {
            if !self.seg_need_cal(seg) {
                continue;
            }
            self.cal_single_bs1point(seg, bi_list);
        }
    }

    // 已完备
    pub fn cal_single_bs1point(&mut self, seg: &CSeg<T>, bi_list: &[T]) {
        let is_buy = seg.is_down();
        let bsp_conf = self.config.get_bs_config(is_buy);

        let zs_cnt = if bsp_conf.bsp1_only_multibi_zs {
            seg.get_multi_bi_zs_cnt()
        } else {
            seg.zs_lst.len()
        };

        let is_target_bsp = bsp_conf.min_zs_cnt == 0 || zs_cnt >= bsp_conf.min_zs_cnt;

        //if !seg.zs_lst.is_empty()
        //    && !seg.zs_lst.iter().last().unwrap().is_one_bi_zs()
        //    && ((seg.zs_lst.iter().last().unwrap().bi_out.is_some()
        //        && seg.zs_lst.iter().last().unwrap().bi_out.unwrap().index() >= seg.end_bi.index())
        //        || seg
        //            .zs_lst
        //            .iter()
        //            .last()
        //            .unwrap()
        //            .bi_lst
        //            .last()
        //            .unwrap()
        //            .index()
        //            >= seg.end_bi.index())
        //    && seg.end_bi.index()
        //        - seg
        //            .zs_lst
        //            .iter()
        //            .last()
        //            .unwrap()
        //            .get_bi_in()
        //            .to_handle()
        //            .index()
        //        > 2
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
    pub fn treat_bsp2(
        &mut self,
        seg: &CSeg<T>,                                           // 当前线段
        bsp1_bi_idx_dict: &HashMap<isize, Handle<CBspPoint<T>>>, // 一类买卖点索引字典
        seg_list: &CSegListChan<T>,                              // 线段列表
        bi_list: &[T],                                           // 笔列表
    ) {
        if !self.seg_need_cal(seg) {
            return;
        }

        let (bsp1_bi_idx, bsp2_bi, break_bi, real_bsp1, bsp_conf, is_buy) = if seg_list.len() > 1 {
            let bsp_conf = self.config.get_bs_config(seg.is_down());
            let bsp1_bi = &seg.end_bi;
            let bsp1_bi_idx = bsp1_bi.index() as isize;
            let real_bsp1 = bsp1_bi_idx_dict.get(&bsp1_bi_idx).cloned();
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
            let bsp1_bi_idx = -1;
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

        if bsp_conf.bsp2_follow_1 && !bsp1_bi_idx_dict.contains_key(&bsp1_bi_idx) {
            //  check bsp2_follow_1
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
    pub fn treat_bsp2s(
        &mut self,
        seg_list: &CSegListChan<T>,              // 线段列表
        bi_list: &[T],                           // 笔列表
        bsp2_bi: &T,                             // 二类买卖点笔
        break_bi: &T,                            // 突破笔
        real_bsp1: Option<Handle<CBspPoint<T>>>, // 关联的一类买卖点
        is_buy: bool,                            // 是否为买点
    ) {
        //let bsp_conf = self.config.get_bs_config(is_buy);
        let mut bias = 2;
        let mut _low = None;
        let mut _high = None;

        // 计算类二
        while bsp2_bi.to_handle().index() + bias < bi_list.len() {
            let bsp2s_bi = &bi_list[bsp2_bi.to_handle().index() + bias];

            assert!(bsp2s_bi.seg_idx().is_some() && bsp2_bi.seg_idx().is_some());

            let bsp_conf = self.config.get_bs_config(is_buy);
            if let Some(max_bsp2s_lv) = bsp_conf.max_bsp2s_lv {
                if bias / 2 > max_bsp2s_lv {
                    break;
                }
            }

            if bsp2s_bi.seg_idx().unwrap() != bsp2_bi.seg_idx().unwrap()
                && (bsp2s_bi.seg_idx().unwrap() < seg_list.len() - 1
                    || bsp2s_bi.seg_idx().unwrap() - bsp2_bi.seg_idx().unwrap() >= 2
                    || seg_list[bsp2_bi.seg_idx().unwrap()].is_sure)
            {
                break;
            }

            if bias == 2 {
                if !has_overlap(
                    bsp2_bi.low(),
                    bsp2_bi.high(),
                    bsp2s_bi.low(),
                    bsp2s_bi.high(),
                    false,
                ) {
                    break;
                }
                _low = Some(bsp2_bi.low().max(bsp2s_bi.low()));
                _high = Some(bsp2_bi.high().min(bsp2s_bi.high()));
            } else if !has_overlap(
                _low.unwrap(),
                _high.unwrap(),
                bsp2s_bi.low(),
                bsp2s_bi.high(),
                false,
            ) {
                break;
            }

            if bsp2s_break_bsp1(bsp2s_bi, break_bi) {
                break;
            }

            let retrace_rate =
                (bsp2s_bi.get_end_val() - break_bi.get_end_val()).abs() / break_bi.amp();
            if retrace_rate > bsp_conf.max_bs2_rate {
                break;
            }

            let feature_dict = Some(hashmap! {
                "bsp2_retrace_rate".to_string() => Some(retrace_rate),
                "bsp2_break_bi_amp".to_string() => Some(break_bi.amp()),
                "bsp2_bi_amp".to_string() => Some(bsp2_bi.amp()),
                "bsp2s_lv".to_string() => Some(bias as f64 /2.0),
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
    pub fn cal_seg_bs3point(&mut self, seg_list: &CSegListChan<T>, bi_list: &[T]) {
        let bsp1_bi_idx_dict = self.bsp1_idx_dict();

        for seg in seg_list.iter() {
            if !self.seg_need_cal(seg) {
                continue;
            }

            let is_buy = seg.is_down();
            let config = self.config.get_bs_config(is_buy);
            if !config.target_types.contains(&BspType::T3A)
                && !config.target_types.contains(&BspType::T3B)
            {
                continue;
            }

            let (next_seg, next_seg_idx, bsp1_bi, real_bsp1, bsp1_bi_idx, is_buy) =
                if seg_list.len() > 1 {
                    let bsp1_bi = seg.end_bi;
                    let bsp1_bi_idx = bsp1_bi.index() as isize;
                    let real_bsp1 = bsp1_bi_idx_dict.get(&bsp1_bi_idx).cloned();
                    let next_seg_idx = seg.index() + 1;
                    let next_seg = seg.to_handle().next(); // 可能为None, 所以并不一定可以保证next_seg_idx == next_seg.idx
                    (
                        next_seg,
                        next_seg_idx,
                        Some(bsp1_bi),
                        real_bsp1,
                        bsp1_bi_idx,
                        seg.is_down(),
                    )
                } else {
                    let next_seg = Some(seg.to_handle());
                    let next_seg_idx = seg.index();
                    let bsp1_bi = None;
                    let real_bsp1 = None;
                    let bsp1_bi_idx = -1;
                    (
                        next_seg,
                        next_seg_idx,
                        bsp1_bi,
                        real_bsp1,
                        bsp1_bi_idx,
                        seg.is_up(),
                    )
                };

            let bsp_conf = self.config.get_bs_config(is_buy);
            if bsp_conf.bsp3_follow_1 && !bsp1_bi_idx_dict.contains_key(&bsp1_bi_idx) {
                continue;
            }

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

    #[allow(clippy::too_many_arguments)]
    pub fn treat_bsp3_after(
        &mut self,
        seg_list: &CSegListChan<T>,
        next_seg: Handle<CSeg<T>>,
        is_buy: bool,
        bi_list: &[T],
        real_bsp1: Option<Handle<CBspPoint<T>>>,
        bsp1_bi_idx: isize,
        next_seg_idx: usize,
    ) {
        let first_zs = next_seg.get_first_multi_bi_zs();
        if first_zs.is_none() {
            return;
        }
        let bsp_conf = self.config.get_bs_config(is_buy);
        //let bsp1_bi_idx_plus_1 = match bsp1_bi_idx {
        //    None => 0,
        //    Some(idx) => idx + 1,
        //};
        if bsp_conf.strict_bsp3
            && first_zs.unwrap().get_bi_in().to_handle().index() != (bsp1_bi_idx + 1) as usize
        {
            return;
        }
        if first_zs.unwrap().bi_out.is_none()
            || first_zs.unwrap().bi_out.unwrap().index() + 1 >= bi_list.len()
        {
            return;
        }

        let bsp3_bi = &bi_list[first_zs.unwrap().bi_out.unwrap().index() + 1];

        //if bsp3_bi.seg_idx() != bsp3_bi.parent_seg_idx() {
        //   println!(
        //       "seg idx:{:?} parent:{:?}",
        //       bsp3_bi.seg_idx(),
        //       bsp3_bi.parent_seg_idx()
        //   )
        //}

        if bsp3_bi.parent_seg_idx().is_none() {
            if next_seg.index() != seg_list.len() - 1 {
                return;
            }
        } else if bsp3_bi.parent_seg_idx().unwrap() != next_seg.index()
            && bsp3_bi.parent_seg_idx().unwrap() < seg_list.len()
        {
            let parent_seg = &seg_list[bsp3_bi.parent_seg_idx().unwrap()];
            if parent_seg.bi_list.len() >= 3 {
                return;
            }
        }
        if bsp3_bi.direction() == next_seg.dir {
            return;
        }
        if bsp3_bi.seg_idx() != Some(next_seg_idx) && next_seg_idx < seg_list.len() - 2 {
            return;
        }
        if bsp3_back2zs(bsp3_bi, first_zs.unwrap()) {
            return;
        }
        let bsp3_peak_zs = bsp3_break_zspeak(bsp3_bi, first_zs.unwrap());
        if bsp_conf.bsp3_peak && !bsp3_peak_zs {
            return;
        }

        let feature_dict = Some(hashmap! {
            "bsp3_zs_height".to_string() => Some((first_zs.unwrap().high - first_zs.unwrap().low)/first_zs.unwrap().low),
            "bsp3_bi_amp".to_string() => Some(bsp3_bi.amp()),
        });

        self.add_bs(
            BspType::T3A,
            bsp3_bi.to_handle(),
            real_bsp1,
            true,
            feature_dict,
        );
    }

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
    (bsp2s_bi.is_down() && bsp2s_bi.low() < bsp2_break_bi.low())
        || (bsp2s_bi.is_up() && bsp2s_bi.high() > bsp2_break_bi.high())
}

/// 判断三类买卖点是否回中枢
fn bsp3_back2zs<T: LineType>(bsp3_bi: &T, zs: Handle<CZs<T>>) -> bool {
    (bsp3_bi.is_down() && bsp3_bi.low() < zs.high) || (bsp3_bi.is_up() && bsp3_bi.high() > zs.low)
}

/// 判断三类买卖点是否突破中枢顶底
fn bsp3_break_zspeak<T: LineType>(bsp3_bi: &T, zs: Handle<CZs<T>>) -> bool {
    (bsp3_bi.is_down() && bsp3_bi.high() >= zs.peak_high)
        || (bsp3_bi.is_up() && bsp3_bi.low() <= zs.peak_low)
}

/// 计算三类买卖点的笔结束索引
fn cal_bsp3_bi_end_idx<T: LineType>(seg: Option<Handle<CSeg<T>>>) -> usize {
    if seg.is_none() {
        return usize::MAX;
    }
    if seg.unwrap().get_multi_bi_zs_cnt() == 0 && seg.unwrap().next().is_none() {
        return usize::MAX;
    }
    let mut end_bi_idx = seg.unwrap().end_bi.index() - 1;
    for zs in seg.unwrap().zs_lst.iter() {
        if zs.is_one_bi_zs() {
            continue;
        }
        if zs.bi_out.is_some() {
            end_bi_idx = zs.bi_out.unwrap().index();
            break;
        }
    }
    end_bi_idx
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
