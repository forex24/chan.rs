pub struct RSI {
    close_arr: Vec<f64>,
    period: usize,
    diff: Vec<f64>,
    up: Vec<f64>,
    down: Vec<f64>,
}

impl RSI {
    pub fn new(period: usize) -> Self {
        RSI {
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

        let last_diff = self.close_arr.last().unwrap() - self.close_arr[self.close_arr.len() - 2];
        self.diff.push(last_diff);

        if self.diff.len() < self.period {
            let up_sum: f64 = self.diff.iter().filter(|&&x| x > 0.0).sum();
            let down_sum: f64 = self.diff.iter().filter(|&&x| x < 0.0).map(|&x| -x).sum();
            self.up.push(up_sum / self.period as f64);
            self.down.push(down_sum / self.period as f64);
        } else {
            let (upval, downval) = if last_diff > 0.0 {
                (last_diff, 0.0)
            } else {
                (0.0, -last_diff)
            };
            let last_up = *self.up.last().unwrap();
            let last_down = *self.down.last().unwrap();
            self.up
                .push((last_up * (self.period - 1) as f64 + upval) / self.period as f64);
            self.down
                .push((last_down * (self.period - 1) as f64 + downval) / self.period as f64);
        }

        let rs = if *self.down.last().unwrap() != 0.0 {
            self.up.last().unwrap() / self.down.last().unwrap()
        } else {
            0.0
        };
        100.0 - 100.0 / (1.0 + rs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsi() {
        let mut rsi = RSI::new(14);
        let closes = vec![
            44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42, 45.84, 46.08, 45.89, 46.03,
            45.61, 46.28, 46.28, 46.00,
        ];
        let mut results = Vec::new();
        for close in closes {
            results.push(rsi.add(close));
        }
        println!("RSI values: {:?}", results);
        assert!((results.last().unwrap() - 70.53).abs() < 0.01);
    }
}
