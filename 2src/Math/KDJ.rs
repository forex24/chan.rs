use std::collections::VecDeque;

#[derive(Clone, Copy, Debug)]
pub struct KDJItem {
    pub k: f64,
    pub d: f64,
    pub j: f64,
}

impl KDJItem {
    pub fn new(k: f64, d: f64, j: f64) -> Self {
        KDJItem { k, d, j }
    }
}

#[derive(Debug)]
pub struct KDJ {
    arr: VecDeque<KDJData>,
    period: usize,
    pre_kdj: KDJItem,
}

#[derive(Debug)]
pub struct KDJData {
    high: f64,
    low: f64,
}

impl KDJ {
    pub fn new(period: usize) -> Self {
        KDJ {
            arr: VecDeque::with_capacity(period),
            period,
            pre_kdj: KDJItem::new(50.0, 50.0, 50.0),
        }
    }

    pub fn add(&mut self, high: f64, low: f64, close: f64) -> KDJItem {
        self.arr.push_back(KDJData { high, low });
        if self.arr.len() > self.period {
            self.arr.pop_front();
        }

        let hn = self
            .arr
            .iter()
            .map(|x| x.high)
            .fold(f64::NEG_INFINITY, f64::max);
        let ln = self.arr.iter().map(|x| x.low).fold(f64::INFINITY, f64::min);
        let cn = close;
        let rsv = if hn != ln {
            100.0 * (cn - ln) / (hn - ln)
        } else {
            0.0
        };

        let cur_k = 2.0 / 3.0 * self.pre_kdj.k + 1.0 / 3.0 * rsv;
        let cur_d = 2.0 / 3.0 * self.pre_kdj.d + 1.0 / 3.0 * cur_k;
        let cur_j = 3.0 * cur_k - 2.0 * cur_d;
        let cur_kdj = KDJItem::new(cur_k, cur_d, cur_j);
        self.pre_kdj = cur_kdj;

        cur_kdj
    }
}
