use crate::BuySellPoint::BSPointConfig::{CBSPointConfig, CPointConfig};
use crate::Common::types::Handle;
use crate::Common::CEnum::{BspType, MacdAlgo};
use crate::Seg::linetype::Line;
use crate::Seg::Seg::CSeg;
use crate::Seg::SegListChan::CSegListChan;
use crate::ZS::ZS::CZS;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::BS_Point::CBSPoint;

pub struct CBSPointList<T> {
    lst: Vec<Handle<CBSPoint<T>>>,
    bsp_dict: HashMap<usize, Handle<CBSPoint<T>>>,
    bsp1_lst: Vec<Handle<CBSPoint<T>>>,
    config: CBSPointConfig,
    last_sure_pos: Option<usize>,
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

    pub fn get(&self, index: usize) -> Option<Handle<CBSPoint<T>>> {
        self.lst.get(index).cloned()
    }

    pub fn cal(&mut self, bi_list: &[Handle<T>], seg_list: &CSegListChan<T>) {
        self.lst.retain(|bsp| match self.last_sure_pos {
            None => false,
            Some(pos) => bsp.borrow().klu.borrow().idx <= pos,
        });
        self.bsp_dict = self
            .lst
            .iter()
            .map(|bsp| {
                (
                    bsp.borrow().bi.borrow().get_end_klu().borrow().idx,
                    Rc::clone(bsp),
                )
            })
            .collect();
        self.bsp1_lst.retain(|bsp| match self.last_sure_pos {
            None => false,
            Some(pos) => bsp.borrow().klu.borrow().idx <= pos,
        });

        self.cal_seg_bs1point(seg_list, bi_list);
        self.cal_seg_bs2point(seg_list, bi_list);
        self.cal_seg_bs3point(seg_list, bi_list);

        self.update_last_pos(seg_list);
    }

    pub fn update_last_pos(&mut self, seg_list: &CSegListChan<T>) {
        self.last_sure_pos = None;
        for seg in seg_list.iter().rev() {
            if seg.borrow().is_sure {
                self.last_sure_pos =
                    Some(seg.borrow().end_bi.borrow().get_begin_klu().borrow().idx);
                return;
            }
        }
    }

    pub fn seg_need_cal(&self, seg: &Handle<CSeg<T>>) -> bool {
        match self.last_sure_pos {
            Some(last_pos) => seg.borrow().end_bi.borrow().get_end_klu().borrow().idx > last_pos,
            None => true,
        }
    }

