use crate::csv_util::read_kline_from_csv;
use clap::Parser;
use czsc::{Analyzer, CChanConfig};
use std::path::PathBuf;
use std::time::Instant;
mod csv_util;

use czsc::Kline;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;

fn parse(opt: &Opt) {
    let fname = PathBuf::from(&opt.csv);
    let klines = read_kline_from_csv(&fname);
    println!("csv loaded");

    let mut analyzer = if let Some(config_path) = &opt.config {
        let config_str = fs::read_to_string(config_path).expect("Failed to read config file");
        let config = CChanConfig::from_json(&config_str).expect("Failed to parse config JSON");
        Analyzer::new(0, config)
    } else {
        Analyzer::new(0, CChanConfig::default())
    };

    if opt.batch {
        analyzer.step_calculation = false;
    }

    let output_dir = PathBuf::from(&opt.csv)
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| format!("{}_output", s))
        .unwrap_or_else(|| "output".to_string());

    czsc_parse(&mut analyzer, &klines, &output_dir)
}

fn czsc_parse(ca: &mut Analyzer, klines: &[Kline], output_dir: &str) {
    println!("start parse");
    let pb = ProgressBar::new(klines.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "[{elapsed_precise}] {percent_precise}% {wide_bar:.cyan/gray} {human_pos}/{human_len} {msg}\t\t",
            )
            .unwrap(), //.progress_chars("##-"),
    );
    let start_time = Instant::now();
    klines.iter().for_each(|k| {
        ca.add_k(k);
        pb.inc(1);
    });

    if ca.step_calculation {
        ca.cal_seg_and_zs();
    }

    let duration = start_time.elapsed();
    pb.finish_with_message("done");

    let _ = ca.to_csv(output_dir);
    println!(
        "parse time:{}s start:{} end:{}\nbar:{} candle:{} bi:{} seg:{} zs:{} bsp:{} seg_seg:{} seg_zs:{} seg_bsp:{}",
        duration.as_secs(),
        ca.bar_list().first().unwrap().time,
        ca.bar_list().last().unwrap().time,
        ca.bar_list().len(),
        ca.candle_list().len(),
        ca.bi_list().len(),
        ca.seg_list().len(),
        ca.bi_zs_list().len(),
        ca.bi_bsp_list().len(),
        ca.seg_seg_list().len(),
        ca.seg_zs_list().len(),
        ca.seg_bsp_list().len(),
    );

    // 保存配置文件
    let config_json = ca.config().to_json().expect("Failed to serialize config");
    let config_path = format!("{}/config.json", output_dir);
    fs::write(&config_path, config_json).expect("Failed to write config file");
}

#[cfg(not(target_env = "msvc"))]
use jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[derive(Parser, Debug, Clone)]
#[command(name = "czsc cli")]
#[command(about = "缠中说禅")]
struct Opt {
    #[arg(index = 1)]
    csv: String,

    #[arg(short, long, help = "Path to JSON config file")]
    config: Option<String>,

    #[arg(short, long, help = "Batch")]
    batch: bool,
}

fn main() {
    let opt = Opt::parse();

    parse(&opt)
}
