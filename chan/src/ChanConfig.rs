use std::collections::HashMap;

use crate::{
    Bi::BiConfig::CBiConfig,
    BuySellPoint::BSPointConfig::CBSPointConfig,
    Common::{
        CEnum::TrendType,
        ChanException::{CChanException, ErrCode},
    },
    KLine::KLine_Unit::MetricModel,
    Math::{
        Demark::CDemarkEngine, TrendModel::CTrendModel, BOLL::BollModel, KDJ::KDJ, MACD::CMACD,
        RSI::RSI,
    },
    Seg::SegConfig::CSegConfig,
    ZS::ZSConfig::CZSConfig,
};
#[derive(Debug, Clone)]
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
    pub mean_metrics: Vec<usize>,
    pub trend_metrics: Vec<usize>,
    pub macd_config: HashMap<String, f64>,
    pub cal_demark: bool,
    pub cal_rsi: bool,
    pub cal_kdj: bool,
    pub rsi_cycle: usize,
    pub kdj_cycle: usize,
    pub demark_config: HashMap<String, bool>,
    pub boll_n: usize,
    pub bs_point_conf: CBSPointConfig,
    pub seg_bs_point_conf: CBSPointConfig,
}

impl CChanConfig {
    pub fn get_metric_model(&self) -> Vec<Box<dyn MetricModel>> {
        let mut res: Vec<Box<dyn MetricModel>> = Vec::new();

        res.push(Box::new(CMACD::new(
            *self.macd_config.get("fast").unwrap_or(&12.0),
            *self.macd_config.get("slow").unwrap_or(&26.0),
            *self.macd_config.get("signal").unwrap_or(&9.0),
        )));

        //for &mean_t in &self.mean_metrics {
        //    res.push(Box::new(CTrendModel::new(TrendType::Mean, mean_t)));
        //}
        //
        //for &trend_t in &self.trend_metrics {
        //    res.push(Box::new(CTrendModel::new(TrendType::Max, trend_t)));
        //    res.push(Box::new(CTrendModel::new(TrendType::Min, trend_t)));
        //}

        //res.push(Box::new(BollModel::new(self.boll_n)));

        //if self.cal_demark {
        //    res.push(Box::new(CDemarkEngine::new(
        //        *self.demark_config.get("demark_len").unwrap() as i32,
        //        *self.demark_config.get("setup_bias").unwrap() as i32,
        //        *self.demark_config.get("countdown_bias").unwrap() as i32,
        //        *self.demark_config.get("max_countdown").unwrap() as i32,
        //        *self.demark_config.get("tiaokong_st").unwrap(),
        //        *self.demark_config.get("setup_cmp2close").unwrap(),
        //        *self.demark_config.get("countdown_cmp2close").unwrap(),
        //    )));
        //}

        //if self.cal_rsi {
        //    res.push(Box::new(RSI::new(self.rsi_cycle)));
        //}
        //
        //if self.cal_kdj {
        //    res.push(Box::new(KDJ::new(self.kdj_cycle)));
        //}

        res
    }
}

impl Default for CChanConfig {
    fn default() -> Self {
        CChanConfig {
            bi_conf: CBiConfig::default(),
            seg_conf: CSegConfig::default(),
            zs_conf: CZSConfig::default(),
            trigger_step: false,
            skip_step: 0,
            kl_data_check: true,
            max_kl_misalgin_cnt: 2,
            max_kl_inconsistent_cnt: 5,
            auto_skip_illegal_sub_lv: false,
            print_warning: true,
            print_err_time: false,
            mean_metrics: Vec::new(),
            trend_metrics: Vec::new(),
            macd_config: HashMap::new(),
            cal_demark: false,
            cal_rsi: false,
            cal_kdj: false,
            rsi_cycle: 14,
            kdj_cycle: 9,
            demark_config: HashMap::new(),
            boll_n: 20,
            bs_point_conf: CBSPointConfig::default(),
            seg_bs_point_conf: CBSPointConfig::default(),
        }
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
                format!("invalid CChanConfig: {}", invalid_keys).to_string(),
                ErrCode::ParaError,
            ))
        } else {
            Ok(())
        }
    }
}
