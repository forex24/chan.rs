use crate::Common::CEnum::{BspType, MacdAlgo};
use std::collections::HashMap;
use strum::IntoEnumIterator;

#[derive(Debug, Clone)]
pub struct CBSPointConfig {
    pub b_conf: CPointConfig,
    pub s_conf: CPointConfig,
}

impl CBSPointConfig {
    pub fn new(args: HashMap<String, String>) -> Self {
        CBSPointConfig {
            b_conf: CPointConfig::new(&args),
            s_conf: CPointConfig::new(&args),
        }
    }

    pub fn get_bs_config(&self, is_buy: bool) -> &CPointConfig {
        if is_buy {
            &self.b_conf
        } else {
            &self.s_conf
        }
    }
}

impl Default for CBSPointConfig {
    fn default() -> Self {
        CBSPointConfig {
            b_conf: CPointConfig::default(),
            s_conf: CPointConfig::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CPointConfig {
    pub divergence_rate: f64,
    pub min_zs_cnt: usize,
    pub bsp1_only_multibi_zs: bool,
    pub max_bs2_rate: f64,
    pub macd_algo: MacdAlgo,
    pub bs1_peak: bool,
    pub tmp_target_types: Vec<String>,
    pub target_types: Vec<BspType>,
    pub bsp2_follow_1: bool,
    pub bsp3_follow_1: bool,
    pub bsp3_peak: bool,
    pub bsp2s_follow_2: bool,
    pub max_bsp2s_lv: Option<usize>,
    pub strict_bsp3: bool,
}

impl CPointConfig {
    pub fn new(args: &HashMap<String, String>) -> Self {
        let mut config = CPointConfig {
            divergence_rate: args["divergence_rate"].parse().unwrap(),
            min_zs_cnt: args["min_zs_cnt"].parse().unwrap(),
            bsp1_only_multibi_zs: args["bsp1_only_multibi_zs"].parse().unwrap(),
            max_bs2_rate: args["max_bs2_rate"].parse().unwrap(),
            macd_algo: MacdAlgo::Area, // Temporary value, will be set later
            bs1_peak: args["bs1_peak"].parse().unwrap(),
            tmp_target_types: args["bs_type"]
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            target_types: Vec::new(),
            bsp2_follow_1: args["bsp2_follow_1"].parse().unwrap(),
            bsp3_follow_1: args["bsp3_follow_1"].parse().unwrap(),
            bsp3_peak: args["bsp3_peak"].parse().unwrap(),
            bsp2s_follow_2: args["bsp2s_follow_2"].parse().unwrap(),
            max_bsp2s_lv: args["max_bsp2s_lv"].parse().ok(),
            strict_bsp3: args["strict_bsp3"].parse().unwrap(),
        };

        assert!(config.max_bs2_rate <= 1.0);
        config.set_macd_algo(&args["macd_algo"]);
        config.parse_target_type();

        config
    }

    pub fn parse_target_type(&mut self) {
        let d: HashMap<String, BspType> = BspType::iter().map(|x| (x.to_string(), x)).collect();
        for target_t in &self.tmp_target_types {
            assert!(["1", "2", "3a", "2s", "1p", "3b"].contains(&target_t.as_str()));
        }
        self.target_types = self.tmp_target_types.iter().map(|t| d[t].clone()).collect();
    }

    pub fn set_macd_algo(&mut self, macd_algo: &str) {
        let d = HashMap::from([
            ("area", MacdAlgo::Area),
            ("peak", MacdAlgo::Peak),
            ("full_area", MacdAlgo::FullArea),
            ("diff", MacdAlgo::Diff),
            ("slope", MacdAlgo::Slope),
            ("amp", MacdAlgo::Amp),
            ("amount", MacdAlgo::Amount),
            ("volumn", MacdAlgo::Volumn),
            ("amount_avg", MacdAlgo::AmountAvg),
            ("volumn_avg", MacdAlgo::VolumnAvg),
            ("turnrate_avg", MacdAlgo::AmountAvg),
            ("rsi", MacdAlgo::Rsi),
        ]);
        self.macd_algo = d[macd_algo].clone();
    }

    pub fn set(&mut self, k: &str, v: &str) {
        if k == "macd_algo" {
            self.set_macd_algo(v);
        } else {
            //let v = parse_inf(v);
            match k {
                "divergence_rate" => self.divergence_rate = v.parse().unwrap(),
                "min_zs_cnt" => self.min_zs_cnt = v.parse().unwrap(),
                "bsp1_only_multibi_zs" => self.bsp1_only_multibi_zs = v.parse().unwrap(),
                "max_bs2_rate" => self.max_bs2_rate = v.parse().unwrap(),
                "bs1_peak" => self.bs1_peak = v.parse().unwrap(),
                "bsp2_follow_1" => self.bsp2_follow_1 = v.parse().unwrap(),
                "bsp3_follow_1" => self.bsp3_follow_1 = v.parse().unwrap(),
                "bsp3_peak" => self.bsp3_peak = v.parse().unwrap(),
                "bsp2s_follow_2" => self.bsp2s_follow_2 = v.parse().unwrap(),
                "max_bsp2s_lv" => self.max_bsp2s_lv = v.parse().ok(),
                "strict_bsp3" => self.strict_bsp3 = v.parse().unwrap(),
                _ => panic!("Unknown key: {}", k),
            }
        }
    }
}

impl Default for CPointConfig {
    fn default() -> Self {
        CPointConfig {
            divergence_rate: 0.0,         // 默认值
            min_zs_cnt: 0,                // 默认值
            bsp1_only_multibi_zs: false,  // 默认值
            max_bs2_rate: 1.0,            // 默认值，假设最大值为1.0
            macd_algo: MacdAlgo::Area,    // 默认值，假设选择 Area
            bs1_peak: false,              // 默认值
            tmp_target_types: Vec::new(), // 默认值
            target_types: Vec::new(),     // 默认值
            bsp2_follow_1: false,         // 默认值
            bsp3_follow_1: false,         // 默认值
            bsp3_peak: false,             // 默认值
            bsp2s_follow_2: false,        // 默认值
            max_bsp2s_lv: None,           // 默认值
            strict_bsp3: false,           // 默认值
        }
    }
}
