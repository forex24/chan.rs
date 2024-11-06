use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use crate::AsHandle;
use crate::Bar;
use crate::CBspPoint;
use crate::CEigenFx;
//use crate::CTrendLine;
use crate::Handle;
use crate::IBspInfo;
use crate::ICalcMetric;
use crate::IHighLow;
use crate::Indexable;
use crate::LineType;
//use crate::TrendLineSide;
use crate::CZs;
use crate::IParent;
use crate::ToHandle;
use crate::{Direction, MacdAlgo};

// 线段
#[derive(Debug)]
pub struct CSeg<T> {
    handle: Handle<Self>,
    //pub idx: usize,
    pub start_bi: Handle<T>,
    pub end_bi: Handle<T>,
    pub is_sure: bool,
    pub dir: Direction,
    pub zs_lst: VecDeque<Handle<CZs<T>>>,
    pub eigen_fx: Option<CEigenFx<T>>,
    pub seg_idx: Option<usize>,
    pub parent_seg_idx: Option<usize>,
    pub parent_seg_dir: Option<Direction>,
    pub bsp: Option<Rc<RefCell<CBspPoint<Self>>>>,
    pub bi_list: Vec<Handle<T>>,
    pub reason: String,
    //pub support_trend_line: Option<CTrendLine>,
    //pub resistance_trend_line: Option<CTrendLine>,
    pub ele_inside_is_sure: bool,
}

impl<T: LineType> CSeg<T> {
    #[allow(clippy::borrowed_box)]
    pub fn new(
        boxed_vec: &Box<Vec<Self>>,
        idx: usize,
        start_bi: Handle<T>,
        end_bi: Handle<T>,
        is_sure: bool,
        seg_dir: Option<Direction>,
        reason: &str,
    ) -> Self {
        assert!(
            start_bi.index() == 0 || start_bi.direction() == end_bi.direction() || !is_sure,
            "{} {} {} {}",
            start_bi.index(),
            end_bi.index(),
            start_bi.direction(),
            end_bi.direction()
        );

        let dir = match seg_dir.is_none() {
            true => end_bi.direction(),
            false => seg_dir.unwrap(),
        };
        let mut seg = Self {
            handle: Handle::new(boxed_vec, idx),
            start_bi,
            end_bi,
            is_sure,
            dir,
            zs_lst: VecDeque::new(),
            eigen_fx: None,
            seg_idx: None,        // 线段的线段用
            parent_seg_idx: None, // 在哪个线段里面
            parent_seg_dir: None,
            bsp: None,       //尾部是不是买卖点
            bi_list: vec![], // 仅通过self.update_bi_list来更新
            reason: reason.to_string(),
            //support_trend_line: None,
            //resistance_trend_line: None,
            ele_inside_is_sure: false,
        };

        if end_bi.index() - start_bi.index() < 2 {
            seg.is_sure = false;
        }
        seg.check();
        seg
    }

    // 已完备
    pub fn add_zs(&mut self, zs: Handle<CZs<T>>) {
        self.zs_lst.push_front(zs);
    }

    // 已完备
    pub fn clear_zs_lst(&mut self) {
        self.zs_lst.clear();
    }

    //pub fn set_seg_idx(&mut self, idx: usize) {
    //    self.seg_idx = Some(idx);
    //}

    // 已完备
    fn check(&self) {
        if !self.is_sure {
            return;
        }
        if self._is_down() {
            if self.start_bi.get_begin_val() < self.end_bi.get_end_val() {
                panic!("下降线段起始点应该高于结束点! idx={}", self.index());
            }
        } else if self.start_bi.get_begin_val() > self.end_bi.get_end_val() {
            panic!("上升线段起始点应该低于结束点! idx={}", self.index());
        }
        if self.end_bi.index() - self.start_bi.index() < 2 {
            panic!(
                "线段({}-{})长度不能小于2! idx={}",
                self.start_bi.index(),
                self.end_bi.index(),
                self.index()
            );
        }
    }

    fn __cal_klu_slope(&self) -> f64 {
        assert!(self.end_bi.index() >= self.start_bi.index());
        (self._get_end_val() - self._get_begin_val())
            / (self._get_end_klu().index() - self._get_begin_klu().index()) as f64
            / self._get_begin_val()
    }

