use crate::Bi::Bi::CBi;
use crate::Common::types::Handle;
use crate::Common::CEnum::{BiDir, TrendLineSide};
use std::f64;

#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: usize,
    pub y: f64,
}

impl Point {
    pub fn new(x: usize, y: f64) -> Self {
        Point { x, y }
    }

    pub fn cal_slope(&self, p: &Point) -> f64 {
        if self.x != p.x {
            (self.y - p.y) / (self.x - p.x) as f64
        } else {
            f64::INFINITY
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Line {
    pub p: Point,
    pub slope: f64,
}

impl Line {
    pub fn new(p: Point, slope: f64) -> Self {
        Line { p, slope }
    }

    pub fn cal_dis(&self, p: &Point) -> f64 {
        (self.slope * p.x as f64 - p.y + self.p.y - self.slope * self.p.x as f64).abs()
            / (self.slope.powi(2) + 1.0).sqrt()
    }
}

pub struct CTrendLine {
    pub line: Option<Line>,
    pub side: TrendLineSide,
}

impl CTrendLine {
    pub fn new(lst: &[crate::Common::types::Handle<CBi>], side: TrendLineSide) -> Self {
        let mut trend_line = CTrendLine { line: None, side };
        trend_line.cal(lst);
        trend_line
    }

    pub fn cal(&mut self, lst: &[Handle<CBi>]) {
        let mut bench = f64::INFINITY;
        let all_p = if self.side == TrendLineSide::Inside {
            lst.iter()
                .rev()
                .step_by(2)
                .map(|bi| {
                    Point::new(
                        bi.borrow().get_begin_klu().borrow().idx,
                        bi.borrow().get_begin_val(),
                    )
                })
                .collect::<Vec<_>>()
        } else {
            lst.iter()
                .rev()
                .step_by(2)
                .map(|bi| {
                    Point::new(
                        bi.borrow().get_end_klu().borrow().idx,
                        bi.borrow().get_end_val(),
                    )
                })
                .collect::<Vec<_>>()
        };
        let mut c_p = all_p.clone();
        while !c_p.is_empty() {
            let (line, idx) = cal_tl(&c_p, lst.last().unwrap().borrow().dir, self.side);
            let dis: f64 = all_p.iter().map(|p| line.cal_dis(p)).sum();
            if dis < bench {
                bench = dis;
                self.line = Some(line);
            }
            c_p = c_p[idx..].to_vec();
            if c_p.len() == 1 {
                break;
            }
        }
    }
}

fn init_peak_slope(dir: BiDir, side: TrendLineSide) -> f64 {
    match (side, dir) {
        (TrendLineSide::Inside, _) => 0.0,
        (_, BiDir::Up) => f64::INFINITY,
        (_, BiDir::Down) => f64::NEG_INFINITY,
    }
}

fn cal_tl(c_p: &[Point], dir: BiDir, side: TrendLineSide) -> (Line, usize) {
    let p = c_p[0];
    let mut peak_slope = init_peak_slope(dir, side);
    let mut idx = 1;
    for (point_idx, p2) in c_p[1..].iter().enumerate() {
        let slope = p.cal_slope(p2);
        if (dir == BiDir::Up && slope < 0.0) || (dir == BiDir::Down && slope > 0.0) {
            continue;
        }
        match (side, dir) {
            (TrendLineSide::Inside, BiDir::Up) if slope > peak_slope => {
                peak_slope = slope;
                idx = point_idx + 1;
            }
            (TrendLineSide::Inside, BiDir::Down) if slope < peak_slope => {
                peak_slope = slope;
                idx = point_idx + 1;
            }
            (TrendLineSide::Outside, BiDir::Up) if slope < peak_slope => {
                peak_slope = slope;
                idx = point_idx + 1;
            }
            (TrendLineSide::Outside, BiDir::Down) if slope > peak_slope => {
                peak_slope = slope;
                idx = point_idx + 1;
            }
            _ => {}
        }
    }
    (Line::new(p, peak_slope), idx)
}
