use crate::csv_util::read_kline_from_csv;
use crate::Opt;
use czsc::{Analyzer, Candle};
use czsc::{AsHandle, Kline};
use dump::context::KlcContext;
use dump::dump_fx::read_fx_dump_file;
use dump::SKlc;
use indicatif::ProgressBar;
use std::path::PathBuf;
use std::time::Instant;

pub async fn parse(opt: &Opt) {
    let mut fname = opt.input.clone().unwrap_or(PathBuf::from(&opt.symbol));
    fname.set_extension("csv");
    let klines = read_kline_from_csv(&fname);

    let mut fname = opt.input.clone().unwrap_or(PathBuf::from(&opt.symbol));
    fname.set_extension("fx.gz");
    let dump = read_fx_dump_file(fname.as_path()).unwrap();
    czsc_parse(&klines, &dump)
}

fn cmp_candle(sklc: &SKlc, k: &Candle) -> bool {
    (sklc.index == k.as_handle().index())
        && (sklc.fx_type == k.as_handle().fx_type)
        && (sklc.high == k.high)
        && (sklc.low == k.low)
        && (sklc.start_bar == k.lst[0].index())
        && (sklc.end_bar == k.lst.last().unwrap().index())
}

fn cmp_candle_list(_step_index: usize, ctx: &[KlcContext], klc_list: &[Box<Candle>]) -> bool {
    for (candle_index, c) in ctx.iter().enumerate() {
        let bi = &klc_list[candle_index];
        let sbi = c.klc.as_ref().unwrap();
        if !cmp_candle(sbi, bi) {
            return false;
        }
    }
    true
}

fn test_fx_ctx(ctx: &[KlcContext]) {
    let index = ctx[0].step_index;
    for c in ctx {
        assert!(c.step_index == index);
    }
}

fn czsc_parse(klines: &[Kline], dump: &[Vec<KlcContext>]) {
    let mut ca = Analyzer::default();
    let pb = ProgressBar::new(klines.len() as u64);
    let start_time = Instant::now();
    for (step_index, k) in klines.iter().enumerate() {
        ca.add_k(k);
        pb.inc(1);
        if step_index < 1 {
            continue;
        }
        if step_index >= klines.len() - 1000 {
            break;
        }
        let step = &dump[step_index - 1];
        test_fx_ctx(step.as_slice());

        if step[0].klc.is_none() {
            continue;
        }

        cmp_candle_list(step_index, step.as_slice(), ca.candle_list());
    }
    pb.finish_with_message("done");
    let duration = start_time.elapsed();
    println!(
        "parse time:{}s\ncandle_list count:{}",
        duration.as_secs(),
        ca.candle_list().len()
    );
}
