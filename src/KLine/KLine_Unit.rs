use crate::{
    Common::{
        types::Handle,
        CEnum::{DataField, TrendType},
        CTime::CTime,
        ChanException::{CChanException, ErrCode},
        //TradeInfo::CTradeInfo,
    },
    //Math::{
    //    Demark::{CDemarkEngine, CDemarkIndex},
    //    TrendModel::CTrendModel,
    //    BOLL::{BOLLMetric, BollModel},
    //    KDJ::KDJ,
    //    MACD::{CMACDItem, CMACD},
    //    RSI::RSI,
    //},
};

use super::KLine::CKLine;

pub struct CKLineUnit {
    pub kl_type: Option<String>,
    pub time: CTime,
    pub close: f64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    //pub trade_info: CTradeInfo,
    //pub demark: CDemarkIndex,
    //pub sub_kl_list: Vec<Handle<CKLineUnit>>,
    //pub sup_kl: Option<Handle<CKLineUnit>>,
    pub klc: Option<Handle<CKLine>>,
    //pub trend: HashMap<TrendType, HashMap<i32, f64>>,
    //pub limit_flag: i32,
    pub pre: Option<Handle<CKLineUnit>>,
    pub next: Option<Handle<CKLineUnit>>,
    pub idx: i32,
    //pub macd: Option<CMACDItem>,
    //pub boll: Option<BOLLMetric>,
    //pub rsi: Option<f64>,
    //pub kdj: Option<KDJ>,
}

impl CKLineUnit {
    pub fn new(
        time: CTime,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        autofix: bool,
    ) -> Result<Self, CChanException> {
        let mut unit = CKLineUnit {
            kl_type: None,
            time,  //: CTime::from_f64(kl_dict[&DataField::FieldTime])?,
            close, //: kl_dict[&DataField::FieldClose],
            open,  //: kl_dict[&DataField::FieldOpen],
            high,  //: kl_dict[&DataField::FieldHigh],
            low,   //: kl_dict[&DataField::FieldLow],
            //trade_info: CTradeInfo::new(kl_dict),
            //demark: CDemarkIndex::new(),
            //sub_kl_list: Vec::new(),
            //sup_kl: None,
            klc: None,
            //trend: HashMap::new(),
            //limit_flag: 0,
            pre: None,
            next: None,
            idx: -1,
            //macd: None,
            //boll: None,
            //rsi: None,
            //kdj: None,
        };

        //unit.check(autofix)?;
        Ok(unit)
    }

    pub fn check(&mut self, autofix: bool) -> Result<(), CChanException> {
        let min_price = self.low.min(self.open).min(self.high).min(self.close);
        if self.low > min_price {
            if autofix {
                self.low = min_price;
            } else {
                return Err(CChanException::new(
                    format!(
                        "{} low price={} is not min of [low={}, open={}, high={}, close={}]",
                        self.time, self.low, self.low, self.open, self.high, self.close
                    ),
                    ErrCode::KlDataInvalid,
                ));
            }
        }

        let max_price = self.low.max(self.open).max(self.high).max(self.close);
        if self.high < max_price {
            if autofix {
                self.high = max_price;
            } else {
                return Err(CChanException::new(
                    format!(
                        "{} high price={} is not max of [low={}, open={}, high={}, close={}]",
                        self.time, self.high, self.low, self.open, self.high, self.close
                    ),
                    ErrCode::KlDataInvalid,
                ));
            }
        }

        Ok(())
    }

    /*pub fn add_children(&mut self, child: Handle<CKLineUnit>) {
        self.sub_kl_list.push(child);
    }

    pub fn set_parent(&mut self, parent: Handle<CKLineUnit>) {
        self.sup_kl = Some(parent);
    }

    pub fn get_children(&self) -> impl Iterator<Item = &Handle<CKLineUnit>> {
        self.sub_kl_list.iter()
    }*/

    pub fn low(&self) -> f64 {
        self.low
    }

    pub fn high(&self) -> f64 {
        self.high
    }

