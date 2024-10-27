// File: chan/src/Common/CEnum.rs

use strum_macros::{Display, EnumString};

#[derive(Debug, EnumString, Display)]
pub enum DATA_SRC {
    BAO_STOCK,
    CCXT,
    CSV,
}

#[derive(Debug, EnumString, Display)]
pub enum KL_TYPE {
    K_1S = 1,
    K_3S = 2,
    K_5S = 3,
    K_10S = 4,
    K_15S = 5,
    K_20S = 6,
    K_30S = 7,
    K_1M = 8,
    K_3M = 9,
    K_5M = 10,
    K_10M = 11,
    K_15M = 12,
    K_30M = 13,
    K_60M = 14,
    K_DAY = 15,
    K_WEEK = 16,
    K_MON = 17,
    K_QUARTER = 18,
    K_YEAR = 19,
}

#[derive(Debug, EnumString, Display)]
pub enum KLINE_DIR {
    UP,
    DOWN,
    COMBINE,
    INCLUDED,
}

#[derive(Debug, EnumString, Display)]
pub enum FX_TYPE {
    BOTTOM,
    TOP,
    UNKNOWN,
}

#[derive(Debug, EnumString, Display)]
pub enum BI_DIR {
    UP,
    DOWN,
}

#[derive(Debug, EnumString, Display)]
pub enum BI_TYPE {
    UNKNOWN,
    STRICT,
    SUB_VALUE,
    TIAOKONG_THRED,
    DAHENG,
    TUIBI,
    UNSTRICT,
    TIAOKONG_VALUE,
}

pub type BSP_MAIN_TYPE = String;

#[derive(Debug, EnumString, Display)]
pub enum BSP_TYPE {
    T1,
    T1P,
    T2,
    T2S,
    T3A,
    T3B,
}

impl BSP_TYPE {
    pub fn main_type(&self) -> BSP_MAIN_TYPE {
        match self {
            BSP_TYPE::T1 | BSP_TYPE::T1P => "1".to_string(),
            BSP_TYPE::T2 | BSP_TYPE::T2S => "2".to_string(),
            BSP_TYPE::T3A | BSP_TYPE::T3B => "3".to_string(),
        }
    }
}

#[derive(Debug, EnumString, Display)]
pub enum AUTYPE {
    QFQ,
    HFQ,
    NONE,
}

#[derive(Debug, EnumString, Display)]
pub enum TREND_TYPE {
    MEAN,
    MAX,
    MIN,
}

#[derive(Debug, EnumString, Display)]
pub enum TREND_LINE_SIDE {
    INSIDE,
    OUTSIDE,
}

#[derive(Debug, EnumString, Display)]
pub enum LEFT_SEG_METHOD {
    ALL,
    PEAK,
}

#[derive(Debug, EnumString, Display)]
pub enum FX_CHECK_METHOD {
    STRICT,
    LOSS,
    HALF,
    TOTALLY,
}

#[derive(Debug, EnumString, Display)]
pub enum SEG_TYPE {
    BI,
    SEG,
}

#[derive(Debug, EnumString, Display)]
pub enum MACD_ALGO {
    AREA,
    PEAK,
    FULL_AREA,
    DIFF,
    SLOPE,
    AMP,
    VOLUMN,
    AMOUNT,
    VOLUMN_AVG,
    AMOUNT_AVG,
    TURNRATE_AVG,
    RSI,
}

pub struct DATA_FIELD;

impl DATA_FIELD {
    pub const FIELD_TIME: &'static str = "time_key";
    pub const FIELD_OPEN: &'static str = "open";
    pub const FIELD_HIGH: &'static str = "high";
    pub const FIELD_LOW: &'static str = "low";
    pub const FIELD_CLOSE: &'static str = "close";
    pub const FIELD_VOLUME: &'static str = "volume";
    pub const FIELD_TURNOVER: &'static str = "turnover";
    pub const FIELD_TURNRATE: &'static str = "turnover_rate";
}

pub const TRADE_INFO_LST: [&str; 3] = [
    DATA_FIELD::FIELD_VOLUME,
    DATA_FIELD::FIELD_TURNOVER,
    DATA_FIELD::FIELD_TURNRATE,
];
