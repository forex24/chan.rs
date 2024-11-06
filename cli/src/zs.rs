use crate::csv_util::read_kline_from_csv;
use crate::Opt;
use czsc::CZs;
use czsc::{Analyzer, CBi};
use dump::SZs;
use dump::{read_zs_dump_file, ZsContext};
use indicatif::ProgressBar;
use std::path::PathBuf;
use std::time::Instant;

use czsc::Kline;

pub async fn parse(opt: &Opt) {
    let mut fname = opt.input.clone().unwrap_or(PathBuf::from(&opt.symbol));
    fname.set_extension("csv");
    let klines = read_kline_from_csv(&fname);

    let mut fname = opt.input.clone().unwrap_or(PathBuf::from(&opt.symbol));
    fname.set_extension("zs.gz");
    let dump = read_zs_dump_file(fname.as_path()).unwrap();
    czsc_parse(&klines, &dump)
}

fn cmp_zs(szs: &SZs, zs: &CZs<CBi>) -> bool {
    (szs.is_sure == zs.is_sure)
        && (szs.begin == zs.begin.index())
        && (szs.begin_bi == zs.begin_bi.index())
        && (szs.end == zs.end.unwrap().index())
        && (szs.end_bi == zs.end_bi.unwrap().index())
        && (szs.high == zs.high)
        && (szs.low == zs.low)
        && (szs.peak_high == zs.peak_high)
        && (szs.peak_low == zs.peak_low)
        && (szs.bi_in == zs.bi_in.map(|x| x.index()))
        && (szs.bi_out == zs.bi_out.map(|x| x.index()))
}

fn cmp_vec(
    _step_index: usize,
    ctx: &[ZsContext],
    zs_list: &[Box<CZs<CBi>>],
    ca: &Analyzer,
) -> bool {
    //println!("ctx len:{} seg_list:{}", ctx.len(), seg_list.len());
    //println!("ctx:{:?}", ctx);
    if ctx.len() != zs_list.len() {
        println!(
            "step:{} ctx_step:{},\ndump seg: {}\n ana seg:{}",
            _step_index,
            ctx[0].step_index,
            ctx.last().unwrap().zs.as_ref().unwrap(),
            zs_list.last().unwrap()
        );

        let begin = zs_list.last().unwrap().begin_bi.index();
        let end = zs_list.last().unwrap().end_bi.unwrap().index();

        (begin..=end).for_each(|k| println!("{}", ca.bi_list()[k]));
    }
    assert!(ctx.len() == zs_list.len());
    for (seg_index, c) in ctx.iter().enumerate() {
        let zs = &zs_list[seg_index];
        let szs = c.zs.as_ref().unwrap();
        if !cmp_zs(szs, zs) {
            //println!(
            //    "step:{} ctx_step:{},\ndump seg: {:?}\n ana seg:{}",
            //    step_index, ctx[0].step_index, sseg, seg
            //);
            return false;
        }
    }
    true
}

fn test_zs_ctx(ctx: &[ZsContext]) {
    let index = ctx[0].step_index;
    for c in ctx {
        assert!(c.step_index == index);
    }
}

fn czsc_parse(klines: &[Kline], dump: &[Vec<ZsContext>]) {
    //let config = Config::default();
    //let mut seg = BiSeries::new(config.bi_config); // CSegSeries::new(config.seg_config);
    let mut ca = Analyzer::default();
    println!("klines len:{} dump len:{}", klines.len(), dump.len());
    //assert!(klines.len() == dump.len());
    let pb = ProgressBar::new(klines.len() as u64);
    let start_time = Instant::now();
    for (step_index, k) in klines.iter().enumerate() {
        //seg.add_k(k);
        // test candle equal
        ca.add_k(k);
        pb.inc(1);
        if step_index < 2 {
            continue;
        }
        let step = &dump[step_index - 1];
        test_zs_ctx(step.as_slice());

        if step[0].zs.is_none() {
            continue;
        }

        cmp_vec(step_index, step.as_slice(), ca.bi_zs_list(), &ca);
    }
    pb.finish_with_message("done");
    let duration = start_time.elapsed();
    println!(
        "parse time:{}s\nzs count:{}",
        duration.as_secs(),
        ca.bi_zs_list().len()
    );
}
