use crate::{BspType, MacdAlgo};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

#[derive(Debug, Clone, Serialize, Deserialize)]
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

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CPointConfig {
    #[serde_as(as = "DisplayFromStr")]
    pub divergence_rate: f64, // 1类买卖点背驰比例，即离开中枢的笔的 MACD 指标相对于进入中枢的笔，默认为 0.9
    pub min_zs_cnt: usize,           // 1类买卖点至少要经历几个中枢，默认为 1
    pub bsp1_only_multibi_zs: bool, // min_zs_cnt 计算的中枢至少 3 笔（少于 3 笔是因为开启了 one_bi_zs 参数），默认为 True
    pub max_bs2_rate: f64, // 2类买卖点那一笔回撤最大比例，默认为 0.618,注：如果是 1.0，那么相当于允许回测到1类买卖点的位置
    pub macd_algo: MacdAlgo, // MACD指标算法
    pub bs1_peak: bool,    // 1类买卖点位置是否必须是整个中枢最低点，默认为 True
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
            min_zs_cnt: 1,
            bsp1_only_multibi_zs: true,
            max_bs2_rate: 0.9999,
            macd_algo: MacdAlgo::Peak,
            bs1_peak: true,
            target_types: vec![
                BspType::T1,
                BspType::T1P,
                BspType::T2,
                BspType::T2S,
                BspType::T3A,
                BspType::T3B,
            ],
            bsp2_follow_1: true,
            bsp3_follow_1: true,
            bsp3_peak: false,
            bsp2s_follow_2: false,
            max_bsp2s_lv: None, // 代表无限制
            strict_bsp3: false,
        }
    }
}

impl CPointConfig {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}
