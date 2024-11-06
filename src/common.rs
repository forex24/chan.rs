use chrono::{DateTime, Utc};

pub trait IHighLow {
    fn high(&self) -> f64;
    fn low(&self) -> f64;
}

pub trait ICandlestick: IHighLow {
    fn unix_time(&self) -> i64;
    fn open(&self) -> f64;
    fn close(&self) -> f64;
}

pub trait IPoint {
    fn unix_time(&self) -> i64 {
        self.time().timestamp_millis()
    }

    fn time(&self) -> DateTime<Utc>;
    fn price(&self) -> f64;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub time: DateTime<Utc>,
    pub price: f64,
}

impl Point {
    pub fn new(time: DateTime<Utc>, price: f64) -> Self {
        Self { time, price }
    }
}

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.time, self.price)
    }
}

impl IPoint for Point {
    fn time(&self) -> DateTime<Utc> {
        self.time
    }

    fn price(&self) -> f64 {
        self.price
    }
}
