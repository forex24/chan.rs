use crate::LeftSegMethod;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CSegConfig {
    pub left_method: LeftSegMethod, // 剩余那些不能归入确定线段的笔如何处理成段，
}

impl Default for CSegConfig {
    fn default() -> Self {
        Self {
            left_method: LeftSegMethod::Peak,
        }
    }
}
