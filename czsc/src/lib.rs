//! 缠中说禅分析库
//!
//! czsc实现了缠中说禅的理论

pub mod common;
pub use common::*;

mod cenum;
pub use cenum::*;

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
