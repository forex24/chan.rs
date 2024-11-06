use crate::{CPivotAlgo, CPivotCombineMode};

#[derive(Debug, Clone, Copy)]
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
