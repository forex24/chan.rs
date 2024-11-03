use crate::csv_util::read_kline_from_csv;
use crate::util::{bi_to_point, seg_to_point};
use crate::Opt;
use std::path::PathBuf;

use czsc::Analyzer;
use plot::TVPlot;

pub async fn parse(opt: &Opt) {
    let mut fname = opt.input.clone().unwrap_or(PathBuf::from(&opt.symbol));
    fname.set_extension("csv");
    let klines = read_kline_from_csv(&fname);
    let mut ca = Analyzer::default();

    let klines = &klines[0..];
    for k in klines {
        ca.add_k(k)
    }

    let (sure_bi_points, unsure_bi_points) = bi_to_point(ca.bi_list());

    let (sure_seg_points, unsure_seg_points) = seg_to_point(ca.seg_list());

    let (sure_seg_seg_points, unsure_seg_seg_points) = seg_to_point(ca.seg_seg_list());

    TVPlot::new(&opt.symbol)
        .add_bar_series(klines)
        .add_line_series(sure_bi_points.as_slice(), "sure_bi", 0, 1, "#FFBF00")
        .add_line_series(unsure_bi_points.as_slice(), "unsure_bi", 2, 1, "#FFBF00")
        .add_line_series(sure_seg_points.as_slice(), "sure_seg", 0, 1, "#6495ED")
        .add_line_series(
            unsure_seg_points.as_slice(),
            "unsure_seg_seg",
            2,
            1,
            "#6495ED",
        )
        .add_line_series(
            sure_seg_seg_points.as_slice(),
            "sure_seg_seg",
            0,
            1,
            "#FF0000",
        )
        .add_line_series(
            unsure_seg_seg_points.as_slice(),
            "unsure_seg",
            2,
            1,
            "#FF0000",
        )
        .display();
}
