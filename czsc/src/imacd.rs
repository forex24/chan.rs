use crate::{AsHandle, Handle};

#[derive(Debug, Clone)]
pub struct CMACDItem {
    handle: Handle<Self>,
    pub fast_ema: f64,
    pub slow_ema: f64,
    pub dif: f64,
    pub dea: f64,
    pub macd: f64,
}

impl CMACDItem {
    #[allow(clippy::borrowed_box)]
    pub fn new(
        box_vec: &Box<Vec<Self>>,
        index: usize,
        fast_ema: f64,
        slow_ema: f64,
        dif: f64,
        dea: f64,
    ) -> CMACDItem {
        CMACDItem {
            handle: Handle::new(box_vec, index),
            fast_ema,
            slow_ema,
            dif,
            dea,
            macd: 2.0 * (dif - dea),
        }
    }
}

impl_handle!(CMACDItem);

#[derive(Debug, Clone)]
pub struct CMACD {
    pub macd_info: Box<Vec<CMACDItem>>,
    pub fastperiod: usize,
    pub slowperiod: usize,
    pub signalperiod: usize,
}

impl CMACD {
    pub fn new(fastperiod: usize, slowperiod: usize, signalperiod: usize) -> CMACD {
        CMACD {
            macd_info: Box::<Vec<CMACDItem>>::default(),
            fastperiod,
            slowperiod,
            signalperiod,
        }
    }

    pub fn add(&mut self, value: f64) -> Handle<CMACDItem> {
        let item = if let Some(last) = self.macd_info.last() {
            let fast_ema = (2.0 * value + (self.fastperiod as f64 - 1.0) * last.fast_ema)
                / (self.fastperiod as f64 + 1.0);
            let slow_ema = (2.0 * value + (self.slowperiod as f64 - 1.0) * last.slow_ema)
                / (self.slowperiod as f64 + 1.0);
            let dif = fast_ema - slow_ema;
            let dea = (2.0 * dif + (self.signalperiod as f64 - 1.0) * last.dea)
                / (self.signalperiod as f64 + 1.0);
            CMACDItem::new(
                &self.macd_info,
                self.macd_info.len(),
                fast_ema,
                slow_ema,
                dif,
                dea,
            )
        } else {
            CMACDItem::new(
                &self.macd_info,
                self.macd_info.len(),
                value,
                value,
                0.0,
                0.0,
            )
        };
        self.macd_info.push(item);
        self.macd_info.last().unwrap().as_handle()
    }
}
