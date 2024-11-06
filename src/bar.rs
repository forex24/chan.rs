use std::fmt::Display;

use chrono::{DateTime, Utc};

use crate::{AsHandle, Candle, Handle, ICandlestick, IHighLow, MetricModel};

// 原始K线
#[derive(Debug, Clone)]
pub struct Bar {
    handle: Handle<Self>,
    //pub kl_type:Duration, //周期
    pub time: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub sub_kl_list: Vec<Handle<Bar>>, // 次级别KLU列表
    pub sup_kl: Option<Handle<Bar>>,   // 指向更高级别KLU
    pub klc: Option<Handle<Candle>>,   // 指向KLine

                                       // indicator
                                       //pub macd: Option<Handle<CMACDItem>>,
                                       //pub boll: Option<BollMetric>,
                                       //pub trade_info: CTradeInfo,
                                       //pub demark: CDemarkIndex,
                                       //pub trend: HashMap<TrendType, HashMap<i32, f64>>,
                                       //pub limit_flag: i32,  // 0:普通 -1:跌停，1:涨停
                                       //pub kdj: Option<KDJItem>,
}

impl Display for Bar {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "index {} time:{} open:{} high:{} low:{} close:{}",
            self.as_handle().index(),
            self.time,
            self.open,
            self.high,
            self.low,
            self.close,
        )
    }
}

impl Bar {
    #[allow(clippy::borrowed_box)]
    pub fn new(
        box_vec: &Box<Vec<Self>>,
        index: usize,
        time: DateTime<Utc>,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
    ) -> Self {
        let mut bar = Self {
            handle: Handle::new(box_vec, index),
            time,
            close,
            open,
            high,
            low,
            sub_kl_list: Vec::new(),
            sup_kl: None,
            klc: None,
            //macd: None,
            //boll: None,
            //trade_info: CTradeInfo::new(kl_dict),
            //demark: CDemarkIndex::new(),
            //trend: HashMap::new(),
            //limit_flag: 0,
            //rsi: None,
            //kdj: None,
        };
        bar.check(false);
        bar
    }

    pub fn check(&mut self, autofix: bool) {
        if self.low > self.low.min(self.open.min(self.high.min(self.close))) {
            if autofix {
                self.low = self.low.min(self.open.min(self.high.min(self.close)));
            } else {
                panic!(
                    "{} low price={} is not min of [low={}, open={}, high={}, close={}]",
                    self.time, self.low, self.low, self.open, self.high, self.close
                );
            }
        }
        if self.high < self.low.max(self.open.max(self.high.max(self.close))) {
            if autofix {
                self.high = self.low.max(self.open.max(self.high.max(self.close)));
            } else {
                panic!(
                    "{} high price={} is not max of [low={}, open={}, high={}, close={}]",
                    self.time, self.high, self.low, self.open, self.high, self.close
                );
            }
        }
    }

    pub fn set_klc(&mut self, klc: &Candle) {
        self.klc = Some(klc.as_handle())
    }

    pub fn add_children(&mut self, child: Handle<Bar>) {
        self.sub_kl_list.push(child);
    }

    pub fn set_parent(&mut self, parent: Handle<Bar>) {
        self.sup_kl = Some(parent);
    }

    pub fn get_children(&self) -> impl Iterator<Item = &Handle<Bar>> {
        self.sub_kl_list.iter()
    }

    //pub fn set_metric(&mut self, metric_model_lst: &mut Vec<MetricModel>) {
    //    for metric_model in metric_model_lst {
    //        match metric_model {
    //            MetricModel::MACD(ref mut cmacd) => {
    //                self.macd = Some(cmacd.add(self.close));
    //            }
    //            MetricModel::BOLL(ref mut boll_model) => {
    //                self.boll = Some(boll_model.add(self.close));
    //            }
    //        }
    //    }
    //}

    pub fn get_parent_klc(&self) -> Option<Handle<Candle>> {
        self.sup_kl.as_ref().and_then(|sup_kl| sup_kl.klc)
    }

    //pub fn set_metric(&mut self, metric_model_lst: &mut [Box<dyn MetricModel>]) {
    //    for metric_model in metric_model_lst {
    //        metric_model.update_bar(self);
    //    }
    //}
}

impl IHighLow for Bar {
    fn high(&self) -> f64 {
        self.high
    }

    fn low(&self) -> f64 {
        self.low
    }
}

impl ICandlestick for Bar {
    fn unix_time(&self) -> i64 {
        self.time.timestamp_millis()
    }

    fn open(&self) -> f64 {
        self.open
    }

    fn close(&self) -> f64 {
        self.close
    }
}

impl_handle!(Bar);
