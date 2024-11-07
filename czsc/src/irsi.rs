#[derive(Debug, Clone)]
pub struct CRSI {
    pub close_arr: Vec<f64>,
    pub period: usize,
    pub diff: Vec<f64>,
    pub up: Vec<f64>,
    pub down: Vec<f64>,
}

impl CRSI {
    pub fn new(period: usize) -> Self {
        Self {
            close_arr: Vec::new(),
            period,
            diff: Vec::new(),
            up: Vec::new(),
            down: Vec::new(),
        }
    }

    pub fn add(&mut self, close: f64) -> f64 {
        self.close_arr.push(close);
        if self.close_arr.len() == 1 {
            return 50.0;
        }
        self.diff.push(
            self.close_arr[self.close_arr.len() - 1] - self.close_arr[self.close_arr.len() - 2],
        );
        if self.diff.len() < self.period {
            self.up
                .push(self.diff.iter().filter(|&&x| x > 0.0).sum::<f64>() / self.period as f64);
            self.down.push(
                self.diff
                    .iter()
                    .filter(|&&x| x < 0.0)
                    .map(|&x| -x)
                    .sum::<f64>()
                    / self.period as f64,
            );
        } else {
            let (upval, downval) = if self.diff[self.diff.len() - 1] > 0.0 {
                (self.diff[self.diff.len() - 1], 0.0)
            } else {
                (0.0, -self.diff[self.diff.len() - 1])
            };
            self.up.push(
                (self.up[self.up.len() - 1] * (self.period as f64 - 1.0) + upval)
                    / self.period as f64,
            );
            self.down.push(
                (self.down[self.down.len() - 1] * (self.period as f64 - 1.0) + downval)
                    / self.period as f64,
            );
        }
        let rs = if self.down[self.down.len() - 1] != 0.0 {
            self.up[self.up.len() - 1] / self.down[self.down.len() - 1]
        } else {
            0.0
        };
        let rsi = 100.0 - 100.0 / (1.0 + rs);
        rsi
    }
}
