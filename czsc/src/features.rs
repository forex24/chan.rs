use std::{collections::HashMap, fmt::Display};

#[derive(Debug, Clone)]
pub struct CFeatures {
    pub features: HashMap<String, Option<f64>>,
}

impl CFeatures {
    pub fn new(init_feat: Option<HashMap<String, Option<f64>>>) -> CFeatures {
        CFeatures {
            features: init_feat.unwrap_or_default(),
        }
    }

    pub fn items(&self) -> impl Iterator<Item = (&String, &Option<f64>)> {
        self.features.iter()
    }

    pub fn get_item(&self, k: &str) -> Option<&Option<f64>> {
        self.features.get(k)
    }

    pub fn add_feat(&mut self, inp1: String, inp2: Option<f64>) {
        self.features.insert(inp1, inp2);
    }
}

impl Display for CFeatures {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "kv:{}",
            self.features
                .iter()
                .map(|(k, v)| format!("{} => {:?}", k, v))
                .collect::<Vec<String>>()
                .join(",")
        )
    }
}
