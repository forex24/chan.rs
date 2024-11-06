use crate::csv_util::read_kline_from_csv;
use crate::Opt;
use czsc::CBspPoint;
use czsc::{Analyzer, CBi};
use dump::{read_bsp_dump_file, BspContext, SBsp};
use indicatif::ProgressBar;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Instant;

use czsc::Kline;

pub async fn parse(opt: &Opt) {
    let mut fname = opt.input.clone().unwrap_or(PathBuf::from(&opt.symbol));
    fname.set_extension("csv");
    let klines = read_kline_from_csv(&fname);

    let mut fname = opt.input.clone().unwrap_or(PathBuf::from(&opt.symbol));
    fname.set_extension("bsp.gz");
    let dump = read_bsp_dump_file(fname.as_path()).unwrap();
    czsc_parse(&klines, &dump)
}

fn cmp_bsp(sbsp: &SBsp, bsp: &Rc<RefCell<CBspPoint<CBi>>>) -> bool {
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

fn cmp_bsp_list(
    _step_index: usize,
    ctx: &[BspContext],
    bsp_list: &[Rc<RefCell<CBspPoint<CBi>>>],
    _ca: &Analyzer,
) -> (bool, usize) {
    //let bsp_list = bsp_list.iter().filter(|x| x.is_valid).collect::<Vec<_>>();
    //println!("ctx len:{} seg_list:{}", ctx.len(), seg_list.len());
    //println!("ctx:{:?}", ctx);
    // if ctx.len() != bsp_list.len() {
    //     println!(
    //         "ctx_len:{} bsp_list:{} step:{} ctx_step:{},\ndump bsp: {}\n ana bsp:{:?}",
    //         ctx.len(),
    //         bsp_list.len(),
    //         _step_index,
    //         ctx[0].step_index,
    //         ctx.last().unwrap().bsp.as_ref().unwrap(),
    //         bsp_list.last()
    //     );
    // }

    //for sctx in ctx.iter() {
    //    if let Some(ref sbsp) = sctx.bsp {
    //        println!("sbsp: {}", sbsp);
    //    } else {
    //        println!("sbsp: None")
    //    }
    //}
    //
    //for bsp in bsp_list.iter() {
    //    println!(" bsp: {} index:{}", bsp, bsp.index());
    //}

    //assert!(ctx.len() == bsp_list.len());
    for (bsp_index, c) in ctx.iter().enumerate() {
        let bsp = &bsp_list[bsp_index];

        let sbsp = c.bsp.as_ref().unwrap();
        if !cmp_bsp(sbsp, bsp) {
            println!(
                "step:{} ctx_step:{}, bsp_index:{}\ndump bsp: {}\n ana bsp: {}",
                _step_index,
                ctx[0].step_index,
                bsp_index,
                sbsp,
                bsp.borrow()
            );

            // println!(
            //     "cmp:{} bi:{} klu:{} is_buy:{} is_segbsp:{} types:{} rbsp1:{} {:?}",
            //     cmp_bsp(sbsp, bsp),
            //     sbsp.bi == bsp.bi.index(),
            //     sbsp.klu == bsp.klu.index(),
            //     sbsp.is_buy == bsp.is_buy,
            //     sbsp.is_segbsp == bsp.is_segbsp,
            //     sbsp.types == bsp.type2str().trim(),
            //     sbsp.relate_bsp1 == bsp.relate_bsp1.map(|x| x.index()),
            //     bsp.relate_bsp1.map(|x| x.bi.index())
            // );
            return (false, bsp_index);
        }
    }
    (true, 0)
}

fn test_bsp_ctx(ctx: &[BspContext]) {
    let index = ctx[0].step_index;
    for c in ctx {
        assert!(c.step_index == index);
    }
}

fn print_all(ctx: &[BspContext], bsp_list: &[Rc<RefCell<CBspPoint<CBi>>>], start_index: usize) {
    //let bsp_list = bsp_list.iter().filter(|x| x.is_valid).collect::<Vec<_>>();
    for (index, sctx) in ctx.iter().enumerate().skip(start_index) {
        if let Some(ref sbsp) = sctx.bsp {
            println!("[{}]sbsp: {}", index, sbsp);
        } else {
            println!("[{}]sbsp: None", index)
        }
    }

    println!("\n");

    for (index, bsp) in bsp_list.iter().enumerate().skip(start_index) {
        println!("[{}] bsp: {} ", index, bsp.borrow());
    }
}
fn czsc_parse(klines: &[Kline], dump: &[Vec<BspContext>]) {
    let mut ca = Analyzer::default();
    println!("klines len:{} dump len:{}", klines.len(), dump.len());
    let pb = ProgressBar::new(klines.len() as u64);
    let start_time = Instant::now();
    //assert!(klines.len() == dump.len());
    for (step_index, k) in klines.iter().enumerate() {
        ca.add_k(k);
        pb.inc(1);
        if step_index < 2 {
            continue;
        }
        let step = &dump[step_index - 1];
        test_bsp_ctx(step.as_slice());

        if step[0].bsp.is_none() {
            continue;
        }

        let (is_ok, start_index) = cmp_bsp_list(step_index, step.as_slice(), ca.bi_bsp_list(), &ca);
        if !is_ok {
            println!("\n");
            let start_index = if start_index >= 1 {
                start_index - 1
            } else {
                start_index
            };
            print_all(step.as_slice(), ca.bi_bsp_list(), start_index);
            break;
        }
    }
    pb.finish_with_message("done");
    let duration = start_time.elapsed();
    println!(
        "parse time:{}s\nbsp count:{}",
        duration.as_secs(),
        ca.bi_bsp_list().len()
    );
}