    fn __cal_amp(&self) -> f64 {
        (self._get_end_val() - self._get_begin_val()) / self._get_begin_val()
    }

    fn __cal_bi_cnt(&self) -> usize {
        self.end_bi.index() - self.start_bi.index() + 1
    }

    // 已完备
    fn _low(&self) -> f64 {
        if self._is_down() {
            self.end_bi.get_end_klu().low
        } else {
            self.start_bi.get_begin_klu().low
        }
    }

    // 已完备
    fn _high(&self) -> f64 {
        if self._is_up() {
            self.end_bi.get_end_klu().high
        } else {
            self.start_bi.get_begin_klu().high
        }
    }

    // 已完备
    fn _is_down(&self) -> bool {
        self.dir == Direction::Down
    }

    // 已完备
    fn _is_up(&self) -> bool {
        self.dir == Direction::Up
    }

    // 已完备
    fn _get_end_val(&self) -> f64 {
        self.end_bi.get_end_val()
    }

    // 已完备
    fn _get_begin_val(&self) -> f64 {
        self.start_bi.get_begin_val()
    }

    // 已完备
    fn _amp(&self) -> f64 {
        (self._get_end_val() - self._get_begin_val()).abs()
    }

    // 已完备
    fn _get_end_klu(&self) -> Handle<Bar> {
        self.end_bi.get_end_klu().as_handle()
    }

    // 已完备
    fn _get_begin_klu(&self) -> Handle<Bar> {
        self.start_bi.get_begin_klu().as_handle()
    }

    // 已完备
    fn __get_klu_cnt(&self) -> usize {
        self._get_end_klu().index() - self._get_begin_klu().index() + 1
    }

    // 已完备
    fn _cal_macd_metric(&self, macd_algo: &MacdAlgo, _is_reverse: bool) -> f64 {
        match macd_algo {
            MacdAlgo::Slope => self.cal_macd_slope(),
            MacdAlgo::Amp => self.cal_macd_amp(),
            _ => panic!(),
        }
    }

    // 已完备, 计算MACD斜率
    fn cal_macd_slope(&self) -> f64 {
        let begin_klu = self._get_begin_klu();
        let end_klu = self._get_end_klu();
        if self._is_up() {
            (end_klu.high - begin_klu.low)
                / end_klu.high
                / (end_klu.index() - begin_klu.index() + 1) as f64
        } else {
            (begin_klu.high - end_klu.low)
                / begin_klu.high
                / (end_klu.index() - begin_klu.index() + 1) as f64
        }
    }

    // 已完备，计算MACD强度
    fn cal_macd_amp(&self) -> f64 {
        let begin_klu = self._get_begin_klu();
        let end_klu = self._get_end_klu();
        if self._is_down() {
            (begin_klu.high - end_klu.low) / begin_klu.high
        } else {
            (end_klu.high - begin_klu.low) / begin_klu.low
        }
    }
}

impl<T: LineType + IParent + ToHandle> CSeg<T> {
    // TODO:趋势性计算
    pub fn update_bi_list(&mut self, bi_lst: &[T], idx1: usize, idx2: usize) {
        (idx1..=idx2).for_each(|bi_idx| {
            bi_lst[bi_idx]
                .to_handle()
                .as_mut()
                .set_parent_seg_idx(self.handle.index());
            bi_lst[bi_idx]
                .to_handle()
                .as_mut()
                .set_parent_seg_dir(self.dir);

            self.bi_list.push(bi_lst[bi_idx].to_handle());
        });
        // TODO:
        //if self.bi_list.len() >= 3 {
        //    self.support_trend_line = Some(CTrendLine::new(&self.bi_list, TrendLineSide::Inside));
        //    self.resistance_trend_line =
        //        Some(CTrendLine::new(&self.bi_list, TrendLineSide::Outside));
        //}
    }
}

impl<T: LineType> CSeg<T> {
    // 已完备
    pub fn get_first_multi_bi_zs(&self) -> Option<Handle<CZs<T>>> {
        self.zs_lst.iter().find(|zs| !zs.is_one_bi_zs()).copied()
    }

