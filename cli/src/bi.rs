use crate::csv_util::read_kline_from_csv;
use crate::Opt;
use czsc::{Analyzer, AsHandle, CBi, ICandlestick};
use dump::context::{BiContext, SBi};
use dump::dump_bi::read_bi_dump_file;
use indicatif::ProgressBar;
use plot::TVPlot;
use std::path::PathBuf;
use std::time::Instant;

use czsc::Indexable;
use czsc::Kline;

pub async fn parse(opt: &Opt) {
    let mut fname = opt.input.clone().unwrap_or(PathBuf::from(&opt.symbol));
    fname.set_extension("csv");
    let klines = read_kline_from_csv(&fname);

    let mut fname = opt.input.clone().unwrap_or(PathBuf::from(&opt.symbol));
    fname.set_extension("bi.gz");
    let dump = read_bi_dump_file(fname.as_path()).unwrap();
    czsc_parse(&klines, &dump)
}

fn _cmp_bsp(sbi: &SBi, cbi: &CBi) -> bool {
    match (sbi.bsp.as_ref(), cbi.bsp.as_ref()) {
        (None, None) => true,
        (Some(sbsp), Some(bsp)) => {
            (sbsp.bi == bsp.borrow().bi.index())
                && (sbsp.klu == bsp.borrow().klu.index())
                && (sbsp.is_buy == bsp.borrow().is_buy)
                && (sbsp.is_segbsp == bsp.borrow().is_segbsp)
                && (sbsp.types == bsp.borrow().type2str().trim())
                && (sbsp.relate_bsp1
                    == bsp
                        .borrow()
                        .relate_bsp1
                        .as_ref()
                        .map(|x| x.borrow().bi.index()))
        }
        _ => false,
    }
}

fn _cmp_parent_seg(sbi: &SBi, cbi: &CBi) -> bool {
    if let Some(ref parent_seg) = sbi.parent_seg {
        if cbi.parent_seg_idx.is_none() {
            return false;
        }
        return (parent_seg.seg_index == cbi.parent_seg_idx.unwrap())
            && (parent_seg.direction == cbi.parent_seg_dir.unwrap());
    }
    if sbi.parent_seg.is_none() && cbi.parent_seg_idx.is_some() {
        return false;
    }
    true
}

fn cmp_bi_base(sbi: &SBi, bi: &CBi) -> bool {
    (sbi.bi_index == bi.index())
        && (sbi.direction == bi.as_handle().dir)
        && (sbi.is_sure == bi.is_sure)
        && (sbi.sure_end == bi.sure_end.map(|x| x.index()))
        && (sbi.begin_candle == bi.begin_klc.index())
        && (sbi.end_candle == bi.end_klc.index())
        && (sbi.seg_index == bi.seg_idx)
}

fn cmp_bi(sbi: &SBi, cbi: &CBi) -> bool {
    cmp_bi_base(sbi, cbi) && _cmp_parent_seg(sbi, cbi) //&& _cmp_bsp(sbi, cbi)
}

fn cmp_vec(_step_index: usize, ctx: &[BiContext], bi_list: &[Box<CBi>], ca: &Analyzer) -> bool {
    //println!("ctx len:{} bi_list:{}", ctx.len(), bi_list.len());
    //println!("ctx:{:?}", ctx);
    if ctx.len() != bi_list.len() {
        println!(
            "step:{} ctx_step:{},\ndump bi: {}\n ana bi:{:?}",
            _step_index,
            ctx[0].step_index,
            ctx.last().unwrap().bi.as_ref().unwrap(),
            bi_list.last()
        );

        let begin = bi_list.last().unwrap().begin_klc.index();
        let end = bi_list.last().unwrap().end_klc.index();

        (begin..=end).for_each(|k| println!("{}", ca.candle_list()[k]));
        let begin_bar = bi_list.last().unwrap().begin_klc.lst[0].index();
        let end_bar = bi_list.last().unwrap().end_klc.lst.last().unwrap().index();
        let plot_klines = &ca.bar_list()[begin_bar..=end_bar];
        //plot("bar", plot_klines);
        //plot("candle", &ca.candle_list()[begin..=end]);
    }
    assert!(ctx.len() == bi_list.len());
    for (bi_index, c) in ctx.iter().enumerate() {
        let bi = &bi_list[bi_index];
        let sbi = c.bi.as_ref().unwrap();
        if !cmp_bi(sbi, bi) {
            println!(
                "step:{} ctx_step:{},\ndump bi: {}\n ana bi:{}",
                _step_index, ctx[0].step_index, sbi, bi
            );
            return false;
        }
    }
    true
}

fn test_bi_ctx(ctx: &[BiContext]) {
    let index = ctx[0].step_index;
    for c in ctx {
        assert!(c.step_index == index);
    }
}

fn czsc_parse(klines: &[Kline], dump: &[Vec<BiContext>]) {
    let mut ca = Analyzer::default();
    println!("klines len:{} dump len:{}", klines.len(), dump.len());
    let pb = ProgressBar::new(klines.len() as u64);
    let start_time = Instant::now();
    //assert!(klines.len() == dump.len());
    for (step_index, k) in klines.iter().enumerate() {
        //seg.add_k(k);
        // test candle equal
        ca.add_k(k);
        pb.inc(1);
        //println!("{}", step_index);
        if step_index < 2 {
            continue;
        }
        let step = &dump[step_index - 1];
        test_bi_ctx(step.as_slice());

        if step[0].bi.is_none() {
            continue;
        }

        cmp_vec(step_index, step.as_slice(), ca.bi_list(), &ca);
    }
    pb.finish_with_message("done");
    let duration = start_time.elapsed();
    println!(
        "parse time:{}s\nbi count:{}",
        duration.as_secs(),
        ca.bi_list().len()
    );

    /*println!(
        "candle count:{} {}",
        seg.candle_series.len(),
        ca.ctx.candles.len()
    );*/
    //println!("bar count:{}", seg.candle_series.bar_series.len());
    //println!("kline count:{}", klines.len());

    //let min5k: Vec<Kline> = klines.chunks(5).map(|x| merge_k(x)).collect();
    //display_seg_chart(&min5k, &seg)*/
}

fn plot<T: ICandlestick>(symbol: &str, klines: &[T]) {
    TVPlot::new(symbol).add_bar_series(klines).display();
}
