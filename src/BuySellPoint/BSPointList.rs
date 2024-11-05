use crate::BuySellPoint::BSPointConfig::CBSPointConfig;
use crate::Common::func_util::has_overlap;
use crate::Common::types::{Handle, StrongHandle, WeakHandle};
use crate::Common::CEnum::BspType;
use crate::Seg::linetype::Line;
use crate::Seg::Seg::CSeg;
use crate::Seg::SegListChan::CSegListChan;
use crate::ZS::ZS::CZS;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

use super::BS_Point::CBSPoint;

pub struct CBSPointList<T> {
    pub lst: Vec<StrongHandle<CBSPoint<T>>>,
    pub bsp_dict: HashMap<usize, WeakHandle<CBSPoint<T>>>,
    pub bsp1_lst: Vec<WeakHandle<CBSPoint<T>>>,
    pub config: CBSPointConfig,
    pub last_sure_pos: Option<usize>,
}

impl<T: Line> CBSPointList<T> {
    pub fn new(bs_point_config: CBSPointConfig) -> Self {
        CBSPointList {
            lst: Vec::new(),
            bsp_dict: HashMap::new(),
            bsp1_lst: Vec::new(),
            config: bs_point_config,
            last_sure_pos: None,
        }
    }

    pub fn len(&self) -> usize {
        self.lst.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lst.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<WeakHandle<CBSPoint<T>>> {
        self.lst.get(index).map(|strong| Rc::downgrade(strong))
    }

    pub fn cal(&mut self, bi_list: &[Handle<T>], seg_list: &CSegListChan<T>) {
        self.lst.retain(|bsp| match self.last_sure_pos {
            None => false,
            Some(pos) => bsp.borrow().klu.upgrade().unwrap().borrow().idx <= pos,
        });
        self.bsp_dict = self
            .lst
            .iter()
            .map(|bsp| {
                (
                    bsp.borrow().klu.upgrade().unwrap().borrow().idx,
                    Rc::downgrade(bsp),
                )
            })
            .collect();
        self.bsp1_lst = self.lst.iter().map(|bsp| Rc::downgrade(bsp)).collect();

        self.cal_seg_bs1point(seg_list, bi_list);
        self.cal_seg_bs2point(seg_list, bi_list);
        self.cal_seg_bs3point(seg_list, bi_list);

        self.update_last_pos(seg_list);
    }

    pub fn update_last_pos(&mut self, seg_list: &CSegListChan<T>) {
        self.last_sure_pos = None;
        for seg in seg_list.iter().rev() {
            if seg.borrow().is_sure {
                self.last_sure_pos = Some(
                    seg.borrow()
                        .end_bi
                        .borrow()
                        .line_get_begin_klu()
                        .borrow()
                        .idx,
                );
                return;
            }
        }
    }

    pub fn seg_need_cal(&self, seg: &Handle<CSeg<T>>) -> bool {
        match self.last_sure_pos {
            Some(last_pos) => {
                seg.borrow().end_bi.borrow().line_get_end_klu().borrow().idx > last_pos
            }
            None => true,
        }
    }

    pub fn add_bs(
        &mut self,
        bs_type: BspType,
        bi: WeakHandle<T>,
        relate_bsp1: Option<WeakHandle<CBSPoint<T>>>,
        is_target_bsp: bool,
        feature_dict: Option<HashMap<String, Option<f64>>>,
    ) {
        let is_buy = bi.upgrade().unwrap().borrow().line_is_down();
        if let Some(exist_bsp) = self.bsp_dict.get(
            &bi.upgrade()
                .unwrap()
                .borrow()
                .line_get_end_klu()
                .borrow()
                .idx,
        ) {
            assert_eq!(exist_bsp.upgrade().unwrap().borrow().is_buy, is_buy);
            exist_bsp
                .upgrade()
                .unwrap()
                .borrow_mut()
                .add_another_bsp_prop(bs_type, relate_bsp1.clone());
            if let Some(feat_dict) = feature_dict {
                exist_bsp
                    .upgrade()
                    .unwrap()
                    .borrow_mut()
                    .add_feat(feat_dict);
            }
            return;
        }
        let mut is_target_bsp = is_target_bsp;
        if !self
            .config
            .get_bs_config(is_buy)
            .target_types
            .contains(&bs_type)
        {
            is_target_bsp = false;
        }

        if is_target_bsp || bs_type == BspType::T1 || bs_type == BspType::T1P {
            let bsp = CBSPoint::new(bi.clone(), is_buy, bs_type, relate_bsp1, feature_dict);
            if is_target_bsp {
                self.lst.push(Rc::clone(&bsp));
                self.bsp_dict.insert(
                    bi.upgrade()
                        .unwrap()
                        .borrow()
                        .line_get_end_klu()
                        .borrow()
                        .idx,
                    Rc::downgrade(&bsp),
                );
            }
            if bs_type == BspType::T1 || bs_type == BspType::T1P {
                self.bsp1_lst.push(Rc::downgrade(&bsp));
            }
        }
    }

    pub fn cal_seg_bs1point(&mut self, seg_list: &CSegListChan<T>, bi_list: &[Handle<T>]) {
        for seg in seg_list.iter() {
            if !self.seg_need_cal(seg) {
                continue;
            }
            self.cal_single_bs1point(seg, bi_list);
        }
    }

    pub fn cal_single_bs1point(&mut self, seg: &Handle<CSeg<T>>, bi_list: &[Handle<T>]) {
        let is_buy = seg.borrow().is_down();
        let bsp_conf = self.config.get_bs_config(is_buy);
        let zs_cnt = if bsp_conf.bsp1_only_multibi_zs {
            seg.borrow().get_multi_bi_zs_cnt()
        } else {
            seg.borrow().zs_lst.len()
        };
        let is_target_bsp = bsp_conf.min_zs_cnt == 0 || zs_cnt >= bsp_conf.min_zs_cnt;
        if !seg.borrow().zs_lst.is_empty()
            && !seg.borrow().zs_lst.back().unwrap().borrow().is_one_bi_zs()
            && ((seg
                .borrow()
                .zs_lst
                .back()
                .unwrap()
                .borrow()
                .bi_out
                .is_some()
                && seg
                    .borrow()
                    .zs_lst
                    .back()
                    .unwrap()
                    .borrow()
                    .bi_out
                    .as_ref()
                    .unwrap()
                    .upgrade()
                    .unwrap()
                    .borrow()
                    .line_idx()
                    >= seg.borrow().end_bi.borrow().line_idx())
                || seg
                    .borrow()
                    .zs_lst
                    .back()
                    .unwrap()
                    .borrow()
                    .bi_lst
                    .last()
                    .unwrap()
                    .upgrade()
                    .unwrap()
                    .borrow()
                    .line_idx()
                    >= seg.borrow().end_bi.borrow().line_idx())
            && seg.borrow().end_bi.borrow().line_idx()
                - seg
                    .borrow()
                    .zs_lst
                    .back()
                    .unwrap()
                    .borrow()
                    .get_bi_in()
                    .borrow()
                    .line_idx()
                > 2
        {
            self.treat_bsp1(seg, is_buy, is_target_bsp);
        } else {
            self.treat_pz_bsp1(seg, is_buy, bi_list, is_target_bsp);
        }
    }

    fn treat_bsp1(&mut self, seg: &Handle<CSeg<T>>, is_buy: bool, mut is_target_bsp: bool) {
        let bsp_conf = self.config.get_bs_config(is_buy);
        let seg_ref = seg.borrow();
        let last_zs = seg_ref.zs_lst.back().unwrap();
        let last_zs_ref = last_zs.borrow();
        let (break_peak, _) = last_zs_ref.out_bi_is_peak(seg.borrow().end_bi.borrow().line_idx());
        if bsp_conf.bs1_peak && !break_peak {
            is_target_bsp = false;
        }
        let (is_diver, divergence_rate) =
            last_zs_ref.is_divergence(bsp_conf, Some(Rc::clone(&seg_ref.end_bi)));
        if !is_diver {
            is_target_bsp = false;
        }
        let feature_dict = HashMap::from([
            ("divergence_rate".to_string(), divergence_rate),
            ("zs_cnt".to_string(), Some(seg.borrow().zs_lst.len() as f64)),
        ]);
        self.add_bs(
            BspType::T1,
            Rc::downgrade(&seg.borrow().end_bi),
            None,
            is_target_bsp,
            Some(feature_dict),
        );
    }

    fn treat_pz_bsp1(
        &mut self,
        seg: &Handle<CSeg<T>>,
        is_buy: bool,
        bi_list: &[Handle<T>],
        mut is_target_bsp: bool,
    ) {
        let bsp_conf = self.config.get_bs_config(is_buy);
        let last_bi = &seg.borrow().end_bi;

        let pre_bi_index = last_bi.borrow().line_idx() as isize - 2;
        let pre_bi_index = if pre_bi_index < 0 {
            (pre_bi_index + bi_list.len() as isize) as usize
        } else {
            pre_bi_index as usize
        };

        let pre_bi = &bi_list[pre_bi_index];
        if last_bi.borrow().line_seg_idx() != pre_bi.borrow().line_seg_idx() {
            return;
        }
        if last_bi.borrow().line_dir() != seg.borrow().dir {
            return;
        }
        if last_bi.borrow().line_is_down()
            && last_bi.borrow().line_low() > pre_bi.borrow().line_low()
        {
            return;
        }
        if last_bi.borrow().line_is_up()
            && last_bi.borrow().line_high() < pre_bi.borrow().line_high()
        {
            return;
        }
        let in_metric = pre_bi
            .borrow()
            .line_cal_macd_metric(bsp_conf.macd_algo, false)
            .unwrap_or(0.0);
        let out_metric = last_bi
            .borrow()
            .line_cal_macd_metric(bsp_conf.macd_algo, true)
            .unwrap_or(0.0);
        let (is_diver, divergence_rate) = (
            out_metric <= bsp_conf.divergence_rate * in_metric,
            Some(out_metric / (in_metric + 1e-7)),
        );
        if !is_diver {
            is_target_bsp = false;
        }
        let feature_dict = HashMap::from([
            ("divergence_rate".to_string(), divergence_rate),
            ("bsp1_bi_amp".to_string(), last_bi.borrow().line_amp()),
        ]);
        self.add_bs(
            BspType::T1P,
            Rc::downgrade(last_bi),
            None,
            is_target_bsp,
            Some(feature_dict),
        );
    }

    fn cal_seg_bs2point(&mut self, seg_list: &CSegListChan<T>, bi_list: &[Handle<T>]) {
        let bsp1_bi_idx_dict: HashMap<usize, WeakHandle<CBSPoint<T>>> = self
            .bsp1_lst
            .iter()
            .map(|bsp| {
                (
                    bsp.upgrade()
                        .unwrap()
                        .borrow()
                        .bi
                        .upgrade()
                        .unwrap()
                        .borrow()
                        .line_idx(),
                    bsp.clone(),
                )
            })
            .collect();

        for seg in seg_list.iter() {
            let config = self.config.get_bs_config(seg.borrow().is_down());

            if !config.target_types.contains(&BspType::T2)
                && !config.target_types.contains(&BspType::T2S)
            {
                continue;
            }

            self.treat_bsp2(seg, &bsp1_bi_idx_dict, seg_list, bi_list);
        }
    }

    fn treat_bsp2(
        &mut self,
        seg: &Handle<CSeg<T>>,
        bsp1_bi_idx_dict: &HashMap<usize, WeakHandle<CBSPoint<T>>>,
        seg_list: &CSegListChan<T>,
        bi_list: &[Handle<T>],
    ) {
        if !self.seg_need_cal(seg) {
            return;
        }

        let (is_buy, bsp1_bi_idx, real_bsp1, break_bi, bsp2_bi) = if seg_list.len() > 1 {
            let is_buy = seg.borrow().is_down();
            let bsp1_bi = Rc::clone(&seg.borrow().end_bi);
            let bsp1_bi_idx = bsp1_bi.borrow().line_idx();
            let real_bsp1 = bsp1_bi_idx_dict.get(&bsp1_bi.borrow().line_idx()).cloned();

            if bsp1_bi.borrow().line_idx() + 2 >= bi_list.len() {
                return;
            }

            let break_bi = &bi_list[bsp1_bi.borrow().line_idx() + 1];
            let bsp2_bi = &bi_list[bsp1_bi.borrow().line_idx() + 2];

            (is_buy, bsp1_bi_idx, real_bsp1, break_bi, bsp2_bi)
        } else {
            let is_buy = seg.borrow().is_up();

            if bi_list.len() == 1 {
                return;
            }

            let bsp2_bi = &bi_list[1];
            let break_bi = &bi_list[0];

            (is_buy, -1_i32 as usize, None, break_bi, bsp2_bi)
        };

        let bsp_conf = self.config.get_bs_config(is_buy);
        if bsp_conf.bsp2_follow_1
            && !self.bsp_dict.values().any(|bsp| {
                bsp.upgrade()
                    .unwrap()
                    .borrow()
                    .bi
                    .upgrade()
                    .unwrap()
                    .borrow()
                    .line_idx()
                    == bsp1_bi_idx
            })
        {
            return;
        }

        let retrace_rate =
            bsp2_bi.borrow().line_amp().unwrap() / break_bi.borrow().line_amp().unwrap();
        let bsp2_flag = retrace_rate <= bsp_conf.max_bs2_rate;

        if bsp2_flag {
            let feature_dict = HashMap::from([
                ("bsp2_retrace_rate".to_string(), Some(retrace_rate)),
                (
                    "bsp2_break_bi_amp".to_string(),
                    break_bi.borrow().line_amp(),
                ),
                ("bsp2_bi_amp".to_string(), bsp2_bi.borrow().line_amp()),
            ]);

            self.add_bs(
                BspType::T2,
                Rc::downgrade(&bsp2_bi),
                real_bsp1.clone(),
                true,
                Some(feature_dict),
            );
        } else if bsp_conf.bsp2s_follow_2 {
            return;
        }

        if !self
            .config
            .get_bs_config(seg.borrow().is_down())
            .target_types
            .contains(&BspType::T2S)
        {
            return;
        }

        self.treat_bsp2s(seg_list, bi_list, bsp2_bi, break_bi, real_bsp1, is_buy);
    }

    fn treat_bsp2s(
        &mut self,
        seg_list: &CSegListChan<T>,
        bi_list: &[Handle<T>],
        bsp2_bi: &Handle<T>,
        break_bi: &Handle<T>,
        real_bsp1: Option<WeakHandle<CBSPoint<T>>>,
        is_buy: bool,
    ) {
        let mut bias = 2;
        let mut overlap_range: Option<(f64, f64)> = None;

        while bsp2_bi.borrow().line_idx() + bias < bi_list.len() {
            let bsp2s_bi = &bi_list[bsp2_bi.borrow().line_idx() + bias];

            debug_assert!(
                bsp2s_bi.borrow().line_seg_idx().is_some()
                    && bsp2_bi.borrow().line_seg_idx().is_some()
            );

            let bsp_conf = self.config.get_bs_config(is_buy);
            if let Some(max_level) = bsp_conf.max_bsp2s_lv {
                if (bias as f64 / 2.0) > max_level as f64 {
                    break;
                }
            }

            if bsp2s_bi.borrow().line_seg_idx().unwrap() != bsp2_bi.borrow().line_seg_idx().unwrap()
                && (bsp2s_bi.borrow().line_seg_idx().unwrap() < seg_list.len() - 1
                    || bsp2s_bi.borrow().line_seg_idx().unwrap()
                        - bsp2_bi.borrow().line_seg_idx().unwrap()
                        >= 2
                    || seg_list[bsp2_bi.borrow().line_seg_idx().unwrap()]
                        .borrow()
                        .is_sure)
            {
                break;
            }

            if bias == 2 {
                if !has_overlap(
                    bsp2_bi.borrow().line_low(),
                    bsp2_bi.borrow().line_high(),
                    bsp2s_bi.borrow().line_low(),
                    bsp2s_bi.borrow().line_high(),
                    false,
                ) {
                    break;
                }
                overlap_range = Some((
                    bsp2_bi
                        .borrow()
                        .line_low()
                        .max(bsp2s_bi.borrow().line_low()),
                    bsp2_bi
                        .borrow()
                        .line_high()
                        .min(bsp2s_bi.borrow().line_high()),
                ));
            } else if let Some((low, high)) = overlap_range {
                if !has_overlap(
                    low,
                    high,
                    bsp2s_bi.borrow().line_low(),
                    bsp2s_bi.borrow().line_high(),
                    false,
                ) {
                    break;
                }
            }

            if bsp2s_break_bsp1(bsp2s_bi, break_bi) {
                break;
            }

            let retrace_rate =
                (bsp2s_bi.borrow().line_get_end_val() - break_bi.borrow().line_get_end_val()).abs()
                    / break_bi.borrow().line_amp().unwrap();
            if retrace_rate > bsp_conf.max_bs2_rate {
                break;
            }

            let feature_dict = HashMap::from([
                ("bsp2s_retrace_rate".to_string(), Some(retrace_rate)),
                (
                    "bsp2s_break_bi_amp".to_string(),
                    break_bi.borrow().line_amp(),
                ),
                ("bsp2s_bi_amp".to_string(), bsp2s_bi.borrow().line_amp()),
                ("bsp2s_lv".to_string(), Some(bias as f64 / 2.0)),
            ]);

            self.add_bs(
                BspType::T2S,
                Rc::downgrade(&bsp2s_bi),
                real_bsp1.clone(),
                true,
                Some(feature_dict),
            );

            bias += 2;
        }
    }

    pub fn cal_seg_bs3point(&mut self, seg_list: &CSegListChan<T>, bi_list: &[Handle<T>]) {
        let bsp1_bi_idx_dict: HashMap<usize, WeakHandle<CBSPoint<T>>> = self
            .bsp1_lst
            .iter()
            .map(|bsp| {
                (
                    bsp.upgrade()
                        .unwrap()
                        .borrow()
                        .bi
                        .upgrade()
                        .unwrap()
                        .borrow()
                        .line_idx(),
                    bsp.clone(),
                )
            })
            .collect();

        for seg in seg_list.iter() {
            if !self.seg_need_cal(seg) {
                continue;
            }
            let config = self.config.get_bs_config(seg.borrow().is_down());
            if !config.target_types.contains(&BspType::T3A)
                && !config.target_types.contains(&BspType::T3B)
            {
                continue;
            }
            let (bsp1_bi, bsp1_bi_idx, real_bsp1, next_seg_idx, next_seg, is_buy) =
                if seg_list.len() > 1 {
                    let bsp1_bi = Rc::clone(&seg.borrow().end_bi);
                    let bsp1_bi_idx = bsp1_bi.borrow().line_idx();
                    let is_buy = seg.borrow().is_down();
                    let real_bsp1 = bsp1_bi_idx_dict.get(&bsp1_bi.borrow().line_idx()).cloned();
                    let next_seg_idx = seg.borrow().idx + 1;
                    let next_seg = seg.borrow().next.clone();
                    (
                        Some(bsp1_bi),
                        Some(bsp1_bi_idx),
                        real_bsp1,
                        next_seg_idx,
                        next_seg,
                        is_buy,
                    )
                } else {
                    let next_seg = Rc::clone(seg);
                    let next_seg_idx = seg.borrow().idx;
                    let is_buy = seg.borrow().is_up();
                    (None, None, None, next_seg_idx, Some(next_seg), is_buy)
                };
            let bsp_conf = self.config.get_bs_config(is_buy);
            if bsp_conf.bsp3_follow_1
                && !self.bsp_dict.values().any(|bsp| {
                    bsp.upgrade()
                        .unwrap()
                        .borrow()
                        .bi
                        .upgrade()
                        .unwrap()
                        .borrow()
                        .line_idx()
                        == bsp1_bi_idx.unwrap_or(usize::MAX)
                })
            {
                continue;
            }
            if let Some(ref next_seg) = next_seg {
                self.treat_bsp3_after(
                    seg_list,
                    next_seg,
                    is_buy,
                    bi_list,
                    real_bsp1.clone().map(|weak| weak.upgrade().unwrap()),
                    bsp1_bi_idx,
                    next_seg_idx,
                );
            }
            self.treat_bsp3_before(
                seg_list,
                seg,
                &next_seg,
                &bsp1_bi,
                is_buy,
                bi_list,
                real_bsp1.map(|weak| weak.upgrade().unwrap()),
                next_seg_idx,
            );
        }
    }

    fn treat_bsp3_after(
        &mut self,
        seg_list: &CSegListChan<T>,
        next_seg: &Handle<CSeg<T>>,
        is_buy: bool,
        bi_list: &[Handle<T>],
        real_bsp1: Option<Handle<CBSPoint<T>>>,
        bsp1_bi_idx: Option<usize>,
        next_seg_idx: usize,
    ) {
        let first_zs = next_seg.borrow().get_first_multi_bi_zs();
        if first_zs.is_none() {
            return;
        }
        let first_zs = first_zs.unwrap();
        let bsp_conf = self.config.get_bs_config(is_buy);
        if bsp_conf.strict_bsp3
            && first_zs.borrow().get_bi_in().borrow().line_idx()
                != bsp1_bi_idx.unwrap_or(usize::MAX) + 1
        {
            return;
        }
        if first_zs.borrow().bi_out.is_none()
            || first_zs
                .borrow()
                .bi_out
                .as_ref()
                .unwrap()
                .upgrade()
                .unwrap()
                .borrow()
                .line_idx()
                + 1
                >= bi_list.len()
        {
            return;
        }
        let bsp3_bi = bi_list
            .get(
                first_zs
                    .borrow()
                    .bi_out
                    .as_ref()
                    .unwrap()
                    .upgrade()
                    .unwrap()
                    .borrow()
                    .line_idx()
                    + 1,
            )
            .unwrap();
        if bsp3_bi.borrow().line_get_parent_seg().is_none() {
            if next_seg.borrow().idx != seg_list.len() - 1 {
                return;
            }
        } else if bsp3_bi
            .borrow()
            .line_get_parent_seg()
            .as_ref()
            .unwrap()
            .borrow()
            .idx
            != next_seg.borrow().idx
            && bsp3_bi
                .borrow()
                .line_get_parent_seg()
                .as_ref()
                .unwrap()
                .borrow()
                .bi_list
                .len()
                >= 3
        {
            return;
        }
        if bsp3_bi.borrow().line_dir() == next_seg.borrow().dir {
            return;
        }
        if bsp3_bi.borrow().line_seg_idx().unwrap_or(usize::MAX) != next_seg_idx
            && next_seg_idx < seg_list.len() - 2
        {
            return;
        }
        if bsp3_back2zs(bsp3_bi, &first_zs) {
            return;
        }
        let bsp3_peak_zs = bsp3_break_zspeak(bsp3_bi, &first_zs);
        if bsp_conf.bsp3_peak && !bsp3_peak_zs {
            return;
        }
        let feature_dict = HashMap::from([
            (
                "bsp3_zs_height".to_string(),
                Some((first_zs.borrow().high - first_zs.borrow().low) / first_zs.borrow().low),
            ),
            ("bsp3_bi_amp".to_string(), bsp3_bi.borrow().line_amp()),
        ]);
        self.add_bs(
            BspType::T3A,
            Rc::downgrade(bsp3_bi),
            real_bsp1.map(|rc| Rc::downgrade(&rc)),
            true,
            Some(feature_dict),
        );
    }

    fn treat_bsp3_before(
        &mut self,
        seg_list: &CSegListChan<T>,
        seg: &Handle<CSeg<T>>,
        next_seg: &Option<Handle<CSeg<T>>>,
        bsp1_bi: &Option<Handle<T>>,
        is_buy: bool,
        bi_list: &[Handle<T>],
        real_bsp1: Option<Handle<CBSPoint<T>>>,
        next_seg_idx: usize,
    ) {
        let cmp_zs = seg.borrow().get_final_multi_bi_zs();
        if cmp_zs.is_none() {
            return;
        }
        let cmp_zs = cmp_zs.unwrap();
        if bsp1_bi.is_none() {
            return;
        }
        let bsp1_bi = bsp1_bi.as_ref().unwrap();
        let bsp_conf = self.config.get_bs_config(is_buy);
        if bsp_conf.strict_bsp3
            && (cmp_zs.borrow().bi_out.is_none()
                || cmp_zs
                    .borrow()
                    .bi_out
                    .as_ref()
                    .unwrap()
                    .upgrade()
                    .unwrap()
                    .borrow()
                    .line_idx()
                    != bsp1_bi.borrow().line_idx())
        {
            return;
        }
        let end_bi_idx = cal_bsp3_bi_end_idx(next_seg);
        for bsp3_bi in bi_list
            .iter()
            .skip(bsp1_bi.borrow().line_idx() + 2)
            .step_by(2)
        {
            if bsp3_bi.borrow().line_idx() > end_bi_idx {
                break;
            }
            assert!(bsp3_bi.borrow().line_seg_idx().is_some());
            if bsp3_bi.borrow().line_seg_idx().unwrap() != next_seg_idx
                && bsp3_bi.borrow().line_seg_idx().unwrap() < seg_list.len() - 1
            {
                break;
            }
            if bsp3_back2zs(bsp3_bi, &cmp_zs) {
                continue;
            }
            let feature_dict = HashMap::from([
                (
                    "bsp3_zs_height".to_string(),
                    Some((cmp_zs.borrow().high - cmp_zs.borrow().low) / cmp_zs.borrow().low),
                ),
                (
                    "bsp3_bi_amp".to_string(),
                    Some(bsp3_bi.borrow().line_amp().unwrap()),
                ),
            ]);
            self.add_bs(
                BspType::T3B,
                Rc::downgrade(bsp3_bi),
                real_bsp1.clone().map(|rc| Rc::downgrade(&rc)),
                true,
                Some(feature_dict),
            );
            break;
        }
    }

    pub fn get_lastest_bsp_list(&self) -> Vec<WeakHandle<CBSPoint<T>>> {
        if self.lst.is_empty() {
            return Vec::new();
        }
        let mut result: Vec<WeakHandle<CBSPoint<T>>> = self
            .lst
            .iter()
            .map(|strong| Rc::downgrade(strong))
            .collect();

        result.sort_by(|a, b| {
            b.upgrade()
                .unwrap()
                .borrow()
                .bi
                .upgrade()
                .unwrap()
                .borrow()
                .line_idx()
                .cmp(
                    &a.upgrade()
                        .unwrap()
                        .borrow()
                        .bi
                        .upgrade()
                        .unwrap()
                        .borrow()
                        .line_idx(),
                )
        });
        result
    }
}

fn bsp2s_break_bsp1<T: Line>(bsp2s_bi: &Handle<T>, bsp2_break_bi: &Handle<T>) -> bool {
    (bsp2s_bi.borrow().line_is_down()
        && bsp2s_bi.borrow().line_low() < bsp2_break_bi.borrow().line_low())
        || (bsp2s_bi.borrow().line_is_up()
            && bsp2s_bi.borrow().line_high() > bsp2_break_bi.borrow().line_high())
}

fn bsp3_back2zs<T: Line>(bsp3_bi: &Handle<T>, zs: &Handle<CZS<T>>) -> bool {
    (bsp3_bi.borrow().line_is_down() && bsp3_bi.borrow().line_low() < zs.borrow().high)
        || (bsp3_bi.borrow().line_is_up() && bsp3_bi.borrow().line_high() > zs.borrow().low)
}

fn bsp3_break_zspeak<T: Line>(bsp3_bi: &Handle<T>, zs: &Handle<CZS<T>>) -> bool {
    (bsp3_bi.borrow().line_is_down() && bsp3_bi.borrow().line_high() >= zs.borrow().peak_high)
        || (bsp3_bi.borrow().line_is_up() && bsp3_bi.borrow().line_low() <= zs.borrow().peak_low)
}

fn cal_bsp3_bi_end_idx<T: Line>(seg: &Option<Handle<CSeg<T>>>) -> usize {
    match seg {
        None => usize::MAX,
        Some(seg) => {
            if seg.borrow().get_multi_bi_zs_cnt() == 0 && seg.borrow().next.is_none() {
                usize::MAX
            } else {
                let mut end_bi_idx = seg.borrow().end_bi.borrow().line_idx() - 1;
                for zs in &seg.borrow().zs_lst {
                    if !zs.borrow().is_one_bi_zs() {
                        if let Some(bi_out) = &zs.borrow().bi_out {
                            end_bi_idx = bi_out.upgrade().unwrap().borrow().line_idx();
                            break;
                        }
                    }
                }
                end_bi_idx
            }
        }
    }
}