    /*pub fn set_metric(&mut self, metric_model_lst: &[Box<dyn MetricModel>]) {
        for metric_model in metric_model_lst {
            if let Some(macd) = metric_model.as_any().downcast_ref::<CMACD>() {
                self.macd = Some(macd.add(self.close));
            } else if let Some(trend_model) = metric_model.as_any().downcast_ref::<CTrendModel>() {
                self.trend
                    .entry(trend_model.get_type())
                    .or_insert_with(HashMap::new)
                    .insert(trend_model.get_t(), trend_model.add(self.close));
            } else if let Some(boll_model) = metric_model.as_any().downcast_ref::<BollModel>() {
                self.boll = Some(boll_model.add(self.close));
            } else if let Some(demark_engine) =
                metric_model.as_any().downcast_ref::<CDemarkEngine>()
            {
                self.demark = demark_engine.update(self.idx, self.close, self.high, self.low);
            } else if let Some(rsi) = metric_model.as_any().downcast_ref::<RSI>() {
                self.rsi = Some(rsi.add(self.close));
            } else if let Some(kdj) = metric_model.as_any().downcast_ref::<KDJ>() {
                self.kdj = Some(kdj.add(self.high, self.low, self.close));
            }
        }
    }

    pub fn get_parent_klc(&self) -> Option<Handle<CKLine>> {
        self.sup_kl
            .as_ref()
            .and_then(|sup_kl| sup_kl.borrow().klc.clone())
    }

    pub fn include_sub_lv_time(&self, sub_lv_t: &str) -> bool {
        if self.time.to_string() == sub_lv_t {
            return true;
        }
        for sub_klu in &self.sub_kl_list {
            let sub_klu = sub_klu.borrow();
            if sub_klu.time.to_string() == sub_lv_t || sub_klu.include_sub_lv_time(sub_lv_t) {
                return true;
            }
        }
        false
    }*/

    pub fn set_pre_klu(self_: Handle<CKLineUnit>, pre_klu: Handle<CKLineUnit>) {
        pre_klu.borrow_mut().next = Some(self_.clone());
        self_.borrow_mut().pre = Some(pre_klu);
    }

    pub fn set_klc(&mut self, klc: Handle<CKLine>) {
        self.klc = Some(klc);
    }

    pub fn get_klc(&self) -> Option<Handle<CKLine>> {
        self.klc.clone()
    }

    pub fn set_idx(&mut self, idx: i32) {
        self.idx = idx;
    }

    pub fn get_idx(&self) -> i32 {
        self.idx
    }
}

//`impl std::fmt::Display for CKLineUnit {
//`    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//`        write!(
//`            f,
//`            "{}:{}/{} open={} close={} high={} low={} {}",
//`            self.idx,
//`            self.time,
//`            self.kl_type.as_deref().unwrap_or(""),
//`            self.open,
//`            self.close,
//`            self.high,
//`            self.low,
//`            //self.trade_info
//`        )
//`    }
//`}

pub trait MetricModel {
    fn as_any(&self) -> &dyn std::any::Any;
}

/*
impl MetricModel for CMACD {}
impl MetricModel for CTrendModel {}
impl MetricModel for BollModel {}
impl MetricModel for CDemarkEngine {}
impl MetricModel for RSI {}
impl MetricModel for KDJ {}

impl Clone for CKLineUnit {
    fn clone(&self) -> Self {
        let mut kl_dict = HashMap::new();
        kl_dict.insert(DataField::FieldTime, self.time.to_f64());
        kl_dict.insert(DataField::FieldClose, self.close);
        kl_dict.insert(DataField::FieldOpen, self.open);
        kl_dict.insert(DataField::FieldHigh, self.high);
        kl_dict.insert(DataField::FieldLow, self.low);

        for metric in TradeInfoList::iter() {
            if let Some(value) = self.trade_info.metric.get(metric) {
                kl_dict.insert(*metric, *value);
            }
        }

        let mut obj = CKLineUnit::new(&kl_dict, false).unwrap();
        obj.demark = self.demark.clone();
        obj.trend = self.trend.clone();
        obj.limit_flag = self.limit_flag;
        obj.macd = self.macd.clone();
        obj.boll = self.boll.clone();
        obj.rsi = self.rsi;
        obj.kdj = self.kdj.clone();
        obj.set_idx(self.idx);
        obj
    }
}
*/
