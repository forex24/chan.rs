//
use super::KLine_Unit::CKLineUnit;
use crate::ChanConfig::CChanConfig;
use crate::Common::handle::Handle;
use crate::Common::CEnum::{FxType, KLineDir};
use crate::KLine::KLine::CKLine;
use crate::Math::metric::MetricModel;

pub struct CKLineList {
    pub kl_type: String,
    pub config: CChanConfig,
    #[allow(clippy::box_collection)]
    pub lst: Box<Vec<CKLine>>,
    pub metric_model_lst: Vec<MetricModel>,
}

impl CKLineList {
    pub fn new(kl_type: &str, conf: &CChanConfig) -> Self {
        Self {
            kl_type: kl_type.to_string(),
            config: conf.clone(),
            lst: Box::new(Vec::with_capacity(102400)),
            metric_model_lst: conf.get_metric_model(),
        }
    }

    pub fn update_candle(&mut self, klu: Handle<CKLineUnit>) -> bool {
        klu.as_mut().set_metric(&mut self.metric_model_lst);
        if self.lst.len() == 0 {
            let candle = CKLine::new(&self.lst, klu, 0, KLineDir::Up);
            self.lst.push(candle);
        } else {
            let _dir = self.lst.last_mut().unwrap().try_add(klu);

            if _dir != KLineDir::Combine {
                let candle = CKLine::new(&self.lst, klu, self.lst.len(), _dir);
                self.lst.push(candle);

                if self.lst.len() >= 3 {
                    let index = self.lst.len() - 2;
                    self.lst[index].fx = self.check_fx();
                }

                return true;
            }
        }
        false
    }

    fn check_fx(&mut self) -> FxType {
        let len = self.lst.len();
        let _cur = &self.lst[len - 2];
        let _pre = &self.lst[len - 3];
        let _next = &self.lst[len - 1];
        if _pre.high < _cur.high
            && _next.high < _cur.high
            && _pre.low < _cur.low
            && _next.low < _cur.low
        {
            return FxType::Top;
        }

        if _pre.high > _cur.high
            && _next.high > _cur.high
            && _pre.low > _cur.low
            && _next.low > _cur.low
        {
            return FxType::Bottom;
        }
        FxType::Unknown
    }

    /*pub fn add_single_klu(&mut self, mut klu: CKLineUnit) -> Result<(), CChanException> {
        klu.set_metric(&mut self.metric_model_lst);
        let klu = Rc::new(RefCell::new(klu));
        if self.lst.is_empty() {
            self.lst.push(CKLine::new(Rc::clone(&klu), 0, KLineDir::Up));
        } else {
            let dir = CKLine::try_add(self.lst.last().as_ref().unwrap(), &klu)?;
            if dir != KLineDir::Combine {
                let new_kline = CKLine::new(Rc::clone(&klu), self.lst.len(), dir);
                self.lst.push(new_kline.clone());
                if self.lst.len() >= 3 {
                    let len = self.lst.len();
                    CKLine::update_fx(&self.lst[len - 2], &self.lst[len - 3], &self.lst[len - 1]);
                }
                if self.bi_list.update_bi(
                    Rc::clone(&self.lst[self.lst.len() - 2]),
                    Rc::clone(&self.lst[self.lst.len() - 1]),
                    true, //self.step_calculation,
                ) && self.step_calculation
                {
                    self.cal_seg_and_zs()?;
                }
            } else if self.step_calculation
                && self
                    .bi_list
                    .try_add_virtual_bi(self.lst.last().unwrap().clone(), true)
            {
                self.cal_seg_and_zs()?;
            }
        }
        Ok(())
    }*/

    //pub fn klu_iter(&self, klc_begin_idx: usize) -> impl Iterator<Item = &Handle<CKLineUnit>> {
    //    self.lst[klc_begin_idx..]
    //        .iter()
    //        .flat_map(|klc| klc.borrow().lst.iter())
    //}
}

impl std::ops::Deref for CKLineList {
    type Target = Box<Vec<CKLine>>;

    fn deref(&self) -> &Self::Target {
        &self.lst
    }
}

impl std::ops::DerefMut for CKLineList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.lst
    }
}
