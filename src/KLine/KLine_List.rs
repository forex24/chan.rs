use std::cell::RefCell;
use std::rc::Rc;

//
use super::KLine_Unit::{CKLineUnit, MetricModel};
use crate::ChanConfig::CChanConfig;
use crate::Common::types::Handle;
use crate::Common::CEnum::KLineDir;
use crate::KLine::KLine::CKLine;

pub struct CKLineList {
    pub kl_type: String,
    pub config: CChanConfig,
    pub lst: Vec<Handle<CKLine>>,
    pub metric_model_lst: Vec<Box<dyn MetricModel>>,
}

impl CKLineList {
    pub fn new(kl_type: &str, conf: &CChanConfig) -> Self {
        Self {
            kl_type: kl_type.to_string(),
            config: conf.clone(),
            lst: Vec::new(),
            metric_model_lst: conf.get_metric_model(),
        }
    }

    pub fn add_single_klu(&mut self, mut klu: CKLineUnit) -> Option<KLineDir> {
        klu.set_metric(&mut self.metric_model_lst);
        let klu = Rc::new(RefCell::new(klu));

        if self.lst.is_empty() {
            self.lst.push(CKLine::new(Rc::clone(&klu), 0, KLineDir::Up));
            return None;
        }

        let dir = CKLine::try_add(self.lst.last().unwrap(), &klu);

        if dir != KLineDir::Combine {
            let new_kline = CKLine::new(Rc::clone(&klu), self.lst.len(), dir);
            self.lst.push(new_kline.clone());
            if self.lst.len() >= 3 {
                let len = self.lst.len();
                CKLine::update_fx(&self.lst[len - 2], &self.lst[len - 3], &self.lst[len - 1]);
            }
        }
        Some(dir)
    }

    //pub fn klu_iter<'a>(
    //    &'a self,
    //    klc_begin_idx: usize,
    //) -> impl Iterator<Item = &'a StrongHandle<CKLineUnit>> {
    //    self.lst[klc_begin_idx..]
    //        .iter()
    //        .flat_map(|klc| klc.borrow().lst.iter())
    //}
}

impl std::ops::Deref for CKLineList {
    type Target = Vec<Handle<CKLine>>;

    fn deref(&self) -> &Self::Target {
        &self.lst
    }
}

impl std::ops::DerefMut for CKLineList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.lst
    }
}
