use std::collections::VecDeque;

//TODO：参见 YATA
pub fn truncate(x: f64) -> f64 {
    if x != 0.0 {
        x
    } else {
        1e-7
    }
}

#[derive(Debug, Clone)]
pub struct BollMetric {
    pub theta: f64,
    pub up: f64,
    pub down: f64,
    pub mid: f64,
}

impl BollMetric {
    pub fn new(ma: f64, theta: f64) -> BollMetric {
        BollMetric {
            theta: truncate(theta),
            up: ma + 2.0 * theta,
            down: truncate(ma - 2.0 * theta),
            mid: ma,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BollModel {
    n: usize,
    arr: VecDeque<f64>,
}

impl BollModel {
    pub fn new(n: usize) -> BollModel {
        debug_assert!(n > 1);
        BollModel {
            n,
            arr: VecDeque::new(),
        }
    }

    pub fn add(&mut self, value: f64) -> BollMetric {
        self.arr.push_back(value);
        if self.arr.len() > self.n {
            self.arr.pop_front();
        }
        let ma: f64 = self.arr.iter().sum::<f64>() / self.arr.len() as f64;
        let theta: f64 = (self.arr.iter().map(|&x| (x - ma).powi(2)).sum::<f64>()
            / self.arr.len() as f64)
            .sqrt();
        BollMetric::new(ma, theta)
    }
}
