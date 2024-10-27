use std::collections::HashMap;

// Assuming TRADE_INFO_LST is defined elsewhere in your Rust project
use crate::Common::CEnum::TRADE_INFO_LST;

pub struct CTradeInfo {
    metric: HashMap<String, Option<f64>>,
}

impl CTradeInfo {
    pub fn new(info: &HashMap<String, f64>) -> Self {
        let mut metric = HashMap::new();
        for metric_name in TRADE_INFO_LST.iter() {
            metric.insert(metric_name.to_string(), info.get(metric_name).copied());
        }
        CTradeInfo { metric }
    }

    pub fn to_string(&self) -> String {
        self.metric
            .iter()
            .map(|(metric_name, value)| format!("{}:{:?}", metric_name, value))
            .collect::<Vec<String>>()
            .join(" ")
    }
}

// ... existing code ...

