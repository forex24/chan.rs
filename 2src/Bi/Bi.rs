use crate::impl_handle;
use crate::BuySellPoint::BS_Point::CBSPoint;
use crate::Common::handle::{Handle, Indexable};
use crate::Common::CEnum::{BiDir, BiType, FxType, MacdAlgo};
use crate::Common::ChanException::{CChanException, ErrCode};
use crate::KLine::KLine::CKLine;
use crate::KLine::KLine_Unit::CKLineUnit;
use crate::Seg::Seg::CSeg;

pub struct CBi {
    handle: Handle<Self>,
    pub begin_klc: Handle<CKLine>,
    pub end_klc: Handle<CKLine>,
    pub dir: BiDir,
    pub bi_type: BiType,
    pub is_sure: bool,
    pub sure_end: Vec<Handle<CKLine>>,
    pub seg_idx: Option<usize>,
    pub parent_seg: Option<Handle<CSeg<CBi>>>,
    pub bsp: Option<Handle<CBSPoint<CBi>>>,
    pub next: Option<Handle<Self>>,
    pub pre: Option<Handle<Self>>,
    //pub memoize_cache: RefCell<HashMap<String, f64>>,
}

impl CBi {
    pub fn new(
        boxed_vec: &Box<Vec<Self>>,
        begin_klc: Handle<CKLine>,
        end_klc: Handle<CKLine>,
        idx: usize,
        is_sure: bool,
    ) -> Self {
        let mut bi = CBi {
            handle: Handle::new(boxed_vec, idx),
            begin_klc,
            end_klc,
            dir: BiDir::Up, // 临时值，将在set方法中更新
            bi_type: BiType::Strict,
            is_sure,
            sure_end: Vec::new(),
            seg_idx: None,
            parent_seg: None,
            bsp: None,
            next: None,
            pre: None,
            //memoize_cache: RefCell::new(HashMap::new()),
        };
        bi.set(begin_klc, end_klc).unwrap();
        bi
    }

    pub fn clean_cache(&self) {
        //self.memoize_cache.borrow_mut().clear();
    }

    pub fn begin_klc(&self) -> Handle<CKLine> {
        self.begin_klc
    }

    pub fn end_klc(&self) -> Handle<CKLine> {
        self.end_klc
    }

    pub fn dir(&self) -> BiDir {
        self.dir
    }

    pub fn idx(&self) -> usize {
        self.handle.index()
    }

    pub fn bi_type(&self) -> BiType {
        self.bi_type
    }

    pub fn is_sure(&self) -> bool {
        self.is_sure
    }

    pub fn sure_end(&self) -> &Vec<Handle<CKLine>> {
        &self.sure_end
    }

    pub fn seg_idx(&self) -> Option<usize> {
        self.seg_idx
    }

    pub fn set_seg_idx(&mut self, idx: usize) {
        self.seg_idx = Some(idx);
    }

    pub fn check(&self) -> Result<(), CChanException> {
        if self.is_down() {
            if self.begin_klc.high <= self.end_klc.low {
                return Err(CChanException::new(
                    format!(
                        "{}:{}~{} 笔的方向和收尾位置不一致!",
                        self.index(),
                        self.begin_klc.lst[0].time,
                        self.end_klc.lst.last().unwrap().time
                    ),
                    ErrCode::BiErr,
                ));
            }
        } else if self.begin_klc.low >= self.end_klc.high {
            return Err(CChanException::new(
                format!(
                    "{}:{}~{} 笔的方向和收尾位置不一致!",
                    self.index(),
                    self.begin_klc.lst[0].time,
                    self.end_klc.lst.last().unwrap().time
                ),
                ErrCode::BiErr,
            ));
        }
        Ok(())
    }

    pub fn set(
        &mut self,
        begin_klc: Handle<CKLine>,
        end_klc: Handle<CKLine>,
    ) -> Result<(), CChanException> {
        self.begin_klc = begin_klc;
        self.end_klc = end_klc;
        self.dir = match begin_klc.fx {
            FxType::Bottom => BiDir::Up,
            FxType::Top => BiDir::Down,
            _ => {
                return Err(CChanException::new(
                    "ERROR DIRECTION when creating bi".to_string(),
                    ErrCode::BiErr,
                ))
            }
        };
        self.check()?;
        self.clean_cache();
        Ok(())
    }

    pub fn get_begin_val(&self) -> f64 {
        if self.is_up() {
            self.begin_klc.low
        } else {
            self.begin_klc.high
        }
    }

    pub fn get_end_val(&self) -> f64 {
        if self.is_up() {
            self.end_klc.high
        } else {
            self.end_klc.low
        }
    }

    pub fn get_begin_klu(&self) -> &CKLineUnit {
        if self.is_up() {
            self.begin_klc.get_peak_klu(false).unwrap()
        } else {
            self.begin_klc.get_peak_klu(true).unwrap()
        }
    }

