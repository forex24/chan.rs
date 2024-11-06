use super::KLine_Unit::CKLineUnit;

pub trait MetricModel {
    fn update_kline_unit(&mut self, klu: &mut CKLineUnit);
}
