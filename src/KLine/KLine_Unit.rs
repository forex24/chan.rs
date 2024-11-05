use std::collections::HashMap;

use crate::{
    impl_handle,
    Common::{
        handle::{AsHandle, Handle, Indexable},
        CEnum::TrendType,
        CTime::CTime,
        ChanException::{CChanException, ErrCode}, //TradeInfo::CTradeInfo,
    },
    Math::{
        metric::MetricModel,
        //Demark::{CDemarkEngine, CDemarkIndex},
        //TrendModel::CTrendModel,
        BOLL::{BOLLMetric, BollModel},
        KDJ::KDJ,
        MACD::{CMACDItem, CMACD},
        RSI::RSI,
    },
};

use super::KLine::CKLine;

pub struct CKLineUnit {
    handle: Handle<Self>,
    // 基本属性
    pub kl_type: Option<String>,
    pub time: CTime,
    pub close: f64,
    pub open: f64,
    pub high: f64,
    pub low: f64,

    // 链表
    pub pre: Option<Handle<Self>>,
    pub next: Option<Handle<Self>>,

    // 上下文属性
    pub sub_kl_list: Vec<Handle<CKLineUnit>>,
    pub sup_kl: Option<Handle<CKLineUnit>>,
    pub klc: Option<Handle<CKLine>>,

    // FIXME: 用更好的模式来处理指标问题
    // 指标
    pub trend: HashMap<TrendType, HashMap<usize, f64>>, // CTrendModel
    //pub limit_flag: i32,
    //pub trade_info: CTradeInfo,
    //pub demark: CDemarkIndex,     // CDemarkEngine的CDemarkIndex
    pub macd: Option<CMACDItem>,  // CMACD
    pub boll: Option<BOLLMetric>, // BollModel的BOLL_Metric
    pub rsi: Option<f64>,         // RSI
    pub kdj: Option<KDJ>,         // KDJ
}

impl CKLineUnit {
    #[allow(clippy::borrowed_box)]
    pub fn new(
        time: CTime,
        box_vec: &Box<Vec<Self>>,
        index: usize,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        autofix: bool,
    ) -> Result<Self, CChanException> {
        let mut unit = CKLineUnit {
            handle: Handle::new(box_vec, index),
            kl_type: None,
            time,  //: CTime::from_f64(kl_dict[&DataField::FieldTime])?,
            close, //: kl_dict[&DataField::FieldClose],
            open,  //: kl_dict[&DataField::FieldOpen],
            high,  //: kl_dict[&DataField::FieldHigh],
            low,   //: kl_dict[&DataField::FieldLow],
            //trade_info: CTradeInfo::new(kl_dict),
            //demark: CDemarkIndex::new(),
            sub_kl_list: Vec::new(),
            sup_kl: None,
            klc: None,
            trend: HashMap::new(),
            //limit_flag: 0,
            pre: None,
            next: None,
            macd: None,
            boll: None,
            rsi: None,
            kdj: None,
        };

        unit.check(autofix)?;
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

    pub fn add_children(&mut self, child: Handle<CKLineUnit>) {
        self.sub_kl_list.push(child);
    }

    pub fn set_parent(&mut self, parent: Handle<CKLineUnit>) {
        self.sup_kl = Some(parent);
    }

    pub fn get_children(&self) -> impl Iterator<Item = &Handle<CKLineUnit>> {
        self.sub_kl_list.iter()
    }

    pub fn low(&self) -> f64 {
        self.low
    }

    pub fn high(&self) -> f64 {
        self.high
    }

    pub fn set_metric(&mut self, metric_model_lst: &mut Vec<MetricModel>) {
        for metric_model in metric_model_lst {
            match metric_model {
                MetricModel::MACD(ref mut cmacd) => {
                    self.macd = Some(cmacd.add(self.close));
                }
                MetricModel::BOLL(ref mut boll_model) => {
                    self.boll = Some(boll_model.add(self.close));
                }
            }
        }
    }

    //pub fn set_metric(&mut self, metric_model_lst: &mut [Box<dyn MetricModel>]) {
    //    for metric_model in metric_model_lst.iter_mut() {
    //        if let Some(macd) = metric_model.as_any_mut().downcast_mut::<CMACD>() {
    //            self.macd = Some(macd.add(self.close));
    //        } else if let Some(trend_model) =
    //            metric_model.as_any_mut().downcast_mut::<CTrendModel>()
    //        {
    //            self.trend
    //                .entry(trend_model.trend_type)
    //                .or_default()
    //                .insert(trend_model.t, trend_model.add(self.close));
    //        } else if let Some(boll_model) = metric_model.as_any_mut().downcast_mut::<BollModel>() {
    //            self.boll = Some(boll_model.add(self.close));
    //        } else if let Some(demark_engine) =
    //            metric_model.as_any_mut().downcast_mut::<CDemarkEngine>()
    //        {
    //            self.demark = demark_engine.update(self.index(), self.close, self.high, self.low);
    //        } else if let Some(rsi) = metric_model.as_any_mut().downcast_mut::<RSI>() {
    //            self.rsi = Some(rsi.add(self.close));
    //        } //else if let Some(kdj) = metric_model.as_any().downcast_ref::<KDJ>() {
    //          //  self.kdj = Some(kdj.add(self.high, self.low, self.close));
    //          //}
    //    }
    //}

    pub fn get_parent_klc(&self) -> Option<Handle<CKLine>> {
        assert!(self.sup_kl.is_some());
        self.sup_kl.as_ref().and_then(|sup_kl| sup_kl.klc.clone())
    }

    pub fn include_sub_lv_time(&self, sub_lv_t: &str) -> bool {
        // FIXME: 这里要优化，不要用字符串比较
        if self.time.to_string() == sub_lv_t {
            return true;
        }
        for sub_klu in &self.sub_kl_list {
            let sub_klu = sub_klu;
            if sub_klu.time.to_string() == sub_lv_t || sub_klu.include_sub_lv_time(sub_lv_t) {
                return true;
            }
        }
        false
    }

    pub fn set_pre_klu(&mut self, pre_klu: Option<Handle<CKLineUnit>>) {
        if let Some(pre_klu) = pre_klu {
            pre_klu.as_mut().next = Some(self.as_handle());
            self.pre = Some(pre_klu);
        }
    }

    pub fn set_klc(&mut self, klc: Handle<CKLine>) {
        self.klc = Some(klc);
    }

    pub fn get_klc(&self) -> Option<Handle<CKLine>> {
        self.klc.clone()
    }
}

// FIXME: 所有的都需要添加Display
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

/*pub trait MetricModel {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

impl MetricModel for CMACD {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl MetricModel for CTrendModel {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl MetricModel for BollModel {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl MetricModel for CDemarkEngine {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl MetricModel for RSI {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl MetricModel for KDJ {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}*/

/*impl Clone for CKLineUnit {
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

impl_handle!(CKLineUnit);