    pub fn add_bs(
        &mut self,
        bs_type: BspType,
        bi: Handle<T>,
        relate_bsp1: Option<Handle<CBSPoint<T>>>,
        is_target_bsp: bool,
        feature_dict: Option<HashMap<String, Option<f64>>>,
    ) {
        let is_buy = bi.borrow().is_down();
        if let Some(exist_bsp) = self.bsp_dict.get(&bi.borrow().get_end_klu().borrow().idx) {
            assert_eq!(exist_bsp.borrow().is_buy, is_buy);
            exist_bsp
                .borrow_mut()
                .add_another_bsp_prop(bs_type, relate_bsp1.clone());
            if let Some(feat_dict) = feature_dict {
                exist_bsp.borrow_mut().add_feat(feat_dict);
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
            let bsp = Rc::new(RefCell::new(CBSPoint::new(
                bi.clone(),
                is_buy,
                bs_type,
                relate_bsp1,
                feature_dict,
            )));
            if is_target_bsp {
                self.lst.push(Rc::clone(&bsp));
                self.bsp_dict
                    .insert(bi.borrow().get_end_klu().borrow().idx, Rc::clone(&bsp));
            }
            if bs_type == BspType::T1 || bs_type == BspType::T1P {
                self.bsp1_lst.push(Rc::clone(&bsp));
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
        let is_target_bsp = bsp_conf.min_zs_cnt <= 0 || zs_cnt >= bsp_conf.min_zs_cnt as usize;
        if !seg.borrow().zs_lst.is_empty()
            && !seg.borrow().zs_lst.last().unwrap().borrow().is_one_bi_zs()
            && ((seg
                .borrow()
                .zs_lst
                .last()
                .unwrap()
                .borrow()
                .bi_out
                .is_some()
                && seg
                    .borrow()
                    .zs_lst
                    .last()
                    .unwrap()
                    .borrow()
                    .bi_out
                    .as_ref()
                    .unwrap()
                    .borrow()
                    .idx()
                    >= seg.borrow().end_bi.borrow().idx())
                || seg
                    .borrow()
                    .zs_lst
                    .last()
                    .unwrap()
                    .borrow()
                    .bi_lst
                    .last()
                    .unwrap()
                    .borrow()
                    .idx()
                    >= seg.borrow().end_bi.borrow().idx())
            && seg.borrow().end_bi.borrow().idx()
                - seg
                    .borrow()
                    .zs_lst
                    .last()
                    .unwrap()
                    .borrow()
                    .get_bi_in()
                    .borrow()
                    .idx()
                > 2
        {
            self.treat_bsp1(seg, is_buy, is_target_bsp);
        } else {
            self.treat_pz_bsp1(seg, is_buy, bi_list, is_target_bsp);
        }
    }

    fn treat_bsp1(&mut self, seg: &Handle<CSeg<T>>, is_buy: bool, mut is_target_bsp: bool) {
        let bsp_conf = self.config.get_bs_config(is_buy);
        let last_zs = seg.borrow().zs_lst.last().unwrap();
        let (break_peak, _) = last_zs
            .borrow()
            .out_bi_is_peak(seg.borrow().end_bi.borrow().idx());
        if bsp_conf.bs1_peak && !break_peak {
            is_target_bsp = false;
        }
        let (is_diver, divergence_rate) = last_zs
            .borrow()
            .is_divergence(bsp_conf, Some(seg.borrow().end_bi));
        if !is_diver {
            is_target_bsp = false;
        }
        let feature_dict = HashMap::from([
            ("divergence_rate".to_string(), divergence_rate),
            ("zs_cnt".to_string(), Some(seg.borrow().zs_lst.len() as f64)),
        ]);
        self.add_bs(
            BspType::T1,
            Rc::clone(&seg.borrow().end_bi),
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
        let pre_bi = &bi_list.get(last_bi.borrow().idx() as usize - 2).unwrap();
        if last_bi.borrow().seg_idx != pre_bi.borrow().seg_idx {
            return;
        }
        if last_bi.borrow().dir() != seg.borrow().dir {
            return;
        }
        if last_bi.borrow().is_down() && last_bi.borrow().low() > pre_bi.borrow().low() {
            return;
        }
        if last_bi.borrow().is_up() && last_bi.borrow().high() < pre_bi.borrow().high() {
            return;
        }
        let in_metric = pre_bi.borrow().cal_macd_metric(bsp_conf.macd_algo, false);
        let out_metric = last_bi.borrow().cal_macd_metric(bsp_conf.macd_algo, true);
        let (is_diver, divergence_rate) = (
            out_metric <= bsp_conf.divergence_rate * in_metric,
            out_metric / (in_metric + 1e-7),
        );
        if !is_diver {
            is_target_bsp = false;
        }
        let feature_dict = HashMap::from([
            ("divergence_rate".to_string(), divergence_rate),
            ("bsp1_bi_amp".to_string(), last_bi.borrow().amp()),
        ]);
        self.add_bs(
            BspType::T1P,
            Rc::clone(last_bi),
            None,
            is_target_bsp,
            Some(feature_dict),
        );
    }

    pub fn cal_seg_bs2point(&mut self, seg_list: &CSegListChan<T>, bi_list: &[Handle<T>]) {
        let bsp1_bi_idx_dict: HashMap<i32, Handle<CBSPoint<T>>> = self
            .bsp1_lst
            .iter()
            .map(|bsp| (bsp.borrow().bi.borrow().idx(), Rc::clone(bsp)))
            .collect();

        for seg in seg_list.iter() {
            if !self.seg_need_cal(seg) {
                continue;
            }
            let bsp_conf = self.config.get_bs_config(seg.borrow().is_down());
            if !bsp_conf.target_types.contains(&BspType::T2) {
                continue;
            }
            let bsp1_bi = &seg.borrow().end_bi;
            let real_bsp1 = bsp1_bi_idx_dict.get(&bsp1_bi.borrow().idx).cloned();
            if bsp_conf.bsp2_follow_1 && real_bsp1.is_none() {
                continue;
            }
            let next_seg = seg.borrow().next.clone();
            if let Some(next_seg) = next_seg {
                self.treat_bsp2(seg_list, &next_seg, bsp_conf, bi_list, real_bsp1);
            }
        }
    }

    fn treat_bsp2(
        &mut self,
        seg_list: &CSegListChan<T>,
        next_seg: &Handle<CSeg<T>>,
        bsp_conf: &CPointConfig,
        bi_list: &[Handle<T>],
        real_bsp1: Option<Handle<CBSPoint<T>>>,
    ) {
        let first_zs = next_seg.borrow().get_first_multi_bi_zs();
        if first_zs.is_none() {
            return;
        }
        let first_zs = first_zs.unwrap();
        if first_zs.borrow().bi_out.is_none()
            || first_zs.borrow().bi_out.as_ref().unwrap().borrow().idx() + 1 >= bi_list.len()
        {
            return;
        }
        let bsp2_bi = bi_list
            .get((first_zs.borrow().bi_out.as_ref().unwrap().borrow().idx() + 1) as usize)
            .unwrap();
        if bsp2_bi.borrow().parent_seg.is_none() {
            if next_seg.borrow().idx != seg_list.len() - 1 {
                return;
            }
        } else if bsp2_bi.borrow().parent_seg.as_ref().unwrap().borrow().idx
            != next_seg.borrow().idx
        {
            if bsp2_bi
                .borrow()
                .parent_seg
                .as_ref()
                .unwrap()
                .borrow()
                .bi_list
                .len()
                >= 3
            {
                return;
            }
        }
        if bsp2_bi.borrow().dir() == next_seg.borrow().dir {
            return;
        }
        if bsp2_bi.borrow().seg_idx != next_seg.borrow().idx
            && next_seg.borrow().idx < seg_list.len() - 2
        {
            return;
        }
        let bsp2_break_bi = first_zs.borrow().bi_out.as_ref().unwrap();
        let retrace_rate = (bsp2_bi.borrow().get_end_val() - bsp2_break_bi.borrow().get_end_val())
            .abs()
            / (bsp2_break_bi.borrow().get_end_val()
                - first_zs.borrow().get_bi_in().borrow().get_end_val())
            .abs();
        if retrace_rate > bsp_conf.max_bs2_rate {
            return;
        }
        let feature_dict = HashMap::from([
            ("bsp2_retrace_rate".to_string(), retrace_rate),
            (
                "bsp2_break_bi_amp".to_string(),
                bsp2_break_bi.borrow().amp(),
            ),
            ("bsp2_bi_amp".to_string(), bsp2_bi.borrow().amp()),
        ]);
        self.add_bs(
            BspType::T2,
            Rc::clone(bsp2_bi),
            real_bsp1,
            true,
            Some(feature_dict),
        );
    }

    pub fn cal_seg_bs3point(&mut self, seg_list: &CSegListChan<T>, bi_list: &[Handle<T>]) {
        let bsp1_bi_idx_dict: HashMap<i32, Handle<CBSPoint<T>>> = self
            .bsp1_lst
            .iter()
            .map(|bsp| (bsp.borrow().bi.borrow().idx(), Rc::clone(bsp)))
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
            let (bsp1_bi, bsp1_bi_idx, real_bsp1, next_seg_idx, next_seg, bsp_conf) =
                if seg_list.len() > 1 {
                    let bsp1_bi = Rc::clone(&seg.borrow().end_bi);
                    let bsp1_bi_idx = bsp1_bi.borrow().idx();
                    let bsp_conf = self.config.get_bs_config(seg.borrow().is_down());
                    let real_bsp1 = bsp1_bi_idx_dict.get(&bsp1_bi.borrow().idx()).cloned();
                    let next_seg_idx = seg.borrow().idx + 1;
                    let next_seg = seg.borrow().next.clone();
                    (
                        Some(bsp1_bi),
                        bsp1_bi_idx,
                        real_bsp1,
                        next_seg_idx,
                        next_seg,
                        bsp_conf,
                    )
                } else {
                    let next_seg = Rc::clone(seg);
                    let next_seg_idx = seg.borrow().idx;
                    let bsp_conf = self.config.get_bs_config(seg.borrow().is_up());
                    (None, -1, None, next_seg_idx, Some(next_seg), bsp_conf)
                };
            if bsp_conf.bsp3_follow_1
                && !self
                    .bsp_dict
                    .values()
                    .any(|bsp| bsp.borrow().bi.borrow().idx() == bsp1_bi_idx)
            {
                continue;
            }
            if let Some(next_seg) = next_seg {
                self.treat_bsp3_after(
                    seg_list,
                    &next_seg,
                    &bsp_conf,
                    bi_list,
                    real_bsp1.clone(),
                    bsp1_bi_idx,
                    next_seg_idx,
                );
            }
            self.treat_bsp3_before(
                seg_list,
                seg,
                next_seg.as_ref(),
                bsp1_bi.as_ref(),
                &bsp_conf,
                bi_list,
                real_bsp1,
                next_seg_idx,
            );
        }
    }

    fn treat_bsp3_after(
        &mut self,
        seg_list: &CSegListChan<T>,
        next_seg: &Handle<CSeg<T>>,
        bsp_conf: &CPointConfig,
        bi_list: &[Handle<T>],
        real_bsp1: Option<Handle<CBSPoint<T>>>,
        bsp1_bi_idx: usize,
        next_seg_idx: usize,
    ) {
        let first_zs = next_seg.borrow().get_first_multi_bi_zs();
        if first_zs.is_none() {
            return;
        }
        let first_zs = first_zs.unwrap();
        if bsp_conf.strict_bsp3 && first_zs.borrow().get_bi_in().borrow().idx() != bsp1_bi_idx + 1 {
            return;
        }
        if first_zs.borrow().bi_out.is_none()
            || first_zs.borrow().bi_out.as_ref().unwrap().borrow().idx() + 1 >= bi_list.len()
        {
            return;
        }
        let bsp3_bi = bi_list
            .get((first_zs.borrow().bi_out.as_ref().unwrap().borrow().idx() + 1) as usize)
            .unwrap();
        if bsp3_bi.borrow().parent_seg.is_none() {
            if next_seg.borrow().idx != seg_list.len() - 1 {
                return;
            }
        } else if bsp3_bi.borrow().parent_seg.as_ref().unwrap().borrow().idx
            != next_seg.borrow().idx
        {
            if bsp3_bi
                .borrow()
                .parent_seg
                .as_ref()
                .unwrap()
                .borrow()
                .bi_list
                .len()
                >= 3
            {
                return;
            }
        }
        if bsp3_bi.borrow().dir() == next_seg.borrow().dir {
            return;
        }
        if bsp3_bi.borrow().seg_idx != next_seg_idx && next_seg_idx < seg_list.len() - 2 {
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
                (first_zs.borrow().high - first_zs.borrow().low) / first_zs.borrow().low,
            ),
            ("bsp3_bi_amp".to_string(), bsp3_bi.borrow().amp()),
        ]);
        self.add_bs(
            BspType::T3A,
            Rc::clone(bsp3_bi),
            real_bsp1,
            true,
            Some(feature_dict),
        );
    }

    fn treat_bsp3_before(
        &mut self,
        seg_list: &CSegListChan<T>,
        seg: &Handle<CSeg<T>>,
        next_seg: Option<&Handle<CSeg<T>>>,
        bsp1_bi: Option<&Handle<T>>,
        bsp_conf: &CPointConfig,
        bi_list: &[Handle<T>],
        real_bsp1: Option<Handle<CBSPoint<T>>>,
        next_seg_idx: i32,
    ) {
        let cmp_zs = seg.borrow().get_final_multi_bi_zs();
        if cmp_zs.is_none() {
            return;
        }
        let cmp_zs = cmp_zs.unwrap();
        if bsp1_bi.is_none() {
            return;
        }
        let bsp1_bi = bsp1_bi.unwrap();
        if bsp_conf.strict_bsp3
            && (cmp_zs.borrow().bi_out.is_none()
                || cmp_zs.borrow().bi_out.as_ref().unwrap().borrow().idx()
                    != bsp1_bi.borrow().idx())
        {
            return;
        }
        let end_bi_idx = cal_bsp3_bi_end_idx(next_seg);
        for bsp3_bi in bi_list
            .iter()
            .skip((bsp1_bi.borrow().idx() + 2) as usize)
            .step_by(2)
        {
            if bsp3_bi.borrow().idx() > end_bi_idx {
                break;
            }
            assert!(bsp3_bi.borrow().seg_idx.is_some());
            if bsp3_bi.borrow().seg_idx.unwrap() != next_seg_idx
                && bsp3_bi.borrow().seg_idx.unwrap() < seg_list.len() as i32 - 1
            {
                break;
            }
            if bsp3_back2zs(bsp3_bi, &cmp_zs) {
                continue;
            }
            let feature_dict = HashMap::from([
                (
                    "bsp3_zs_height".to_string(),
                    (cmp_zs.borrow().high - cmp_zs.borrow().low) / cmp_zs.borrow().low,
                ),
                ("bsp3_bi_amp".to_string(), bsp3_bi.borrow().amp()),
            ]);
            self.add_bs(
                BspType::T3B,
                Rc::clone(bsp3_bi),
                real_bsp1.clone(),
                true,
                Some(feature_dict),
            );
            break;
        }
    }

    pub fn get_lastest_bsp_list(&self) -> Vec<Handle<CBSPoint<T>>> {
        if self.lst.is_empty() {
            return Vec::new();
        }
        let mut result = self.lst.clone();
        result.sort_by(|a, b| {
            b.borrow()
                .bi
                .borrow()
                .idx()
                .cmp(&a.borrow().bi.borrow().idx())
        });
        result
    }
}

fn bsp2s_break_bsp1<T: Line>(bsp2s_bi: &Handle<T>, bsp2_break_bi: &Handle<T>) -> bool {
    (bsp2s_bi.borrow().is_down() && bsp2s_bi.borrow().low() < bsp2_break_bi.borrow().low())
        || (bsp2s_bi.borrow().is_up() && bsp2s_bi.borrow().high() > bsp2_break_bi.borrow().high())
}

fn bsp3_back2zs<T: Line>(bsp3_bi: &Handle<T>, zs: &CZS<T>) -> bool {
    (bsp3_bi.borrow().is_down() && bsp3_bi.borrow().low() < zs.high)
        || (bsp3_bi.borrow().is_up() && bsp3_bi.borrow().high() > zs.low)
}

fn bsp3_break_zspeak<T: Line>(bsp3_bi: &Handle<T>, zs: &CZS<T>) -> bool {
    (bsp3_bi.borrow().is_down() && bsp3_bi.borrow().high() >= zs.peak_high)
        || (bsp3_bi.borrow().is_up() && bsp3_bi.borrow().low() <= zs.peak_low)
}

fn cal_bsp3_bi_end_idx<T: Line>(seg: Option<&Handle<CSeg<T>>>) -> usize {
    match seg {
        None => usize::MAX,
        Some(seg) => {
            if seg.borrow().get_multi_bi_zs_cnt() == 0 && seg.borrow().next.is_none() {
                usize::MAX
            } else {
                let mut end_bi_idx = seg.borrow().end_bi.borrow().idx() - 1;
                for zs in &seg.borrow().zs_lst {
                    if !zs.borrow().is_one_bi_zs() {
                        if let Some(bi_out) = &zs.borrow().bi_out {
                            end_bi_idx = bi_out.borrow().idx();
                            break;
                        }
                    }
                }
                end_bi_idx
            }
        }
    }
}
