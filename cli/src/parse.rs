use crate::csv_util::read_kline_from_csv;
use crate::Opt;
use czsc::Analyzer;
use std::path::PathBuf;
use std::time::Instant;

use czsc::Kline;
use indicatif::ProgressBar;

pub async fn parse(opt: &Opt) {
    let mut fname = opt.input.clone().unwrap_or(PathBuf::from(&opt.symbol));
    fname.set_extension("csv");
    let klines = read_kline_from_csv(&fname);

    czsc_parse(&klines)
}

fn czsc_parse(klines: &[Kline]) {
    let mut ca = Analyzer::default();
    println!("start parse");
    let pb = ProgressBar::new(klines.len() as u64);
    let start_time = Instant::now();
    klines.iter().enumerate().for_each(|(_step_index, k)| {
        ca.add_k(k);
        pb.inc(1);
    });
    let duration = start_time.elapsed();
    pb.finish_with_message("done");
    println!(
        "parse time:{}s\nbsp count:{} seg count:{} bi count:{}",
        duration.as_secs(),
        ca.bi_bsp_list().len(),
        ca.seg_list().len(),
        ca.bi_list().len()
    );
}