    pub fn get_end_klu(&self) -> &CKLineUnit {
        if self.is_up() {
            self.end_klc.get_peak_klu(true).unwrap()
        } else {
            self.end_klc.get_peak_klu(false).unwrap()
        }
    }

    pub fn amp(&self) -> f64 {
        (self.get_end_val() - self.get_begin_val()).abs()
    }

    pub fn get_klu_cnt(&self) -> usize {
        self.get_end_klu().index() - self.get_begin_klu().index() + 1
    }

    pub fn get_klc_cnt(&self) -> usize {
        assert_eq!(
            self.end_klc.index(),
            self.get_end_klu().klc.as_ref().unwrap().index()
        );
        assert_eq!(
            self.begin_klc.index(),
            self.get_begin_klu().klc.as_ref().unwrap().index()
        );
        self.end_klc.index() - self.begin_klc.index() + 1
    }

    pub fn high(&self) -> f64 {
        if self.is_up() {
            self.end_klc.high
        } else {
            self.begin_klc.high
        }
    }

    pub fn low(&self) -> f64 {
        if self.is_up() {
            self.begin_klc.low
        } else {
            self.end_klc.low
        }
    }

    pub fn mid(&self) -> f64 {
        (self.high() + self.low()) / 2.0
    }

    pub fn is_down(&self) -> bool {
        self.dir == BiDir::Down
    }

    pub fn is_up(&self) -> bool {
        self.dir == BiDir::Up
    }

    pub fn update_virtual_end(&mut self, new_klc: Handle<CKLine>) {
        self.append_sure_end(self.end_klc);
        self.update_new_end(new_klc);
        self.is_sure = false;
    }

    pub fn restore_from_virtual_end(&mut self, sure_end: Handle<CKLine>) {
        self.is_sure = true;
        self.update_new_end(sure_end);
        self.sure_end.clear();
    }

    pub fn append_sure_end(&mut self, klc: Handle<CKLine>) {
        self.sure_end.push(klc);
    }

    pub fn update_new_end(&mut self, new_klc: Handle<CKLine>) {
        self.end_klc = new_klc;
        self.check().unwrap();
        self.clean_cache();
    }

    pub fn cal_macd_metric(
        &self,
        macd_algo: MacdAlgo,
        is_reverse: bool,
    ) -> Result<f64, CChanException> {
        match macd_algo {
            //MacdAlgo::Rsi => self.cal_rsi(),
            MacdAlgo::Area => self.cal_macd_half(is_reverse),
            MacdAlgo::Peak => self.cal_macd_peak(),
            MacdAlgo::FullArea => self.cal_macd_area(),
            MacdAlgo::Diff => self.cal_macd_diff(),
            MacdAlgo::Slope => self.cal_macd_slope(),
            MacdAlgo::Amp => self.cal_macd_amp(),
            //MacdAlgo::Amount => self.cal_macd_trade_metric(DataField::FieldTurnover, false),
            //MacdAlgo::Volumn => self.cal_macd_trade_metric(DataField::FieldVolume, false),
            //MacdAlgo::VolumnAvg => self.cal_macd_trade_metric(DataField::FieldVolume, true),
            //MacdAlgo::AmountAvg => self.cal_macd_trade_metric(DataField::FieldTurnover, true),
            //MacdAlgo::TurnrateAvg => self.cal_macd_trade_metric(DataField::FieldTurnrate, true),
            _ => Err(CChanException::new(
                format!(
                    "unsupport macd_algo={:?}, should be one of area/full_area/peak/diff/slope/amp",
                    macd_algo
                ),
                ErrCode::ParaError,
            )),
        }
    }

    //pub fn cal_rsi(&self) -> Result<f64, CChanException> {
    //    let mut rsi_lst: Vec<f64> = Vec::new();
    //    for klc in self.klc_lst() {
    //        for klu in klc.lst.iter() {
    //            if let Some(rsi) = klu.rsi {
    //                rsi_lst.push(rsi);
    //            }
    //        }
    //    }
    //    if self.is_down() {
    //        Ok(10000.0 / (rsi_lst.iter().fold(f64::INFINITY, |a, &b| a.min(b)) + 1e-7))
    //    } else {
    //        Ok(rsi_lst
    //            .iter()
    //            .max_by(|a, b| a.partial_cmp(b).unwrap())
    //            .copied()
    //            .unwrap_or(0.0))
    //    }
    //}

    pub fn cal_macd_area(&self) -> Result<f64, CChanException> {
        let mut s = 1e-7;
        for klc in self.klc_lst() {
            for klu in klc.lst.iter() {
                s += klu.macd.as_ref().unwrap().macd.abs();
            }
        }
        Ok(s)
    }

    pub fn cal_macd_peak(&self) -> Result<f64, CChanException> {
        let mut peak = 1e-7;
        for klc in self.klc_lst() {
            for klu in klc.lst.iter() {
                let macd_val = klu.macd.as_ref().unwrap().macd;
                if macd_val.abs() > peak {
                    if self.is_down() && macd_val < 0.0 {
                        peak = macd_val.abs();
                    } else if self.is_up() && macd_val > 0.0 {
                        peak = macd_val.abs();
                    }
                }
            }
        }
        Ok(peak)
    }

