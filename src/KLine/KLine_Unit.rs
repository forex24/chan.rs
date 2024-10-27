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

pub struct KDJ {
    arr: VecDeque<KDJData>,
    period: usize,
    pre_kdj: KDJItem,
}

struct KDJData {
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

impl Clone for CKLine_Unit {
    fn clone(&self) -> Self {
        let mut kl_dict = HashMap::new();
        kl_dict.insert(DATA_FIELD::FIELD_TIME, self.time.clone().into());
        kl_dict.insert(DATA_FIELD::FIELD_CLOSE, self.close);
        kl_dict.insert(DATA_FIELD::FIELD_OPEN, self.open);
        kl_dict.insert(DATA_FIELD::FIELD_HIGH, self.high);
        kl_dict.insert(DATA_FIELD::FIELD_LOW, self.low);

        for metric in TRADE_INFO_LST.iter() {
            if let Some(value) = self.trade_info.metric.get(metric) {
                kl_dict.insert(*metric, *value);
            }
        }

        let mut obj = CKLine_Unit::new(&kl_dict, false).unwrap();
        {
            let mut obj = obj.borrow_mut();
            obj.demark = self.demark.clone();
            obj.trend = self.trend.clone();
            obj.limit_flag = self.limit_flag;
            obj.macd = self.macd.clone();
            obj.boll = self.boll.clone();
            obj.rsi = self.rsi.clone();
            obj.kdj = self.kdj.clone();
            obj.set_idx(self.idx);
        }

        Rc::try_unwrap(obj).unwrap().into_inner()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kdj() {
        let mut kdj = KDJ::new(9);
        let data = vec![
            (10.0, 8.0, 9.0),
            (11.0, 9.0, 10.0),
            (12.0, 10.0, 11.0),
            (13.0, 11.0, 12.0),
            (14.0, 12.0, 13.0),
        ];

        for (high, low, close) in data {
            let result = kdj.add(high, low, close);
            println!(
                "KDJ: K={:.2}, D={:.2}, J={:.2}",
                result.k, result.d, result.j
            );
        }

        let last_result = kdj.add(15.0, 13.0, 14.0);
        assert!((last_result.k - 77.78).abs() < 0.01);
        assert!((last_result.d - 68.52).abs() < 0.01);
        assert!((last_result.j - 96.30).abs() < 0.01);
    }
}
