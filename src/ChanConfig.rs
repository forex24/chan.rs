use std::collections::HashMap;

use crate::{
    Bi::BiConfig::CBiConfig, BuySellPoint::BSPointConfig::CBSPointConfig, ZS::ZSConfig::CZSConfig,
};

pub struct CChanConfig {
    pub bi_conf: CBiConfig,
    pub seg_conf: CSegConfig,
    pub zs_conf: CZSConfig,
    pub trigger_step: bool,
    pub skip_step: usize,
    pub kl_data_check: bool,
    pub max_kl_misalgin_cnt: usize,
    pub max_kl_inconsistent_cnt: usize,
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
    pub bs_point_conf: CBSPointConfig,
    pub seg_bs_point_conf: CBSPointConfig,
}

impl CChanConfig {
    pub fn new(conf: Option<HashMap<String, serde_json::Value>>) -> Result<Self, CChanException> {
        let mut conf = ConfigWithCheck::new(conf.unwrap_or_default());

        let bi_conf = CBiConfig::new(
            conf.get("bi_algo").unwrap_or("normal".into()),
            conf.get("bi_strict").unwrap_or(true),
            conf.get("bi_fx_check").unwrap_or("strict".into()),
            conf.get("gap_as_kl").unwrap_or(false),
            conf.get("bi_end_is_peak").unwrap_or(true),
            conf.get("bi_allow_sub_peak").unwrap_or(true),
        );

        let seg_conf = CSegConfig::new(
            conf.get("seg_algo").unwrap_or("chan".into()),
            conf.get("left_seg_method").unwrap_or("peak".into()),
        );

        let zs_conf = CZSConfig::new(
            conf.get("zs_combine").unwrap_or(true),
            conf.get("zs_combine_mode").unwrap_or("zs".into()),
            conf.get("one_bi_zs").unwrap_or(false),
            conf.get("zs_algo").unwrap_or("normal".into()),
        );

        let mut config = CChanConfig {
            bi_conf,
            seg_conf,
            zs_conf,
            trigger_step: conf.get("trigger_step").unwrap_or(false),
            skip_step: conf.get("skip_step").unwrap_or(0),
            kl_data_check: conf.get("kl_data_check").unwrap_or(true),
            max_kl_misalgin_cnt: conf.get("max_kl_misalgin_cnt").unwrap_or(2),
            max_kl_inconsistent_cnt: conf.get("max_kl_inconsistent_cnt").unwrap_or(5),
            auto_skip_illegal_sub_lv: conf.get("auto_skip_illegal_sub_lv").unwrap_or(false),
            print_warning: conf.get("print_warning").unwrap_or(true),
            print_err_time: conf.get("print_err_time").unwrap_or(false),
            mean_metrics: conf.get("mean_metrics").unwrap_or_else(Vec::new),
            trend_metrics: conf.get("trend_metrics").unwrap_or_else(Vec::new),
            macd_config: conf.get("macd").unwrap_or_else(|| {
                let mut map = HashMap::new();
                map.insert("fast".to_string(), 12);
                map.insert("slow".to_string(), 26);
                map.insert("signal".to_string(), 9);
                map
            }),
            cal_demark: conf.get("cal_demark").unwrap_or(false),
            cal_rsi: conf.get("cal_rsi").unwrap_or(false),
            cal_kdj: conf.get("cal_kdj").unwrap_or(false),
            rsi_cycle: conf.get("rsi_cycle").unwrap_or(14),
            kdj_cycle: conf.get("kdj_cycle").unwrap_or(9),
            demark_config: conf.get("demark").unwrap_or_else(|| {
                let mut map = HashMap::new();
                map.insert("demark_len".to_string(), 9);
                map.insert("setup_bias".to_string(), 4);
                map.insert("countdown_bias".to_string(), 2);
                map.insert("max_countdown".to_string(), 13);
                map.insert("tiaokong_st".to_string(), true);
                map.insert("setup_cmp2close".to_string(), true);
                map.insert("countdown_cmp2close".to_string(), true);
                map
            }),
            boll_n: conf.get("boll_n").unwrap_or(20),
            bs_point_conf: CBSPointConfig::default(),
            seg_bs_point_conf: CBSPointConfig::default(),
        };

        config.set_bsp_config(&mut conf)?;

        conf.check()?;

        Ok(config)
    }

    pub fn get_metric_model(&self) -> Vec<Box<dyn MetricModel>> {
        let mut res: Vec<Box<dyn MetricModel>> = Vec::new();

        res.push(Box::new(CMACD::new(
            *self.macd_config.get("fast").unwrap(),
            *self.macd_config.get("slow").unwrap(),
            *self.macd_config.get("signal").unwrap(),
        )));

        for &mean_t in &self.mean_metrics {
            res.push(Box::new(CTrendModel::new(TrendType::Mean, mean_t)));
        }

        for &trend_t in &self.trend_metrics {
            res.push(Box::new(CTrendModel::new(TrendType::Max, trend_t)));
            res.push(Box::new(CTrendModel::new(TrendType::Min, trend_t)));
        }

        res.push(Box::new(BollModel::new(self.boll_n)));

        if self.cal_demark {
            res.push(Box::new(CDemarkEngine::new(
                *self.demark_config.get("demark_len").unwrap() as i32,
                *self.demark_config.get("setup_bias").unwrap() as i32,
                *self.demark_config.get("countdown_bias").unwrap() as i32,
                *self.demark_config.get("max_countdown").unwrap() as i32,
                *self.demark_config.get("tiaokong_st").unwrap(),
                *self.demark_config.get("setup_cmp2close").unwrap(),
                *self.demark_config.get("countdown_cmp2close").unwrap(),
            )));
        }

        if self.cal_rsi {
            res.push(Box::new(RSI::new(self.rsi_cycle)));
        }

        if self.cal_kdj {
            res.push(Box::new(KDJ::new(self.kdj_cycle)));
        }

        res
    }

