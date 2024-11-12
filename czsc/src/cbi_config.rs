use crate::{BiAlgo, FxCheckMethod};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CBiConfig {
    pub bi_algo: BiAlgo,
    pub is_strict: bool,
    pub bi_fx_check: FxCheckMethod,
    pub gap_as_kl: bool,
    pub bi_end_is_peak: bool,
    pub bi_allow_sub_peak: bool,
}

impl CBiConfig {
    pub fn new(
        is_strict: bool,
        bi_fx_check: FxCheckMethod,
        gap_as_kl: bool,
        bi_end_is_peak: bool,
        bi_allow_sub_peak: bool,
    ) -> Self {
        CBiConfig {
            bi_algo: BiAlgo::Normal,
            is_strict,
            bi_fx_check,
            gap_as_kl,
            bi_end_is_peak,
            bi_allow_sub_peak,
        }
    }

    #[allow(dead_code)]
    fn default_strict() -> Self {
        Self {
            bi_algo: BiAlgo::Normal,
            bi_fx_check: FxCheckMethod::Strict,
            gap_as_kl: false,
            bi_end_is_peak: true,
            is_strict: true,
            bi_allow_sub_peak: true,
        }
    }

    #[allow(dead_code)]
    fn default_loss() -> Self {
        Self {
            bi_algo: BiAlgo::Normal,
            bi_fx_check: FxCheckMethod::Loss,
            gap_as_kl: true,
            bi_end_is_peak: true,
            is_strict: false,
            bi_allow_sub_peak: false,
        }
    }
}

impl CBiConfig {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl Default for CBiConfig {
    fn default() -> Self {
        Self {
            bi_algo: BiAlgo::Normal,
            is_strict: true,
            gap_as_kl: false,
            bi_end_is_peak: true,
            bi_allow_sub_peak: true,
            bi_fx_check: FxCheckMethod::Strict,
        }
    }
}
