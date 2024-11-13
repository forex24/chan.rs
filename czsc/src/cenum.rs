use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq, Eq, Hash)]
pub enum DataSrc {
    BaoStock,
    Ccxt,
    Csv,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq, Eq, Hash)]
pub enum KlType {
    K1M,
    KDay,
    KWeek,
    KMon,
    KYear,
    K5M,
    K15M,
    K30M,
    K60M,
    K3M,
    KQuarter,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq, Eq, Hash)]
pub enum KlineDir {
    Up,
    Down,
    Combine,
    Included,
}

impl Display for KlineDir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KlineDir::Up => f.write_str("KLINE_DIR.UP"),
            KlineDir::Down => f.write_str("KLINE_DIR.DOWN"),
            KlineDir::Combine => f.write_str("KLINE_DIR.COMBINE"),
            KlineDir::Included => f.write_str("KLINE_DIR.INCLUDED"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq, Eq, Hash)]
pub enum FxType {
    Bottom,
    Top,
    Unknown,
}

impl Display for FxType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FxType::Top => f.write_str("FX_TYPE.TOP"),
            FxType::Bottom => f.write_str("FX_TYPE.BOTTOM"),
            FxType::Unknown => f.write_str("FX_TYPE.UNKNOWN"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    Down,
}
impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Direction::Up => f.write_str("BI_DIR.UP"),
            Direction::Down => f.write_str("BI_DIR.DOWN"),
        }
    }
}

impl Direction {
    pub fn flip(&self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq, Eq, Hash)]
pub enum BiType {
    Unknown,
    Strict,
    SubValue,
    TiaokongThred,
    Daheng,
    Tuibi,
    Unstrict,
    TiaokongValue,
}

impl Display for BiType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            BiType::Unknown => f.write_str("BI_TYPE.UNKNOWN"),
            BiType::Strict => f.write_str("BI_TYPE.STRICT"),
            BiType::SubValue => f.write_str("BI_TYPE.SUBVALUE"),
            BiType::TiaokongThred => f.write_str("BI_TYPE.TIAOKONGTHRED"),
            BiType::Daheng => f.write_str("BI_TYPE.DAHENG"),
            BiType::Tuibi => f.write_str("BI_TYPE.TUIBI"),
            BiType::Unstrict => f.write_str("BI_TYPE.UNSTRICT"),
            BiType::TiaokongValue => f.write_str("BI_TYPE.TIAOKONGVALUE"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq, Eq, Hash)]
pub enum BspType {
    T1,  // 一类买卖点
    T1P, //盘整背驰1类买卖点
    T2,  // 二类买卖点
    T2S, // 类二买卖点
    T3A, //中枢在1类后面
    T3B, //中枢在1类前面
}

impl Display for BspType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            BspType::T1 => f.write_str("1"),
            BspType::T1P => f.write_str("1p"),
            BspType::T2 => f.write_str("2"),
            BspType::T2S => f.write_str("2s"),
            BspType::T3A => f.write_str("3a"),
            BspType::T3B => f.write_str("3b"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq, Eq, Hash)]
pub enum Autype {
    Qfq,
    Hfq,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq, Eq, Hash)]
pub enum TrendType {
    Mean,
    Max,
    Min,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq, Eq, Hash)]
pub enum TrendLineSide {
    Inside,
    Outside,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq, Eq, Hash)]
pub enum LeftSegMethod {
    All,  // 收集至最后一个方向正确的笔，成为一段
    Peak, // 如果有个靠谱的新的极值，那么分成两段（默认）
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq, Eq, Hash)]
pub enum BiAlgo {
    Normal,
    Fx,
}

impl Display for BiAlgo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            BiAlgo::Normal => f.write_str("normal"),
            BiAlgo::Fx => f.write_str("fx"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq, Eq, Hash)]
pub enum FxCheckMethod {
    Strict,  // 底分型的最低点必须比顶分型3元素最低点的最小值还低，顶分型反之
    Loss,    // 底分型的最低点比顶分型中间元素低点还低，顶分型反之
    Half, // 对于上升笔，底分型的最低点比顶分型前两元素最低点还低，顶分型的最高点比底分型后两元素高点还高。下降笔反之
    Totally, // 底分型3元素的最高点必须必顶分型三元素的最低点还低
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq, Eq, Hash)]
pub enum SegType {
    Bi,
    Seg,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq, Eq, Hash)]
pub enum MacdAlgo {
    Area,
    Peak,
    FullArea,
    Diff,
    Slope,
    Amp,
    /*Volumn,
    Amount,
    VolumnAvg,
    AmountAvg,
    TurnrateAvg,
    Rsi,*/
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq, Eq, Hash)]
pub enum CPivotAlgo {
    Normal,
    OverSeg,
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq, Eq, Hash)]
pub enum CPivotCombineMode {
    Zs,   // 两中枢区间有重叠才合并（默认）
    Peak, //两中枢有K线重叠就合并
}
pub struct DataField;

/*
impl DataField {
    const FIELD_TIME: &'static str = "time_key";
    const FIELD_OPEN: &'static str = "open";
    const FIELD_HIGH: &'static str = "high";
    const FIELD_LOW: &'static str = "low";
    const FIELD_CLOSE: &'static str = "close";
    const FIELD_VOLUME: &'static str = "volume";
    const FIELD_TURNOVER: &'static str = "turnover";
    const FIELD_TURNRATE: &'static str = "turnover_rate";
}

pub const TRADE_INFO_LST: [&str; 3] = [
    DataField::FIELD_VOLUME,
    DataField::FIELD_TURNOVER,
    DataField::FIELD_TURNRATE,
];
*/

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq, Eq)]
pub enum EqualMode {
    TopEqual,
    BottomEqual,
}

impl Display for EqualMode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            EqualMode::TopEqual => f.write_str("允许顶相等"),
            EqualMode::BottomEqual => f.write_str("允许底相等"),
        }
    }
}
