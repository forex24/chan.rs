use crate::{CPivotAlgo, CPivotCombineMode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CZsConfig {
    pub need_combine: bool,
    pub zs_combine_mode: CPivotCombineMode,
    pub one_bi_zs: bool,
    pub zs_algo: CPivotAlgo,
}

impl Default for CZsConfig {
    fn default() -> Self {
        Self {
            need_combine: true,
            zs_combine_mode: CPivotCombineMode::Zs,
            one_bi_zs: false,
            zs_algo: CPivotAlgo::Normal,
        }
    }
}

impl CZsConfig {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}
