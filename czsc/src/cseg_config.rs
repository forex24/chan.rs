use crate::LeftSegMethod;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CSegConfig {
    pub left_method: LeftSegMethod,
}

impl Default for CSegConfig {
    fn default() -> Self {
        Self {
            left_method: LeftSegMethod::Peak,
        }
    }
}

impl CSegConfig {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}
