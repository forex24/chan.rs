use std::fmt::Display;

use chrono::{DateTime, Utc};

use crate::{ICandlestick, IHighLow};

// 未经过包含处理的K线
#[derive(Debug, Clone, Copy)]
pub struct Kline {
    pub time: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub vol: f64,
}

impl Kline {
    pub fn new(time: DateTime<Utc>, open: f64, high: f64, low: f64, close: f64, vol: f64) -> Self {
        Self {
            time,
            open,
            high,
            low,
            close,
            vol,
        }
    }
}

impl Display for Kline {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "time:{} open:{} high:{} low:{} close:{} vol:{}",
            self.time, self.open, self.high, self.low, self.close, self.vol
        )
    }
}

impl IHighLow for Kline {
    fn high(&self) -> f64 {
        self.high
    }

    fn low(&self) -> f64 {
        self.low
    }
}

impl ICandlestick for Kline {
    fn unix_time(&self) -> i64 {
        self.time.timestamp_millis()
    }

    fn open(&self) -> f64 {
        self.open
    }

    fn close(&self) -> f64 {
        self.close
    }
}
