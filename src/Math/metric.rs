use super::{BOLL::BollModel, MACD::CMACD};

pub enum MetricModel {
    MACD(CMACD),
    BOLL(BollModel),
}
