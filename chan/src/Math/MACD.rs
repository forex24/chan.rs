#[derive(Clone, Debug)]
pub struct CMACDItem {
    pub fast_ema: f64,
    pub slow_ema: f64,
    pub dif: f64,
    pub dea: f64,
    pub macd: f64,
}

impl CMACDItem {
    pub fn new(fast_ema: f64, slow_ema: f64, dif: f64, dea: f64) -> Self {
        CMACDItem {
            fast_ema,
            slow_ema,
            dif,
            dea,
            macd: 2.0 * (dif - dea),
        }
    }
}

pub struct CMACD {
    macd_info: Vec<CMACDItem>,
    fastperiod: f64,
    slowperiod: f64,
    signalperiod: f64,
}

impl CMACD {
    pub fn new(fastperiod: f64, slowperiod: f64, signalperiod: f64) -> Self {
        CMACD {
            macd_info: Vec::new(),
            fastperiod,
            slowperiod,
            signalperiod,
        }
    }

    pub fn add(&mut self, value: f64) -> CMACDItem {
        if self.macd_info.is_empty() {
            let item = CMACDItem::new(value, value, 0.0, 0.0);
            self.macd_info.push(item.clone());
            item
        } else {
            let last = self.macd_info.last().unwrap();
            let fast_ema =
                (2.0 * value + (self.fastperiod - 1.0) * last.fast_ema) / (self.fastperiod + 1.0);
            let slow_ema =
                (2.0 * value + (self.slowperiod - 1.0) * last.slow_ema) / (self.slowperiod + 1.0);
            let dif = fast_ema - slow_ema;
            let dea =
                (2.0 * dif + (self.signalperiod - 1.0) * last.dea) / (self.signalperiod + 1.0);
            let item = CMACDItem::new(fast_ema, slow_ema, dif, dea);
            self.macd_info.push(item.clone());
            item
        }
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmacd() {
        let mut macd = CMACD::new(12.0, 26.0, 9.0);
        let values = vec![10.0, 12.0, 15.0, 14.0, 13.0, 16.0, 18.0, 20.0];

        for value in values {
            let item = macd.add(value);
            println!("Value: {}, MACD: {:.4}", value, item.macd);
        }
    }
}
*/
