use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{BiAlgo, CBSPointConfig, CBiConfig, CPointConfig, CSegConfig, CZsConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CChanConfig {
    pub bi_conf: CBiConfig,
    pub seg_conf: CSegConfig,
    pub zs_conf: CZsConfig,
    pub bs_point_conf: CBSPointConfig,
    pub seg_bs_point_conf: CBSPointConfig,
    pub trigger_step: bool,
    pub skip_step: i32,
    pub kl_data_check: bool,
    pub max_kl_misalgin_cnt: i32,
    pub max_kl_inconsistent_cnt: i32,
    pub auto_skip_illegal_sub_lv: bool,
    pub print_warning: bool,
    pub print_err_time: bool,
    pub mean_metrics: Vec<i32>,
    pub trend_metrics: Vec<i32>,
    pub macd_config: HashMap<String, i32>,
    pub cal_demark: bool,
    pub cal_rsi: bool,
    pub cal_kdj: bool,
    pub rsi_cycle: i32,
    pub kdj_cycle: i32,
    pub demark_config: HashMap<String, bool>,
    pub boll_n: i32,
}

impl CChanConfig {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl Default for CChanConfig {
    fn default() -> Self {
        Self {
            bi_conf: CBiConfig {
                bi_algo: BiAlgo::Normal,
                is_strict: true,
                bi_fx_check: crate::FxCheckMethod::Strict,
                gap_as_kl: false,
                bi_end_is_peak: true,
                bi_allow_sub_peak: true,
            },
            seg_conf: CSegConfig {
                left_method: crate::LeftSegMethod::Peak,
            },
            zs_conf: CZsConfig {
                need_combine: true,
                zs_combine_mode: crate::CPivotCombineMode::Zs,
                one_bi_zs: false,
                zs_algo: crate::CPivotAlgo::Normal,
            },
            bs_point_conf: CBSPointConfig {
                b_conf: CPointConfig {
                    divergence_rate: f64::INFINITY,
                    min_zs_cnt: 1,
                    bsp1_only_multibi_zs: true,
                    max_bs2_rate: 0.9999,
                    macd_algo: crate::MacdAlgo::Peak,
                    bs1_peak: false,
                    target_types: vec![
                        crate::BspType::T1,
                        crate::BspType::T2,
                        crate::BspType::T3A,
                        crate::BspType::T1P,
                        crate::BspType::T2S,
                        crate::BspType::T3B,
                    ],
                    bsp2_follow_1: false,
                    bsp3_follow_1: false,
                    bsp3_peak: false,
                    bsp2s_follow_2: false,
                    max_bsp2s_lv: None, // 代表无限制
                    strict_bsp3: false,
                },
                s_conf: CPointConfig {
                    divergence_rate: f64::INFINITY,
                    min_zs_cnt: 1,
                    bsp1_only_multibi_zs: true,
                    max_bs2_rate: 0.9999,
                    macd_algo: crate::MacdAlgo::Peak,
                    bs1_peak: false,
                    target_types: vec![
                        crate::BspType::T1,
                        crate::BspType::T2,
                        crate::BspType::T3A,
                        crate::BspType::T1P,
                        crate::BspType::T2S,
                        crate::BspType::T3B,
                    ],
                    bsp2_follow_1: false,
                    bsp3_follow_1: false,
                    bsp3_peak: false,
                    bsp2s_follow_2: false,
                    max_bsp2s_lv: None, // 代表无限制
                    strict_bsp3: false,
                },
            },
            seg_bs_point_conf: CBSPointConfig {
                b_conf: CPointConfig {
                    divergence_rate: f64::INFINITY,
                    min_zs_cnt: 1,
                    bsp1_only_multibi_zs: false,
                    max_bs2_rate: 0.9999,
                    macd_algo: crate::MacdAlgo::Slope,
                    bs1_peak: false,
                    target_types: vec![
                        crate::BspType::T1,
                        crate::BspType::T2,
                        crate::BspType::T3A,
                        crate::BspType::T1P,
                        crate::BspType::T2S,
                        crate::BspType::T3B,
                    ],
                    bsp2_follow_1: false,
                    bsp3_follow_1: false,
                    bsp3_peak: false,
                    bsp2s_follow_2: false,
                    max_bsp2s_lv: None, // 代表无限制
                    strict_bsp3: false,
                },
                s_conf: CPointConfig {
                    divergence_rate: f64::INFINITY,
                    min_zs_cnt: 1,
                    bsp1_only_multibi_zs: false,
                    max_bs2_rate: 0.9999,
                    macd_algo: crate::MacdAlgo::Slope,
                    bs1_peak: false,
                    target_types: vec![
                        crate::BspType::T1,
                        crate::BspType::T2,
                        crate::BspType::T3A,
                        crate::BspType::T1P,
                        crate::BspType::T2S,
                        crate::BspType::T3B,
                    ],
                    bsp2_follow_1: false,
                    bsp3_follow_1: false,
                    bsp3_peak: false,
                    bsp2s_follow_2: false,
                    max_bsp2s_lv: None, // 代表无限制
                    strict_bsp3: false,
                },
            },

            trigger_step: true,
            skip_step: 0,

            kl_data_check: true,
            max_kl_misalgin_cnt: 2,
            max_kl_inconsistent_cnt: 5,
            auto_skip_illegal_sub_lv: false,

            print_warning: true,
            print_err_time: false,

            mean_metrics: vec![],
            trend_metrics: vec![],

            macd_config: Default::default(),
            cal_demark: Default::default(),
            cal_rsi: Default::default(),
            cal_kdj: Default::default(),
            rsi_cycle: Default::default(),
            kdj_cycle: Default::default(),
            demark_config: Default::default(),
            boll_n: Default::default(),
        }
    }
}
