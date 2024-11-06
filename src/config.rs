use std::collections::HashMap;

use crate::{CBSPointConfig, CBiConfig, CSegConfig, CZsConfig};

#[derive(Default)]
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

/*
impl CChanConfig {
    fn new(conf: Option<HashMap<String, String>>) -> Self {
        let mut conf = conf.unwrap_or_else(HashMap::new);
        let conf = ConfigWithCheck::new(conf);
        let bi_conf = CBiConfig {
            bi_algo: conf.get("bi_algo", "normal"),
            is_strict: conf.get("bi_strict", true),
            bi_fx_check: conf.get("bi_fx_check", "strict"),
            gap_as_kl: conf.get("gap_as_kl", false),
            bi_end_is_peak: conf.get("bi_end_is_peak", true),
            bi_allow_sub_peak: conf.get("bi_allow_sub_peak", true),
        };
        let seg_conf = CSegConfig {
            seg_algo: conf.get("seg_algo", "chan"),
            left_method: conf.get("left_seg_method", "peak"),
        };
        let zs_conf = CZSConfig {
            need_combine: conf.get("zs_combine", true),
            zs_combine_mode: conf.get("zs_combine_mode", "zs"),
            one_bi_zs: conf.get("one_bi_zs", false),
            zs_algo: conf.get("zs_algo", "normal"),
        };
        let trigger_step = conf.get("trigger_step", false);
        let skip_step = conf.get("skip_step", 0);
        let kl_data_check = conf.get("kl_data_check", true);
        let max_kl_misalgin_cnt = conf.get("max_kl_misalgin_cnt", 2);
        let max_kl_inconsistent_cnt = conf.get("max_kl_inconsistent_cnt", 5);
        let auto_skip_illegal_sub_lv = conf.get("auto_skip_illegal_sub_lv", false);
        let print_warning = conf.get("print_warning", true);
        let print_err_time = conf.get("print_err_time", false);
        let mean_metrics: Vec<i32> = conf.get("mean_metrics", vec![]);
        let trend_metrics: Vec<i32> = conf.get("trend_metrics", vec![]);
        let macd_config: HashMap<String, i32> = conf.get("macd", {
            let mut map = HashMap::new();
            map.insert("fast".to_string(), 12);
            map.insert("slow".to_string(), 26);
            map.insert("signal".to_string(), 9);
            map
        });
        let cal_demark = conf.get("cal_demark", false);
        let cal_rsi = conf.get("cal_rsi", false);
        let cal_kdj = conf.get("cal_kdj", false);
        let rsi_cycle = conf.get("rsi_cycle", 14);
        let kdj_cycle = conf.get("kdj_cycle", 9);
        let demark_config: HashMap<String, bool> = conf.get("demark", {
            let mut map = HashMap::new();
            map.insert("demark_len".to_string(), 9);
            map.insert("setup_bias".to_string(), 4);
            map.insert("countdown_bias".to_string(), 2);
            map.insert("max_countdown".to_string(), 13);
            map.insert("tiaokong_st".to_string(), true);
            map.insert("setup_cmp2close".to_string(), true);
            map.insert("countdown_cmp2close".to_string(), true);
            map
        });
        let boll_n = conf.get("boll_n", 20);
        let bs_point_conf = CBSPointConfig::new(conf);
        let seg_bs_point_conf = CBSPointConfig::new(conf);
        seg_bs_point_conf.b_conf.macd_algo = "slope".to_string();
        seg_bs_point_conf.s_conf.macd_algo = "slope".to_string();
        seg_bs_point_conf.b_conf.bsp1_only_multibi_zs = false;
        seg_bs_point_conf.s_conf.bsp1_only_multibi_zs = false;
        for (k, v) in conf.iter() {
            if let Some(v) = v.parse::<f64>().ok() {
                if k.ends_with("-buy") {
                    let prop = k.replace("-buy", "");
                    bs_point_conf.b_conf.set(prop, v);
                } else if k.ends_with("-sell") {
                    let prop = k.replace("-sell", "");
                    bs_point_conf.s_conf.set(prop, v);
                } else if k.ends_with("-segbuy") {
                    let prop = k.replace("-segbuy", "");
                    seg_bs_point_conf.b_conf.set(prop, v);
                } else if k.ends_with("-segsell") {
                    let prop = k.replace("-segsell", "");
                    seg_bs_point_conf.s_conf.set(prop, v);
                } else if k.ends_with("-seg") {
                    let prop = k.replace("-seg", "");
                    seg_bs_point_conf.b_conf.set(prop, v);
                    seg_bs_point_conf.s_conf.set(prop, v);
                } else if let Some(default_value) = para_dict.get(k) {
                    bs_point_conf.b_conf.set(k, v);
                    bs_point_conf.s_conf.set(k, v);
                } else {
                    return Err(CChanException {
                        message: format!("unknown para = {}", k),
                        err_code: ErrCode::PARA_ERROR,
                    });
                }
            }
        }
        bs_point_conf.b_conf.parse_target_type();
        bs_point_conf.s_conf.parse_target_type();
        seg_bs_point_conf.b_conf.parse_target_type();
        seg_bs_point_conf.s_conf.parse_target_type();
        Ok(Self {
            bi_conf,
            seg_conf,
            zs_conf,
            trigger_step,
            skip_step,
            kl_data_check,
            max_kl_misalgin_cnt,
            max_kl_inconsistent_cnt,
            auto_skip_illegal_sub_lv,
            print_warning,
            print_err_time,
            mean_metrics,
            trend_metrics,
            macd_config,
            cal_demark,
            cal_rsi,
            cal_kdj,
            rsi_cycle,
            kdj_cycle,
            demark_config,
            boll_n,
            bs_point_conf,
            seg_bs_point_conf,
        })
    }

    fn get_metric_model(&self) -> Vec<CMACD | CTrendModel | BollModel | CDemarkEngine | RSI | KDJ> {
        let mut res: Vec<CMACD | CTrendModel | BollModel | CDemarkEngine | RSI | KDJ> = vec![
            CMACD {
                fastperiod: self.macd_config.get("fast").unwrap(),
                slowperiod: self.macd_config.get("slow").unwrap(),
                signalperiod: self.macd_config.get("signal").unwrap(),
            }
        ];
        res.extend(self.mean_metrics.iter().map(|&mean_T| CTrendModel {
            trend_type: TREND_TYPE::MEAN,
            trend_T: mean_T,
        }));
        for &trend_T in self.trend_metrics.iter() {
            res.push(CTrendModel {
                trend_type: TREND_TYPE::MAX,
                trend_T,
            });
            res.push(CTrendModel {
                trend_type: TREND_TYPE::MIN,
                trend_T,
            });
        }
        res.push(BollModel {
            n: self.boll_n,
        });
        if self.cal_demark {
            res.push(CDemarkEngine {
                demark_len: self.demark_config.get("demark_len").unwrap(),
                setup_bias: self.demark_config.get("setup_bias").unwrap(),
                countdown_bias: self.demark_config.get("countdown_bias").unwrap(),
                max_countdown: self.demark_config.get("max_countdown").unwrap(),
                tiaokong_st: self.demark_config.get("tiaokong_st").unwrap(),
                setup_cmp2close: self.demark_config.get("setup_cmp2close").unwrap(),
                countdown_cmp2close: self.demark_config.get("countdown_cmp2close").unwrap(),
            });
        }
        if self.cal_rsi {
            res.push(RSI {
                cycle: self.rsi_cycle,
            });
        }
        if self.cal_kdj {
            res.push(KDJ {
                cycle: self.kdj_cycle,
            });
        }
        res
    }

    fn set_bsp_config(&mut self, conf: HashMap<String, String>) {
        let para_dict: HashMap<String, String> = [
            ("divergence_rate", "inf"),
            ("min_zs_cnt", "1"),
            ("bsp1_only_multibi_zs", "true"),
            ("max_bs2_rate", "0.9999"),
            ("macd_algo", "peak"),
            ("bs1_peak", "true"),
            ("bs_type", "1,1p,2,2s,3a,3b"),
            ("bsp2_follow_1", "true"),
            ("bsp3_follow_1", "true"),
            ("bsp3_peak", "false"),
            ("bsp2s_follow_2", "false"),
            ("max_bsp2s_lv", "None"),
            ("strict_bsp3", "false"),
        ].iter().cloned().collect();
        let args = para_dict.iter().map(|(para, default_value)| {
            (para, conf.get(para).unwrap_or(default_value))
        }).collect::<HashMap<_, _>>();
        self.bs_point_conf = CBSPointConfig::new(args);
        self.seg_bs_point_conf = CBSPointConfig::new(args);
        self.seg_bs_point_conf.b_conf.macd_algo = "slope".to_string();
        self.seg_bs_point_conf.s_conf.macd_algo = "slope".to_string();
        self.seg_bs_point_conf.b_conf.bsp1_only_multibi_zs = false;
        self.seg_bs_point_conf.s_conf.bsp1_only_multibi_zs = false;
        for (k, v) in conf.iter() {
            if let Some(v) = v.parse::<f64>().ok() {
                if k.ends_with("-buy") {
                    let prop = k.replace("-buy", "");
                    self.bs_point_conf.b_conf.set(prop, v);
                } else if k.ends_with("-sell") {
                    let prop = k.replace("-sell", "");
                    self.bs_point_conf.s_conf.set(prop, v);
                } else if k.ends_with("-segbuy") {
                    let prop = k.replace("-segbuy", "");
                    self.seg_bs_point_conf.b_conf.set(prop, v);
                } else if k.ends_with("-segsell") {
                    let prop = k.replace("-segsell", "");
                    self.seg_bs_point_conf.s_conf.set(prop, v);
                } else if k.ends_with("-seg") {
                    let prop = k.replace("-seg", "");
                    self.seg_bs_point_conf.b_conf.set(prop, v);
                    self.seg_bs_point_conf.s_conf.set(prop, v);
                } else if let Some(default_value) = para_dict.get(k) {
                    self.bs_point_conf.b_conf.set(k, v);
                    self.bs_point_conf.s_conf.set(k, v);
                } else {
                    return Err(CChanException {
                        message: format!("unknown para = {}", k),
                        err_code: ErrCode::PARA_ERROR,
                    });
                }
            }
        }
        self.bs_point_conf.b_conf.parse_target_type();
        self.bs_point_conf.s_conf.parse_target_type();
        self.seg_bs_point_conf.b_conf.parse_target_type();
        self.seg_bs_point_conf.s_conf.parse_target_type();
        Ok(())
    }
}

struct ConfigWithCheck {
    conf: HashMap<String, String>,
}

impl ConfigWithCheck {
    fn new(conf: HashMap<String, String>) -> Self {
        Self {
            conf,
        }
    }

    fn get(&mut self, k: &str, default_value: impl Into<String>) -> String {
        let default_value = default_value.into();
        let res = self.conf.get(k).cloned().unwrap_or(default_value);
        self.conf.remove(k);
        res
    }

    fn iter(&mut self) -> impl Iterator<Item = (String, String)> {
        let mut visit_keys = Vec::new();
        let conf = std::mem::replace(&mut self.conf, HashMap::new());
        let iter = conf.into_iter().filter_map(move |(k, v)| {
            visit_keys.push(k.clone());
            Some((k, v))
        });
        for k in visit_keys {
            self.conf.remove(&k);
        }
        iter
    }

    fn check(&self) -> Result<(), CChanException> {
        if !self.conf.is_empty() {
            let invalid_key_lst = self.conf.keys().cloned().collect::<Vec<_>>().join(",");
            Err(CChanException {
                message: format!("invalid CChanConfig: {}", invalid_key_lst),
                err_code: ErrCode::PARA_ERROR,
            })
        } else {
            Ok(())
        }
    }
}
*/
