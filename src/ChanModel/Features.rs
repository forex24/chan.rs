use std::collections::HashMap;

pub struct CFeatures {
    features: HashMap<String, Option<f64>>,
}

impl CFeatures {
    pub fn new(init_feat: Option<HashMap<String, Option<f64>>>) -> Self {
        CFeatures {
            features: init_feat.unwrap_or_default(),
        }
    }

    pub fn items(&self) -> impl Iterator<Item = (&String, &Option<f64>)> {
        self.features.iter()
    }

    pub fn get(&self, k: &str) -> Option<&Option<f64>> {
        self.features.get(k)
    }

    pub fn add_feat(&mut self, inp1: impl Into<FeatureInput>) {
        match inp1.into() {
            FeatureInput::Single(key, value) => {
                self.features.insert(key, Some(value));
            }
            FeatureInput::SingleOpt(key, opt_value) => {
                self.features.insert(key, opt_value);
            }
            FeatureInput::Multiple(map) => {
                for (key, value) in map {
                    self.features.insert(key, Some(value));
                }
            }
            FeatureInput::MultipleOpt(map) => {
                for (key, opt_value) in map {
                    self.features.insert(key, opt_value);
                }
            }
            FeatureInput::Dict(features) => {
                self.features.extend(features.features);
            }
        }
    }
}

pub enum FeatureInput {
    Single(String, f64),
    SingleOpt(String, Option<f64>),
    Multiple(HashMap<String, f64>),
    MultipleOpt(HashMap<String, Option<f64>>),
    Dict(CFeatures),
}

impl From<(&str, f64)> for FeatureInput {
    fn from(pair: (&str, f64)) -> Self {
        FeatureInput::Single(pair.0.to_string(), pair.1)
    }
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

impl From<HashMap<&str, f64>> for FeatureInput {
    fn from(map: HashMap<&str, f64>) -> Self {
        FeatureInput::Multiple(map.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
    }
}

impl From<HashMap<String, Option<f64>>> for FeatureInput {
    fn from(map: HashMap<String, Option<f64>>) -> Self {
        FeatureInput::MultipleOpt(map)
    }
}

impl From<HashMap<&str, Option<f64>>> for FeatureInput {
    fn from(map: HashMap<&str, Option<f64>>) -> Self {
        FeatureInput::MultipleOpt(map.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
    }
}

impl From<CFeatures> for FeatureInput {
    fn from(features: CFeatures) -> Self {
        FeatureInput::Dict(features)
    }
}

impl From<(String, Option<f64>)> for FeatureInput {
    fn from(pair: (String, Option<f64>)) -> Self {
        FeatureInput::SingleOpt(pair.0, pair.1)
    }
}

impl From<(&str, Option<f64>)> for FeatureInput {
    fn from(pair: (&str, Option<f64>)) -> Self {
        FeatureInput::SingleOpt(pair.0.to_string(), pair.1)
    }
}
