use crate::Common::{
    CEnum::LeftSegMethod,
    ChanException::{CChanException, ErrCode},
};
#[derive(Debug, Clone)]
pub struct CSegConfig {
    pub seg_algo: String,
    pub left_method: LeftSegMethod,
}

impl CSegConfig {
    pub fn new(seg_algo: String, left_method: String) -> Result<Self, CChanException> {
        let left_method = match left_method.as_str() {
            "all" => LeftSegMethod::All,
            "peak" => LeftSegMethod::Peak,
            _ => {
                return Err(CChanException::new(
                    format!("unknown left_seg_method={}", left_method),
                    ErrCode::ParaError,
                ))
            }
        };

        Ok(CSegConfig {
            seg_algo,
            left_method,
        })
    }
}

impl Default for CSegConfig {
    fn default() -> Self {
        CSegConfig {
            seg_algo: "chan".to_string(),
            left_method: LeftSegMethod::Peak,
        }
    }
}
