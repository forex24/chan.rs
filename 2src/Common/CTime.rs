// File: chan/src/Common/CTime.rs

use chrono::Datelike;
use chrono::{DateTime, NaiveDateTime, Timelike, Utc};
use std::cmp::Ordering;
use std::fmt;
#[derive(Clone, Debug, Copy)]
pub struct CTime {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
    pub auto: bool,
    pub ts: f64,
}

impl CTime {
    pub fn new(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
        auto: bool,
    ) -> Self {
        let mut ctime = CTime {
            year,
            month,
            day,
            hour,
            minute,
            second,
            auto,
            ts: 0.0,
        };
        ctime.set_timestamp();
        ctime
    }

    pub fn from_naive_date_time(ndt: NaiveDateTime, auto: bool, ts: f64) -> Self {
        CTime {
            year: ndt.year(),
            month: ndt.month(),
            day: ndt.day(),
            hour: ndt.hour(),
            minute: ndt.minute(),
            second: ndt.second(),
            auto,
            ts,
        }
    }

    pub fn to_str(&self) -> String {
        if self.hour == 0 && self.minute == 0 {
            format!("{:04}/{:02}/{:02}", self.year, self.month, self.day)
        } else {
            format!(
                "{:04}/{:02}/{:02} {:02}:{:02}",
                self.year, self.month, self.day, self.hour, self.minute
            )
        }
    }

    pub fn to_date_str(&self, splt: &str) -> String {
        format!(
            "{:04}{}{:02}{}{:02}",
            self.year, splt, self.month, splt, self.day
        )
    }

    pub fn to_date(&self) -> CTime {
        CTime::new(self.year, self.month, self.day, 0, 0, 0, false)
    }

    pub fn set_timestamp(&mut self) {
        let date = if self.hour == 0 && self.minute == 0 && self.auto {
            NaiveDateTime::new(
                chrono::NaiveDate::from_ymd_opt(self.year, self.month, self.day).unwrap(),
                chrono::NaiveTime::from_hms_opt(23, 59, self.second).unwrap(),
            )
        } else {
            NaiveDateTime::new(
                chrono::NaiveDate::from_ymd_opt(self.year, self.month, self.day).unwrap(),
                chrono::NaiveTime::from_hms_opt(self.hour, self.minute, self.second).unwrap(),
            )
        };
        self.ts = DateTime::<Utc>::from_utc(date, Utc).timestamp() as f64;
    }

    pub fn from_timestamp(timestamp: f64) -> Self {
        let naive = NaiveDateTime::from_timestamp_opt(timestamp as i64, 0).unwrap_or_default();
        CTime::from_naive_date_time(naive, true, timestamp)
    }

    pub fn from_datetime_str(datetime_str: &str) -> Result<Self, chrono::ParseError> {
        let naive = NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%d %H:%M:%S")?;
        Ok(CTime::from_naive_date_time(
            naive,
            true,
            DateTime::<Utc>::from_utc(naive, Utc).timestamp() as f64,
        ))
    }

    pub fn from_timestamp_millis(timestamp_ms: i64) -> Self {
        let naive = NaiveDateTime::from_timestamp_millis(timestamp_ms).expect("Invalid timestamp");
        Self::from_naive_date_time(naive, true, timestamp_ms as f64)
    }
}

impl fmt::Display for CTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl PartialEq for CTime {
    fn eq(&self, other: &Self) -> bool {
        self.ts == other.ts
    }
}

impl Eq for CTime {}

impl PartialOrd for CTime {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.ts.partial_cmp(&other.ts)
    }
}

impl Ord for CTime {
    fn cmp(&self, other: &Self) -> Ordering {
        self.ts.partial_cmp(&other.ts).unwrap()
    }
}
