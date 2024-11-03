use crate::csv_util::read_kline_from_csv;
use crate::Opt;
use czsc::{Analyzer, CBi, CSeg, CZs};
use dump::context::{SSeg, SegContext};
use dump::dump_seg::read_seg_dump_file;
use dump::SZs;
use std::path::PathBuf;
use std::time::Instant;

use czsc::Indexable;
use czsc::Kline;
use indicatif::ProgressBar;

pub async fn parse(opt: &Opt) {
    let mut fname = opt.input.clone().unwrap_or(PathBuf::from(&opt.symbol));
    fname.set_extension("csv");
    let klines = read_kline_from_csv(&fname);

    let mut fname = opt.input.clone().unwrap_or(PathBuf::from(&opt.symbol));
    fname.set_extension("seg.gz");
    let dump = read_seg_dump_file(fname.as_path()).unwrap();
    czsc_parse(&klines, &dump)
}

fn cmp_seg_base(sseg: &SSeg, seg: &CSeg<CBi>) -> bool {
    (sseg.seg_index == seg.index())
        && (sseg.direction == seg.dir)
        && (sseg.is_sure == seg.is_sure)
        && (sseg.begin_bi == seg.start_bi.index())
        && (sseg.end_bi == seg.end_bi.index())
        && (sseg.reason == seg.reason)
        && (sseg.ele_inside_is_sure == seg.ele_inside_is_sure)
}

fn _cmp_seg_bsp(sseg: &SSeg, seg: &CSeg<CBi>) -> bool {
    match (sseg.bsp.as_ref(), seg.bsp.as_ref()) {
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

fn cmp_seg_zs(sseg: &SSeg, seg: &CSeg<CBi>) -> bool {
    match (sseg.zs_lst.is_none(), seg.zs_lst.is_empty()) {
        (true, true) => return true,
        (true, false) => return false,
        (false, true) => return false,
        _ => {}
    }

    for index in 0..seg.zs_lst.len() {
        let zs_cmp = cmp_zs(&sseg.zs_lst.as_ref().unwrap()[index], &seg.zs_lst[index]);
        if !zs_cmp {
            return false;
        }
    }

    true
}

fn cmp_seg(sseg: &SSeg, seg: &CSeg<CBi>) -> bool {
    cmp_seg_base(sseg, seg) && cmp_seg_zs(sseg, seg) && _cmp_seg_bsp(sseg, seg)
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
    //&& (szs.bi_in == zs.bi_in.map(|x| x.index()))
    //&& (szs.bi_out == zs.bi_out.map(|x| x.index()))
}

fn cmp_seg_lst(
    _step_index: usize,
    ctx: &[SegContext],
    seg_list: &[Box<CSeg<CBi>>],
    ca: &Analyzer,
) -> bool {
    //println!("ctx len:{} seg_list:{}", ctx.len(), seg_list.len());
    //println!("ctx:{:?}", ctx);
    if ctx.len() != seg_list.len() {
        println!(
            "step:{} ctx_step:{},\ndump seg: {}\n ana seg:{}",
            _step_index,
            ctx[0].step_index,
            ctx.last().unwrap().seg.as_ref().unwrap(),
            seg_list.last().unwrap()
        );

        let begin = seg_list.last().unwrap().start_bi.index();
        let end = seg_list.last().unwrap().end_bi.index();

        (begin..=end).for_each(|k| println!("{}", ca.bi_list()[k]));
    }
    assert!(ctx.len() == seg_list.len());
    for (seg_index, c) in ctx.iter().enumerate() {
        let seg = &seg_list[seg_index];
        let sseg = c.seg.as_ref().unwrap();
        if !cmp_seg(sseg, seg) {
            println!(
                "step:{} ctx_step:{},\ndump seg: {}\n ana seg: {}\n",
                _step_index, ctx[0].step_index, sseg, seg
            );

            if let Some(ref bsp) = sseg.bsp {
                println!("sbsp:{}", bsp);
            }
            if let Some(ref bsp) = seg.bsp {
                println!(" bsp:{}\n", bsp.borrow());
            }

            if let Some(ref zs_list) = sseg.zs_lst {
                for zs in zs_list.iter() {
                    println!("szs:{}", zs);
                }
            }
            for zs in &seg.zs_lst {
                println!(" zs:{}", zs.to_ref());
            }
            return false;
        }
    }
    true
}

fn test_seg_ctx(ctx: &[SegContext]) {
    let index = ctx[0].step_index;
    for c in ctx {
        assert!(c.step_index == index);
    }
}

fn czsc_parse(klines: &[Kline], dump: &[Vec<SegContext>]) {
    //let config = Config::default();
    //let mut seg = BiSeries::new(config.bi_config); // CSegSeries::new(config.seg_config);
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
        if step_index < 2 {
            continue;
        }
        let step = &dump[step_index - 1];
        test_seg_ctx(step.as_slice());

        if step[0].seg.is_none() {
            continue;
        }

        if !cmp_seg_lst(step_index, step.as_slice(), ca.seg_list(), &ca) {
            //println!("compare seg list failed");
            break;
        }
    }
    pb.finish_with_message("done");
    let duration = start_time.elapsed();
    println!(
        "parse time:{}s\nseg count:{}",
        duration.as_secs(),
        ca.seg_list().len()
    );
}
