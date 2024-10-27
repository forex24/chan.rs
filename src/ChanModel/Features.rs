use std::collections::HashMap;

pub struct CFeatures {
    features: HashMap<String, f64>,
}

impl CFeatures {
    pub fn new(init_feat: Option<HashMap<String, f64>>) -> Self {
        CFeatures {
            features: init_feat.unwrap_or_default(),
        }
    }

    pub fn items(&self) -> impl Iterator<Item = (&String, &f64)> {
        self.features.iter()
    }

    pub fn get(&self, k: &str) -> Option<&f64> {
        self.features.get(k)
    }

    pub fn add_feat(&mut self, inp1: impl Into<FeatureInput>, inp2: Option<f64>) {
        match inp1.into() {
            FeatureInput::Single(key, value) => {
                self.features.insert(key, value);
            }
            FeatureInput::Multiple(map) => {
                self.features.extend(map);
            }
        }
    }
}

pub enum FeatureInput {
    Single(String, f64),
    Multiple(HashMap<String, f64>),
}

impl From<(String, f64)> for FeatureInput {
    fn from(pair: (String, f64)) -> Self {
        FeatureInput::Single(pair.0, pair.1)
    }
}

impl From<HashMap<String, f64>> for FeatureInput {
    fn from(map: HashMap<String, f64>) -> Self {
        FeatureInput::Multiple(map)
    }
}
