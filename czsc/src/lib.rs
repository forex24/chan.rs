//! 缠中说禅分析库
//!
//! czsc实现了缠中说禅的理论

pub mod common;
pub use common::*;

mod cenum;
pub use cenum::*;
mod chan_exception;
pub use chan_exception::*;

mod func_util;
mod handle;
pub use func_util::*;
pub use handle::*;
#[macro_use]
mod slice_macro;

mod metric;
pub use metric::*;

mod bar;
mod bar_list;
pub use bar::*;
pub use bar_list::*;

mod candle;
mod candle_list;
mod config;

pub use candle::*;
pub use candle_list::*;
pub use config::*;

mod cbi;
mod cbi_algo;
mod cbi_config;
mod cbi_list;
pub use cbi::*;
pub use cbi_config::*;
pub use cbi_list::*;

// config
mod bsp_config;
mod cseg_config;
mod czs_config;
pub use bsp_config::*;
pub use cseg_config::*;
pub use czs_config::*;

// seg
mod ceigen;
mod ceigen_fx;
mod cseg;
mod cseg_list;
mod cseg_list_comm;

pub use ceigen::*;
pub use ceigen_fx::*;
pub use cseg::*;
pub use cseg_list::*;

// zs
mod czs;
mod czs_list;
pub use czs::*;
pub use czs_list::*;

// bsp
mod bsp;
mod bsp_list;
mod features;
pub use bsp::*;
pub use bsp_list::*;
pub use features::*;

//mod ctrade_info;
//mod demark;
mod iboll;
pub use iboll::*;

mod imacd;
pub use imacd::*;

//mod ikdj;
//pub use ikdj::*;

//mod irsi;
//pub use irsi::*;

//pub use ctrade_info::*;

//pub use demark::*;

mod line_type;
pub use line_type::*;

/*mod metric;
pub use metric::*;
*/
mod kline;
pub use kline::*;
mod analyzer;
mod storage;
pub use analyzer::*;

// czsc/src/lib.rs
use pyo3::prelude::*;

// 保留原有的模块导出
pub mod common;
pub use common::*;
mod cenum;
pub use cenum::*;
// ... 其他原有的模块导出 ...

// 添加 Python 模块相关代码
mod python;
use python::analyzer::PyAnalyzer;

/// czsc 的 Python 模块
#[pymodule]
fn _czsc(_py: Python, m: &PyModule) -> PyResult<()> {
    // 注册异常类型
    register_exceptions(m)?;
    
    // 注册类型
    m.add_class::<PyAnalyzer>()?;
    
    // 添加版本信息
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    
    Ok(())
}

// 注册自定义异常
fn register_exceptions(m: &PyModule) -> PyResult<()> {
    let py = m.py();
    
    // 创建基础异常类
    let base_exception = py.get_type::<pyo3::exceptions::PyException>();
    
    // 注册自定义异常
    let czsc_error = create_exception!(m, "CzscError", base_exception)?;
    let invalid_data_error = create_exception!(m, "InvalidDataError", czsc_error)?;
    let calculation_error = create_exception!(m, "CalculationError", czsc_error)?;
    
    // 将异常添加到模块
    m.add("CzscError", czsc_error)?;
    m.add("InvalidDataError", invalid_data_error)?;
    m.add("CalculationError", calculation_error)?;
    
    Ok(())
}

// 项目结构示意
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version() {
        let version = env!("CARGO_PKG_VERSION");
        assert!(!version.is_empty());
    }
}
