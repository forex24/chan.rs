use std::collections::VecDeque;

fn truncate(x: f64) -> f64 {
    if x != 0.0 {
        x
    } else {
        1e-7
    }
}

pub struct BOLLMetric {
    pub theta: f64,
    pub up: f64,
    pub down: f64,
    pub mid: f64,
}

impl BOLLMetric {
    pub fn new(ma: f64, theta: f64) -> Self {
        let theta = truncate(theta);
        BOLLMetric {
            theta,
            up: ma + 2.0 * theta,
            down: truncate(ma - 2.0 * theta),
            mid: ma,
        }
    }
}

pub struct BollModel {
    n: usize,
    arr: VecDeque<f64>,
}

impl BollModel {
    pub fn new(n: usize) -> Result<Self, &'static str> {
        if n <= 1 {
            Err("N must be greater than 1")
        } else {
            Ok(BollModel {
                n,
                arr: VecDeque::with_capacity(n),
            })
        }
    }

    pub fn add(&mut self, value: f64) -> BOLLMetric {
        self.arr.push_back(value);
        if self.arr.len() > self.n {
            self.arr.pop_front();
        }

        let ma = self.arr.iter().sum::<f64>() / self.arr.len() as f64;
        let theta = (self.arr.iter().map(|&x| (x - ma).powi(2)).sum::<f64>()
            / self.arr.len() as f64)
            .sqrt();

        BOLLMetric::new(ma, theta)
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boll_model() {
        let mut model = BollModel::new(20).unwrap();
        for i in 1..=25 {
            let metric = model.add(i as f64);
            println!(
                "Value: {}, UP: {}, DOWN: {}, MID: {}",
                i, metric.up, metric.down, metric.mid
            );
        }
    }

    #[test]
    fn test_invalid_n() {
        assert!(BollModel::new(1).is_err());
        assert!(BollModel::new(0).is_err());
    }
}
*/
