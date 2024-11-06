use crate::{AsHandle, Bar, Handle, Kline};

#[derive(Debug)]
pub struct CBarList {
    #[allow(clippy::box_collection)]
    bar_list: Box<Vec<Bar>>,
    //metric
    //pub metric_model_lst: Vec<Box<dyn MetricModel>>,
}

impl CBarList {
    pub fn new() -> Self {
        Self {
            bar_list: Box::new(Vec::with_capacity(10_024_000)),
            //metric_model_lst: vec![],
        }
    }

    pub fn add_kline(&mut self, k: &Kline) -> Handle<Bar> {
        let bar = Bar::new(
            &self.bar_list,
            self.bar_list.len(),
            k.time,
            k.open,
            k.high,
            k.low,
            k.close,
        );
        self.add_bar(bar)
    }

    fn add_bar(&mut self, bar: Bar) -> Handle<Bar> {
        //bar.set_metric(&mut self.metric_model_lst);
        let klu_handle = bar.as_handle();
        self.bar_list.push(bar);
        klu_handle
    }
}

impl Default for CBarList {
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Deref for CBarList {
    type Target = Box<Vec<Bar>>;

    fn deref(&self) -> &Self::Target {
        &self.bar_list
    }
}

impl std::ops::DerefMut for CBarList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bar_list
    }
}
