use crate::Common::CEnum::{ZsAlgo, ZsCombineMode};

#[derive(Debug, Clone)]
pub struct CZSConfig {
    pub need_combine: bool,
    pub zs_combine_mode: ZsCombineMode,
    pub one_bi_zs: bool,
    pub zs_algo: ZsAlgo,
}

impl CZSConfig {
    pub fn new(
        need_combine: Option<bool>,
        zs_combine_mode: Option<String>,
        one_bi_zs: Option<bool>,
        zs_algo: Option<String>,
    ) -> Self {
        CZSConfig {
            need_combine: need_combine.unwrap_or(true),
            zs_combine_mode: zs_combine_mode
                .unwrap_or_else(|| "zs".to_string())
                .parse()
                .unwrap_or(ZsCombineMode::Zs),
            one_bi_zs: one_bi_zs.unwrap_or(false),
            zs_algo: zs_algo
                .unwrap_or_else(|| "normal".to_string())
                .parse()
                .unwrap_or(ZsAlgo::Normal),
        }
    }
}

impl Default for CZSConfig {
    fn default() -> Self {
        Self::new(None, None, None, None)
    }
}
