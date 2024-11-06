use crate::{BspType, MacdAlgo};

#[derive(Default)]
pub struct CBSPointConfig {
    pub b_conf: CPointConfig,
    pub s_conf: CPointConfig,
}

impl CBSPointConfig {
    pub fn get_bs_config(&self, is_buy: bool) -> &CPointConfig {
        if is_buy {
            &self.b_conf
        } else {
            &self.s_conf
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

impl CPointConfig {
    /*pub fn new(args: HashMap<&str, String>) -> Self {
        let divergence_rate = args.get("divergence_rate").unwrap().parse().unwrap();
        let min_zs_cnt = args.get("min_zs_cnt").unwrap().parse().unwrap();
        let bsp1_only_multibi_zs = args.get("bsp1_only_multibi_zs").unwrap().parse().unwrap();
        let max_bs2_rate = args.get("max_bs2_rate").unwrap().parse().unwrap();
        let macd_algo = args.get("macd_algo").unwrap();
        let bs1_peak = args.get("bs1_peak").unwrap().parse().unwrap();
        let bs_type = args.get("bs_type").unwrap();
        let bsp2_follow_1 = args.get("bsp2_follow_1").unwrap().parse().unwrap();
        let bsp3_follow_1 = args.get("bsp3_follow_1").unwrap().parse().unwrap();
        let bsp3_peak = args.get("bsp3_peak").unwrap().parse().unwrap();
        let bsp2s_follow_2 = args.get("bsp2s_follow_2").unwrap().parse().unwrap();
        let max_bsp2s_lv = args.get("max_bsp2s_lv").map(|v| _parse_inf(v).unwrap());
        let strict_bsp3 = args.get("strict_bsp3").unwrap().parse().unwrap();

        assert!(max_bs2_rate <= 1.0);

        let mut config = Self {
            divergence_rate,
            min_zs_cnt,
            bsp1_only_multibi_zs,
            max_bs2_rate,
            macd_algo: MacdAlgo::from_str(macd_algo).unwrap(),
            bs1_peak,
            target_types: vec![],
            bsp2_follow_1,
            bsp3_follow_1,
            bsp3_peak,
            bsp2s_follow_2,
            max_bsp2s_lv,
            strict_bsp3,
        };

        config.parse_target_type(bs_type);
        config
    }

    pub fn parse_target_type(&mut self, tmp_target_types: &str) {
        let d: HashMap<&str, BspType> = BspType::iter().map(|x| (x.as_ref(), x)).collect();
        let tmp_target_types: Vec<&str> = tmp_target_types.split(',').map(str::trim).collect();

        for target_t in &tmp_target_types {
            assert!(["1", "2", "3a", "2s", "1p", "3b"].contains(target_t));
        }

        self.target_types = tmp_target_types.iter().map(|&t| d[t]).collect();
    }

    pub fn set(&mut self, k: &str, v: String) {
        let v = _parse_inf(&v).unwrap();
        match k {
            "macd_algo" => self.macd_algo = MacdAlgo::from_str(&v).unwrap(),
            _ => {
                // Use reflection or a match statement to set the appropriate field
                // Rust does not support reflection, so you would need to manually match each field
            }
        }
    }*/
}
