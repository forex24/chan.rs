use crate::Common::CEnum::FxCheckMethod;
use crate::Common::ChanException::{CChanException, ErrCode};

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
        bi_algo: Option<String>,
        is_strict: Option<bool>,
        bi_fx_check: Option<String>,
        gap_as_kl: Option<bool>,
        bi_end_is_peak: Option<bool>,
        bi_allow_sub_peak: Option<bool>,
    ) -> Result<Self, CChanException> {
        let bi_fx_check = match bi_fx_check.as_deref().unwrap_or("half") {
            "strict" => FxCheckMethod::Strict,
            "loss" => FxCheckMethod::Loss,
            "half" => FxCheckMethod::Half,
            "totally" => FxCheckMethod::Totally,
            unknown => {
                return Err(CChanException::new(
                    format!("unknown bi_fx_check={}", unknown),
                    ErrCode::ParaError,
                ))
            }
        };

        Ok(Self {
            bi_algo: bi_algo.unwrap_or_else(|| "normal".to_string()),
            is_strict: is_strict.unwrap_or(true),
            bi_fx_check,
            gap_as_kl: gap_as_kl.unwrap_or(true),
            bi_end_is_peak: bi_end_is_peak.unwrap_or(true),
            bi_allow_sub_peak: bi_allow_sub_peak.unwrap_or(true),
        })
    }
}

impl Default for CBiConfig {
    fn default() -> Self {
        Self {
            bi_algo: "normal".to_string(),
            is_strict: true,
            bi_fx_check: FxCheckMethod::Strict,
            gap_as_kl: true,
            bi_end_is_peak: true,
            bi_allow_sub_peak: true,
        }
    }
}
