use std::{cell::RefCell, rc::Rc};

use crate::{Bar, CBspPoint, Direction, Handle, IHighLow, MacdAlgo};

pub trait LineType: IHighLow {
    fn get_begin_klu(&self) -> Handle<Bar>;
    fn get_end_klu(&self) -> Handle<Bar>;

    fn get_begin_val(&self) -> f64;
    fn get_end_val(&self) -> f64;

    fn direction(&self) -> Direction;
    fn is_up(&self) -> bool;
    fn is_down(&self) -> bool;

    fn is_sure(&self) -> bool;

    fn amp(&self) -> f64;
}

pub trait ToHandle {
    fn to_handle(&self) -> Handle<Self>
    where
        Self: Sized;
}

pub trait IParent {
    fn seg_idx(&self) -> Option<usize>;
    fn set_seg_idx(&mut self, idx: usize);

    fn set_parent_seg_idx(&mut self, parent_seg_idx: Option<usize>);
    fn parent_seg_idx(&self) -> Option<usize>;

    fn set_parent_seg_dir(&mut self, dir: Option<Direction>);
    fn parent_seg_dir(&self) -> Option<Direction>;
}

pub trait IBspInfo {
    fn set_bsp(&mut self, bsp: Rc<RefCell<CBspPoint<Self>>>)
    where
        Self: std::marker::Sized;
}

pub trait ICalcMetric {
    fn cal_macd_metric(&self, algo: &MacdAlgo, is_reverse: bool) -> f64;
}
