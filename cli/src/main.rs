use std::{env, path::PathBuf};

use dotenv::dotenv;
use period::Period;
use std::str::FromStr;
use structopt::*;

use tracing_subscriber::{filter, prelude::*};
mod action;
use action::Action;

//mod bi;
//mod bsp;
//mod check;
mod csv_util;
//mod download;
//mod fx;
mod parse;
mod period;
//mod plot;
//mod seg;
mod util;
//mod web;
//mod zs;

#[cfg(not(target_env = "msvc"))]
extern crate jemallocator;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[allow(dead_code)]
#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "czsc cli", about = "缠中说禅")]
struct Opt {
    #[structopt(long)]
    action: Action,

    #[structopt(long)]
    symbol: String,

    #[structopt(long)]
    start: Option<String>,

    #[structopt(long)]
    end: Option<String>,

    #[structopt(long)]
    period: Option<Period>,

    #[structopt(long, parse(from_os_str))]
    output: Option<PathBuf>,

    #[structopt(long, parse(from_os_str))]
    input: Option<PathBuf>,

    #[structopt(long)]
    days: Option<u64>,

    #[structopt(long, parse(from_os_str))]
    from: Option<PathBuf>,

    #[structopt(long, parse(from_os_str))]
    to: Option<PathBuf>,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    // 初始化
    let env = dotenv();
    if env.is_err() {
        println!("dotenv init failed!, please check .env file");
        return;
    }
    let stdout_log = tracing_subscriber::fmt::layer().with_ansi(true).pretty();

    //let file_appender = tracing_appender::rolling::daily("./logs", "download_debug");
    //let (non_blocking_appender, _worker_guard) = tracing_appender::non_blocking(file_appender);

    let stdout_log_level =
        tracing::Level::from_str(&env::var("stdout_log_level").unwrap_or("info".to_string()))
            .unwrap();
    //let file_log_level =
    //    tracing::Level::from_str(&env::var("file_log_level").unwrap_or("debug".to_string()))
    //        .unwrap();

    //let debug_log = tracing_subscriber::fmt::layer()
    //    .with_ansi(false)
    //    .with_writer(non_blocking_appender);

    tracing_subscriber::registry()
        .with(
            stdout_log.with_filter(filter::LevelFilter::from_level(stdout_log_level)), // Combine the filtered `stdout_log` layer with the
                                                                                       // `debug_log` layer, producing a new `Layered` layer.
                                                                                       //.and_then(debug_log.with_filter(filter::LevelFilter::from_level(file_log_level))),
        )
        .init();

    // 分析命令行
    let opt = Opt::from_args();

    match opt.action {
        Action::Parse => parse::parse(&opt).await,
        //Action::Download => download::download(&opt).await,
        //Action::Check => check::parse(&opt).await,
        //Action::Plot => plot::parse(&opt).await,
        //Action::Web => web::parse(&opt).await,
        //Action::Fx => fx::parse(&opt).await,
        //Action::Bi => bi::parse(&opt).await,
        //Action::Seg => seg::parse(&opt).await,
        //Action::Zs => zs::parse(&opt).await,
        //Action::Bsp => bsp::parse(&opt).await,
    }
}