    pub fn cal_macd_half(&self, is_reverse: bool) -> Result<f64, CChanException> {
        if is_reverse {
            self.cal_macd_half_reverse()
        } else {
            self.cal_macd_half_obverse()
        }
    }

    pub fn cal_macd_half_obverse(&self) -> Result<f64, CChanException> {
        let mut s = 1e-7;
        let begin_klu = self.get_begin_klu();
        let peak_macd = begin_klu.macd.as_ref().unwrap().macd;
        for klc in self.klc_lst() {
            for klu in klc.lst.iter() {
                if klu.index() < begin_klu.index() {
                    continue;
                }
                if klu.macd.as_ref().unwrap().macd * peak_macd > 0.0 {
                    s += klu.macd.as_ref().unwrap().macd.abs();
                } else {
                    return Ok(s);
                }
            }
        }
        Ok(s)
    }

    pub fn cal_macd_half_reverse(&self) -> Result<f64, CChanException> {
        let mut s = 1e-7;
        let begin_klu = self.get_end_klu();
        let peak_macd = begin_klu.macd.as_ref().unwrap().macd;
        for klc in self.klc_lst_re() {
            for klu in klc.lst.iter().rev() {
                if klu.index() > begin_klu.index() {
                    continue;
                }
                if klu.macd.as_ref().unwrap().macd * peak_macd > 0.0 {
                    s += klu.macd.as_ref().unwrap().macd.abs();
                } else {
                    return Ok(s);
                }
            }
        }
        Ok(s)
    }

    pub fn cal_macd_diff(&self) -> Result<f64, CChanException> {
        let mut max = f64::NEG_INFINITY;
        let mut min = f64::INFINITY;
        for klc in self.klc_lst() {
            for klu in klc.lst.iter() {
                let macd = klu.macd.as_ref().unwrap().macd;
                if macd > max {
                    max = macd;
                }
                if macd < min {
                    min = macd;
                }
            }
        }
        Ok(max - min)
    }

    pub fn cal_macd_slope(&self) -> Result<f64, CChanException> {
        let begin_klu = self.get_begin_klu();
        let end_klu = self.get_end_klu();
        if self.is_up() {
            Ok((end_klu.high - begin_klu.low)
                / end_klu.high
                / (end_klu.index() - begin_klu.index() + 1) as f64)
        } else {
            Ok((begin_klu.high - end_klu.low)
                / begin_klu.high
                / (end_klu.index() - begin_klu.index() + 1) as f64)
        }
    }

    pub fn cal_macd_amp(&self) -> Result<f64, CChanException> {
        let begin_klu = self.get_begin_klu();
        let end_klu = self.get_end_klu();
        if self.is_down() {
            Ok((begin_klu.high - end_klu.low) / begin_klu.high)
        } else {
            Ok((end_klu.high - begin_klu.low) / begin_klu.low)
        }
    }

    //pub fn cal_macd_trade_metric(
    //    &self,
    //    metric: DataField,
    //    cal_avg: bool,
    //) -> Result<f64, CChanException> {
    //    let mut s = 0.0;
    //    let mut count = 0;
    //    for klc in self.klc_lst() {
    //        for klu in klc.lst.iter() {
    //            if let Some(metric_res) = klu.trade_info.metric.get(&metric.to_string()) {
    //                s += metric_res;
    //                count += 1;
    //            } else {
    //                return Ok(0.0);
    //            }
    //        }
    //    }
    //    if cal_avg && count > 0 {
    //        Ok(s / count as f64)
    //    } else {
    //        Ok(s)
    //    }
    //}

    // Helper methods for iterating over KLines
    fn klc_lst(&self) -> impl Iterator<Item = Handle<CKLine>> {
        KlcIterator {
            current: Some(self.begin_klc),
            end_idx: self.end_klc.index(),
        }
    }

    fn klc_lst_re(&self) -> impl Iterator<Item = Handle<CKLine>> {
        KlcReverseIterator {
            current: Some(self.end_klc),
            begin_idx: self.begin_klc.index(),
        }
    }
}

struct KlcIterator {
    current: Option<Handle<CKLine>>,
    end_idx: usize,
}

impl Iterator for KlcIterator {
    type Item = Handle<CKLine>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current.take()?;
        let current_idx = current.index();
        if current_idx <= self.end_idx {
            self.current = current.next.clone();
            Some(current)
        } else {
            None
        }
    }
}

struct KlcReverseIterator {
    current: Option<Handle<CKLine>>,
    begin_idx: usize,
}

impl Iterator for KlcReverseIterator {
    type Item = Handle<CKLine>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current.take()?;
        let current_idx = current.index();
        if current_idx >= self.begin_idx {
            self.current = current.pre.clone();
            Some(current)
        } else {
            None
        }
    }
}

impl_handle!(CBi);
