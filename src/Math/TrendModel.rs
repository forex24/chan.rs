use crate::Common::CEnum::TrendType;

pub struct CTrendModel {
    pub t: usize,
    pub arr: Vec<f64>,
    pub trend_type: TrendType,
}

impl CTrendModel {
    pub fn new(trend_type: TrendType, t: usize) -> Self {
        CTrendModel {
            t,
            arr: Vec::with_capacity(t),
            trend_type,
        }
    }

    pub fn add(&mut self, value: f64) -> f64 {
        self.arr.push(value);
        if self.arr.len() > self.t {
            self.arr = self.arr.split_off(self.arr.len() - self.t);
        }

        match self.trend_type {
            TrendType::Mean => self.arr.iter().sum::<f64>() / self.arr.len() as f64,

            TrendType::Max => self
                .arr
                .iter()
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .map(|&x| x)
                .unwrap(),

            TrendType::Min => self
                .arr
                .iter()
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .map(|&x| x)
                .unwrap(),
        }
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trend_model_mean() {
        let mut model = CTrendModel::new(TrendType::Mean, 3);
        assert_eq!(model.add(1.0), 1.0);
        assert_eq!(model.add(2.0), 1.5);
        assert_eq!(model.add(3.0), 2.0);
        assert_eq!(model.add(4.0), 3.0);
    }

    #[test]
    fn test_trend_model_max() {
        let mut model = CTrendModel::new(TrendType::Max, 3);
        assert_eq!(model.add(1.0), 1.0);
        assert_eq!(model.add(3.0), 3.0);
        assert_eq!(model.add(2.0), 3.0);
        assert_eq!(model.add(4.0), 4.0);
    }

    #[test]
    fn test_trend_model_min() {
        let mut model = CTrendModel::new(TrendType::Min, 3);
        assert_eq!(model.add(3.0), 3.0);
        assert_eq!(model.add(1.0), 1.0);
        assert_eq!(model.add(2.0), 1.0);
        assert_eq!(model.add(4.0), 1.0);
    }
}
*/