    // 已完备
    pub fn get_final_multi_bi_zs(&self) -> Option<Handle<CZs<T>>> {
        self.zs_lst
            .iter()
            .rev()
            .find(|zs| !zs.is_one_bi_zs())
            .copied()
    }

    // 已完备
    pub fn get_multi_bi_zs_cnt(&self) -> usize {
        self.zs_lst.iter().filter(|zs| !zs.is_one_bi_zs()).count()
    }
}

impl<T: LineType> IHighLow for CSeg<T> {
    fn high(&self) -> f64 {
        self._high()
    }

    fn low(&self) -> f64 {
        self._low()
    }
}

impl<T> IParent for CSeg<T> {
    fn seg_idx(&self) -> Option<usize> {
        self.seg_idx
    }

    fn set_seg_idx(&mut self, idx: usize) {
        self.seg_idx = Some(idx)
    }

    fn set_parent_seg_idx(&mut self, parent_seg_idx: usize) {
        self.parent_seg_idx = Some(parent_seg_idx);
    }

    fn parent_seg_idx(&self) -> Option<usize> {
        self.parent_seg_idx
    }

    fn set_parent_seg_dir(&mut self, dir: Direction) {
        self.parent_seg_dir = Some(dir);
    }

    fn parent_seg_dir(&self) -> Option<Direction> {
        self.parent_seg_dir
    }
}

impl<T> IBspInfo for CSeg<T> {
    fn set_bsp(&mut self, bsp: Rc<RefCell<CBspPoint<Self>>>) {
        self.bsp = Some(bsp);
    }
}

impl<T> ToHandle for CSeg<T> {
    fn to_handle(&self) -> Handle<Self> {
        self.handle
    }
}

impl<T: ICalcMetric + LineType> ICalcMetric for CSeg<T> {
    fn cal_macd_metric(&self, macd_algo: &MacdAlgo, is_reverse: bool) -> f64 {
        self._cal_macd_metric(macd_algo, is_reverse)
    }
}

impl<T: LineType> LineType for CSeg<T> {
    fn get_begin_klu(&self) -> Handle<Bar> {
        self._get_begin_klu().as_handle()
    }

    fn get_end_klu(&self) -> Handle<Bar> {
        self._get_end_klu().as_handle()
    }

    fn get_begin_val(&self) -> f64 {
        self._get_begin_val()
    }

    fn get_end_val(&self) -> f64 {
        self._get_end_val()
    }

    fn direction(&self) -> Direction {
        self.dir
    }

    fn is_up(&self) -> bool {
        self._is_up()
    }

    fn is_down(&self) -> bool {
        self._is_down()
    }

    fn amp(&self) -> f64 {
        self._amp()
    }

    fn is_sure(&self) -> bool {
        self.is_sure
    }
}

impl<T> std::fmt::Display for CSeg<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fx_str = match self.eigen_fx {
            None => "[]".to_string(),
            Some(ref fx) => {
                format!(
                    " {} | {} | {}",
                    fx.ele[0]
                        .as_ref()
                        .unwrap()
                        .lst
                        .iter()
                        .map(|x| x.index().to_string())
                        .collect::<Vec<_>>()
                        .join(","),
                    fx.ele[1]
                        .as_ref()
                        .unwrap()
                        .lst
                        .iter()
                        .map(|x| x.index().to_string())
                        .collect::<Vec<_>>()
                        .join(","),
                    fx.ele[2]
                        .as_ref()
                        .unwrap()
                        .lst
                        .iter()
                        .map(|x| x.index().to_string())
                        .collect::<Vec<_>>()
                        .join(","),
                )
            }
        };

        write!(
            f,
            "index {} begin:{} end:{} dir:{} is_sure:{} fx:{} reason:{} ele_inside_is_sure:{}",
            self.index(),
            self.start_bi.index(),
            self.end_bi.index(),
            self.dir,
            self.is_sure,
            fx_str,
            self.reason,
            self.ele_inside_is_sure
        )
    }
}

impl_handle!(CSeg<T>);
