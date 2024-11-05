use crate::Common::CEnum::{BspType, MacdAlgo};

#[derive(Debug, Clone)]
pub struct CBSPointConfig {
    pub b_conf: CPointConfig,
    pub s_conf: CPointConfig,
}

impl CBSPointConfig {
    //pub fn new(args: HashMap<String, String>) -> Self {
    //    CBSPointConfig {
    //        b_conf: CPointConfig::new(&args),
    //        s_conf: CPointConfig::new(&args),
    //    }
    //}

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
    pub divergence_rate: f64,        // 1类买卖点背驰比例
    pub min_zs_cnt: usize,           // 1类买卖点至少要经历几个中枢，默认为 1
    pub bsp1_only_multibi_zs: bool, // min_zs_cnt 计算的中枢至少 3 笔（少于 3 笔是因为开启了 one_bi_zs 参数），默认为 True
    pub max_bs2_rate: f64,          // 2类买卖点那一笔回撤最大比例，默认为 0.618
    pub macd_algo: MacdAlgo,        // MACD指标算法
    pub bs1_peak: bool,             // 1类买卖点位置是否必须是整个中枢最低点，默认为 True
    pub target_types: Vec<BspType>, // 关注的买卖点类型
    pub bsp2_follow_1: bool, // 2类买卖点是否必须跟在1类买卖点后面（用于小转大时1类买卖点因为背驰度不足没生成），默认为 True
    pub bsp3_follow_1: bool, // 3类买卖点是否必须跟在1类买卖点后面（用于小转大时1类买卖点因为背驰度不足没生成），默认为 True
    pub bsp3_peak: bool,     // 3类买卖点突破笔是不是必须突破中枢里面最高/最低的，默认为 False
    pub bsp2s_follow_2: bool, // 类2买卖点是否必须跟在2类买卖点后面（2类买卖点可能由于不满足 max_bs2_rate 最大回测比例条件没生成），默认为 False
    pub max_bsp2s_lv: Option<usize>, // 类2买卖点最大层级（距离2类买卖点的笔的距离/2），默认为None，不做限制
    pub strict_bsp3: bool,           // 3类买卖点对应的中枢必须紧挨着1类买卖点，默认为 False
}

/*impl CPointConfig {
    pub fn new(args: &HashMap<String, String>) -> Self {
        let mut config = CPointConfig {
            divergence_rate: args["divergence_rate"].parse().unwrap(),
            min_zs_cnt: args["min_zs_cnt"].parse().unwrap(),
            bsp1_only_multibi_zs: args["bsp1_only_multibi_zs"].parse().unwrap(),
            max_bs2_rate: args["max_bs2_rate"].parse().unwrap(),
            macd_algo: MacdAlgo::Area, // Temporary value, will be set later
            bs1_peak: args["bs1_peak"].parse().unwrap(),
            //tmp_target_types: args["bs_type"]
            //    .split(',')
            //    .map(|s| s.trim().to_string())
            //    .collect(),
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
}*/

impl Default for CPointConfig {
    fn default() -> Self {
        Self {
            divergence_rate: f64::INFINITY,
            min_zs_cnt: 0,
            bsp1_only_multibi_zs: true,
            max_bs2_rate: 0.9999,
            macd_algo: MacdAlgo::Slope,
            bs1_peak: false,
            target_types: vec![
                BspType::T1,
                BspType::T2,
                BspType::T3A,
                BspType::T1P,
                BspType::T2S,
                BspType::T3B,
            ],
            bsp2_follow_1: false,
            bsp3_follow_1: false,
            bsp3_peak: false,
            bsp2s_follow_2: false,
            max_bsp2s_lv: None, // 代表无限制
            strict_bsp3: false,
        }
    }
}
