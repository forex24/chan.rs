pub struct CZSConfig {
    pub need_combine: bool,
    pub zs_combine_mode: String,
    pub one_bi_zs: bool,
    pub zs_algo: String,
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
            zs_combine_mode: zs_combine_mode.unwrap_or_else(|| "zs".to_string()),
            one_bi_zs: one_bi_zs.unwrap_or(false),
            zs_algo: zs_algo.unwrap_or_else(|| "normal".to_string()),
        }
    }
}

impl Default for CZSConfig {
    fn default() -> Self {
        Self::new(None, None, None, None)
    }
}
