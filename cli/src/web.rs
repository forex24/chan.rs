use crate::util::seg_to_point;
use crate::Opt;
use crate::{csv_util::read_kline_from_csv, util::bi_to_point};
use std::{path::PathBuf, sync::Arc};

use czsc::Analyzer;
use plot::TVPlot;

use axum::{extract::State, response::Html, routing::get, Router};

#[derive(Debug, Clone)]
struct AppState {
    option: Opt,
}

pub async fn parse(opt: &Opt) {
    let shared_state = Arc::new(AppState {
        option: opt.clone(),
    });
    let app = Router::new()
        .route("/", get(build_html))
        .with_state(shared_state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn build_html(State(state): State<Arc<AppState>>) -> Html<String> {
    let opt = state.option.clone();
    let mut fname = opt.input.clone().unwrap_or(PathBuf::from(&opt.symbol));
    fname.set_extension("csv");
    let klines = read_kline_from_csv(&fname);
    let mut ca = Analyzer::default();

    let klines = &klines[0..];
    for k in klines {
        ca.add_k(k)
    }

    let (sure_bi_points, unsure_bi_points) = bi_to_point(&ca.bi_list());

    let (sure_seg_points, unsure_seg_points) = seg_to_point(&ca.seg_list());

    let html = TVPlot::new(&opt.symbol)
        .add_bar_series(klines)
        .add_line_series(sure_bi_points.as_slice(), "sure_bi", 0, 1, "#FFBF00")
        .add_line_series(unsure_bi_points.as_slice(), "unsure_bi", 2, 1, "#FFBF00")
        .add_line_series(sure_seg_points.as_slice(), "sure_seg", 0, 1, "#6495ED")
        .add_line_series(unsure_seg_points.as_slice(), "unsure_seg", 2, 1, "#6495ED")
        .generate_html();
    Html(html)
}