    fn set_bsp_config(&mut self, conf: &mut ConfigWithCheck) -> Result<(), CChanException> {
        let para_dict = [
            ("divergence_rate", serde_json::Value::from(f64::INFINITY)),
            ("min_zs_cnt", serde_json::Value::from(1)),
            ("bsp1_only_multibi_zs", serde_json::Value::from(true)),
            ("max_bs2_rate", serde_json::Value::from(0.9999)),
            ("macd_algo", serde_json::Value::from("peak")),
            ("bs1_peak", serde_json::Value::from(true)),
            ("bs_type", serde_json::Value::from("1,1p,2,2s,3a,3b")),
            ("bsp2_follow_1", serde_json::Value::from(true)),
            ("bsp3_follow_1", serde_json::Value::from(true)),
            ("bsp3_peak", serde_json::Value::from(false)),
            ("bsp2s_follow_2", serde_json::Value::from(false)),
            ("max_bsp2s_lv", serde_json::Value::Null),
            ("strict_bsp3", serde_json::Value::from(false)),
        ]
        .iter()
        .cloned()
        .collect::<HashMap<_, _>>();

        let args: HashMap<String, serde_json::Value> = para_dict
            .into_iter()
            .map(|(k, v)| (k.to_string(), conf.get(k).unwrap_or(v)))
            .collect();

        self.bs_point_conf = CBSPointConfig::new(&args)?;
        self.seg_bs_point_conf = CBSPointConfig::new(&args)?;

        self.seg_bs_point_conf
            .b_conf
            .set("macd_algo", "slope".into())?;
        self.seg_bs_point_conf
            .s_conf
            .set("macd_algo", "slope".into())?;
        self.seg_bs_point_conf
            .b_conf
            .set("bsp1_only_multibi_zs", false)?;
        self.seg_bs_point_conf
            .s_conf
            .set("bsp1_only_multibi_zs", false)?;

        for (k, v) in conf.items() {
            let v = if let serde_json::Value::String(s) = v {
                serde_json::Value::String(format!("\"{}\"", s))
            } else {
                v
            };
            let v = parse_inf(v);

            if k.ends_with("-buy") {
                let prop = k.trim_end_matches("-buy");
                self.bs_point_conf.b_conf.set(prop, v)?;
            } else if k.ends_with("-sell") {
                let prop = k.trim_end_matches("-sell");
                self.bs_point_conf.s_conf.set(prop, v)?;
            } else if k.ends_with("-segbuy") {
                let prop = k.trim_end_matches("-segbuy");
                self.seg_bs_point_conf.b_conf.set(prop, v)?;
            } else if k.ends_with("-segsell") {
                let prop = k.trim_end_matches("-segsell");
                self.seg_bs_point_conf.s_conf.set(prop, v)?;
            } else if k.ends_with("-seg") {
                let prop = k.trim_end_matches("-seg");
                self.seg_bs_point_conf.b_conf.set(prop, v.clone())?;
                self.seg_bs_point_conf.s_conf.set(prop, v)?;
            } else if args.contains_key(k) {
                self.bs_point_conf.b_conf.set(k, v.clone())?;
                self.bs_point_conf.s_conf.set(k, v)?;
            } else {
                return Err(CChanException::new(
                    &format!("unknown para = {}", k),
                    ErrCode::ParaError,
                ));
            }
        }

        self.bs_point_conf.b_conf.parse_target_type()?;
        self.bs_point_conf.s_conf.parse_target_type()?;
        self.seg_bs_point_conf.b_conf.parse_target_type()?;
        self.seg_bs_point_conf.s_conf.parse_target_type()?;

        Ok(())
    }
}

struct ConfigWithCheck {
    conf: HashMap<String, serde_json::Value>,
}

impl ConfigWithCheck {
    fn new(conf: HashMap<String, serde_json::Value>) -> Self {
        ConfigWithCheck { conf }
    }

    fn get(&mut self, k: &str) -> Option<serde_json::Value> {
        self.conf.remove(k)
    }

    fn items(&mut self) -> Vec<(String, serde_json::Value)> {
        let keys: Vec<String> = self.conf.keys().cloned().collect();
        keys.into_iter()
            .filter_map(|k| self.conf.remove(&k).map(|v| (k, v)))
            .collect()
    }

    fn check(&self) -> Result<(), CChanException> {
        if !self.conf.is_empty() {
            let invalid_keys = self.conf.keys().cloned().collect::<Vec<_>>().join(", ");
            Err(CChanException::new(
                &format!("invalid CChanConfig: {}", invalid_keys),
                ErrCode::ParaError,
            ))
        } else {
            Ok(())
        }
    }
}

pub trait MetricModel {}

impl MetricModel for CMACD {}
impl MetricModel for CTrendModel {}
impl MetricModel for BollModel {}
impl MetricModel for CDemarkEngine {}
impl MetricModel for RSI {}
impl MetricModel for KDJ {}
