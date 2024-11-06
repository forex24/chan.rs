use crate::FxCheckMethod;

#[derive(Debug, Clone)]
pub struct CBiConfig {
    pub bi_algo: String,
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
            bi_algo: "normal".to_string(),
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
            bi_algo: "normal".to_string(),
            bi_fx_check: FxCheckMethod::Strict,
            gap_as_kl: false,
            bi_end_is_peak: true,
            is_strict: true,
            bi_allow_sub_peak: true,
        }
    }

    fn default_loss() -> Self {
        Self {
            bi_algo: "normal".to_string(),
            bi_fx_check: FxCheckMethod::Loss,
            gap_as_kl: true,
            bi_end_is_peak: true,
            is_strict: false,
            bi_allow_sub_peak: false,
        }
    }
}

impl Default for CBiConfig {
    fn default() -> Self {
        Self::default_loss()
    }
}
