use strum_macros::{Display, EnumString};

#[derive(Debug, EnumString, Display, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DATA_SRC {
    BAO_STOCK,
    CCXT,
    CSV,
}

#[derive(Debug, EnumString, Display, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KlType {
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

#[derive(Debug, EnumString, Display, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KlineDir {
    Up,
    Down,
    Combine,
    Included,
}

#[derive(Debug, EnumString, Display, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FxType {
    Bottom,
    Top,
    Unknown,
}

#[derive(Debug, EnumString, Display, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BiDir {
    Up,
    Down,
}

#[derive(Debug, EnumString, Display, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BiType {
    Unknown,
    Strict,
    SubValue,
    TiaokongThred,
    Daheng,
    TuiBi,
    Unstrict,
    TiaokongValue,
}

pub type BSP_MAIN_TYPE = String;

#[derive(Debug, EnumString, Display, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BspType {
    T1,
    T1P,
    T2,
    T2S,
    T3A,
    T3B,
}

impl BspType {
    pub fn main_type(&self) -> BSP_MAIN_TYPE {
        match self {
            BspType::T1 | BspType::T1P => "1".to_string(),
            BspType::T2 | BspType::T2S => "2".to_string(),
            BspType::T3A | BspType::T3B => "3".to_string(),
        }
    }
}

#[derive(Debug, EnumString, Display, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuType {
    Qfq,
    Hfq,
    None,
}

#[derive(Debug, EnumString, Display, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrendType {
    Mean,
    Max,
    Min,
}

#[derive(Debug, EnumString, Display, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrendLineSide {
    Inside,
    Outside,
}

#[derive(Debug, EnumString, Display, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LeftSegMethod {
    All,
    Peak,
}

#[derive(Debug, EnumString, Display, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FxCheckMethod {
    Strict,
    Loss,
    Half,
    Totally,
}

#[derive(Debug, EnumString, Display, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SegType {
    BI,
    SEG,
}

#[derive(Debug, EnumString, Display, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MacdAlgo {
    Area,
    Peak,
    FullArea,
    Diff,
    Slope,
    Amp,
    Volumn,
    Amount,
    VolumnAvg,
    AmountAvg,
    TurnrateAvg,
    Rsi,
}

pub struct DataField;

impl DataField {
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
    DataField::FIELD_VOLUME,
    DataField::FIELD_TURNOVER,
    DataField::FIELD_TURNRATE,
];
