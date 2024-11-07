//use crate::Bar;

//pub trait MetricModel {
//    fn update_bar(&mut self, klu: &Bar);
//}

use crate::{BollModel, CMACD, CRSI};

#[derive(Debug)]
pub enum MetricModel {
    MACD(CMACD),
    BOLL(BollModel),
    //RSI(CRSI),
}
