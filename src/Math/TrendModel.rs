use crate::Common::CEnum::TrendType;
use crate::Common::ChanException::{CChanException, ErrCode};

pub struct CTrendModel {
    t: usize,
    arr: Vec<f64>,
    trend_type: TrendType,
}

impl CTrendModel {
    pub fn new(trend_type: TrendType, t: usize) -> Self {
        CTrendModel {
            t,
            arr: Vec::with_capacity(t),
            trend_type,
        }
    }

    pub fn add(&mut self, value: f64) -> Result<f64, CChanException> {
        self.arr.push(value);
        if self.arr.len() > self.t {
            self.arr = self.arr.split_off(self.arr.len() - self.t);
        }

        match self.trend_type {
            TrendType::Mean => Ok(self.arr.iter().sum::<f64>() / self.arr.len() as f64),
            TrendType::Max => self
                .arr
                .iter()
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .map(|&x| x)
                .ok_or_else(|| CChanException::new("Empty array".to_string(), ErrCode::ParaError)),
            TrendType::Min => self
                .arr
                .iter()
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .map(|&x| x)
                .ok_or_else(|| CChanException::new("Empty array".to_string(), ErrCode::ParaError)),
            _ => Err(CChanException::new(
                &format!("Unknown trendModel Type = {:?}", self.trend_type),
                ErrCode::ParaError,
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trend_model_mean() {
        let mut model = CTrendModel::new(TrendType::Mean, 3);
        assert_eq!(model.add(1.0).unwrap(), 1.0);
        assert_eq!(model.add(2.0).unwrap(), 1.5);
        assert_eq!(model.add(3.0).unwrap(), 2.0);
        assert_eq!(model.add(4.0).unwrap(), 3.0);
    }

    #[test]
    fn test_trend_model_max() {
        let mut model = CTrendModel::new(TrendType::Max, 3);
        assert_eq!(model.add(1.0).unwrap(), 1.0);
        assert_eq!(model.add(3.0).unwrap(), 3.0);
        assert_eq!(model.add(2.0).unwrap(), 3.0);
        assert_eq!(model.add(4.0).unwrap(), 4.0);
    }

    #[test]
    fn test_trend_model_min() {
        let mut model = CTrendModel::new(TrendType::Min, 3);
        assert_eq!(model.add(3.0).unwrap(), 3.0);
        assert_eq!(model.add(1.0).unwrap(), 1.0);
        assert_eq!(model.add(2.0).unwrap(), 1.0);
        assert_eq!(model.add(4.0).unwrap(), 1.0);
    }

    #[test]
    fn test_trend_model_unknown_type() {
        let mut model = CTrendModel::new(TrendType::Unknown, 3);
        assert!(model.add(1.0).is_err());
    }
}
