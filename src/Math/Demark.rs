use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use crate::Common::types::SharedCell;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BiDir {
    Up,
    Down,
}

#[derive(Clone, Copy, Debug)]
pub struct CKL {
    idx: i32,
    close: f64,
    high: f64,
    low: f64,
}

impl CKL {
    fn new(idx: i32, close: f64, high: f64, low: f64) -> Self {
        CKL {
            idx,
            close,
            high,
            low,
        }
    }

    fn v(&self, is_close: bool, dir: BiDir) -> f64 {
        if is_close {
            self.close
        } else if dir == BiDir::Up {
            self.high
        } else {
            self.low
        }
    }
}

#[derive(Clone, Debug)]
pub enum DemarkType {
    Setup,
    Countdown,
}

#[derive(Clone, Debug)]
pub struct DemarkIndex {
    dir: BiDir,
    idx: i32,
    demark_type: DemarkType,
    series: SharedCell<CDemarkSetup>,
}

pub struct CDemarkIndex {
    data: Vec<DemarkIndex>,
}

impl CDemarkIndex {
    fn new() -> Self {
        CDemarkIndex { data: Vec::new() }
    }

    fn add(
        &mut self,
        dir: BiDir,
        demark_type: DemarkType,
        idx: i32,
        series: SharedCell<CDemarkSetup>,
    ) {
        self.data.push(DemarkIndex {
            dir,
            idx,
            demark_type,
            series,
        });
    }

    fn get_setup(&self) -> Vec<&DemarkIndex> {
        self.data
            .iter()
            .filter(|info| matches!(info.demark_type, DemarkType::Setup))
            .collect()
    }

    fn get_countdown(&self) -> Vec<&DemarkIndex> {
        self.data
            .iter()
            .filter(|info| matches!(info.demark_type, DemarkType::Countdown))
            .collect()
    }

    fn update(&mut self, demark_index: &CDemarkIndex) {
        self.data.extend(demark_index.data.clone());
    }
}

pub struct CDemarkCountdown {
    dir: BiDir,
    kl_list: VecDeque<CKL>,
    idx: i32,
    tdst_peak: f64,
    finish: bool,
}

impl CDemarkCountdown {
    fn new(dir: BiDir, kl_list: &[CKL], tdst_peak: f64) -> Self {
        CDemarkCountdown {
            dir,
            kl_list: VecDeque::from(kl_list.to_vec()),
            idx: 0,
            tdst_peak,
            finish: false,
        }
    }

    fn update(&mut self, kl: CKL) -> bool {
        if self.finish {
            return false;
        }
        self.kl_list.push_back(kl);
        if self.kl_list.len() <= CDemarkEngine::COUNTDOWN_BIAS as usize {
            return false;
        }
        if self.idx == CDemarkEngine::MAX_COUNTDOWN {
            self.finish = true;
            return false;
        }
        if (self.dir == BiDir::Down && kl.high > self.tdst_peak)
            || (self.dir == BiDir::Up && kl.low < self.tdst_peak)
        {
            self.finish = true;
            return false;
        }
        let last = self.kl_list.back().unwrap();
        let compare = self.kl_list[self.kl_list.len() - 1 - CDemarkEngine::COUNTDOWN_BIAS as usize];
        if (self.dir == BiDir::Down
            && last.close < compare.v(CDemarkEngine::COUNTDOWN_CMP2CLOSE, self.dir))
            || (self.dir == BiDir::Up
                && last.close > compare.v(CDemarkEngine::COUNTDOWN_CMP2CLOSE, self.dir))
        {
            self.idx += 1;
            return true;
        }
        false
    }
}

pub struct CDemarkSetup {
    dir: BiDir,
    kl_list: VecDeque<CKL>,
    pre_kl: CKL,
    countdown: Option<CDemarkCountdown>,
    setup_finished: bool,
    idx: i32,
    tdst_peak: Option<f64>,
    last_demark_index: CDemarkIndex,
}

impl CDemarkSetup {
    fn new(dir: BiDir, kl_list: &[CKL], pre_kl: CKL) -> Self {
        assert_eq!(kl_list.len(), CDemarkEngine::SETUP_BIAS as usize);
        CDemarkSetup {
            dir,
            kl_list: VecDeque::from(kl_list.to_vec()),
            pre_kl,
            countdown: None,
            setup_finished: false,
            idx: 0,
            tdst_peak: None,
            last_demark_index: CDemarkIndex::new(),
        }
    }

    fn update(&mut self, kl: CKL) -> CDemarkIndex {
        self.last_demark_index = CDemarkIndex::new();
        if !self.setup_finished {
            self.kl_list.push_back(kl);
            let last = self.kl_list.back().unwrap();
            let compare = self.kl_list[self.kl_list.len() - 1 - CDemarkEngine::SETUP_BIAS as usize];
            if (self.dir == BiDir::Down
                && last.close < compare.v(CDemarkEngine::SETUP_CMP2CLOSE, self.dir))
                || (self.dir == BiDir::Up
                    && last.close > compare.v(CDemarkEngine::SETUP_CMP2CLOSE, self.dir))
            {
                self.add_setup();
            } else {
                self.setup_finished = true;
            }
        }
        if self.idx == CDemarkEngine::DEMARK_LEN && !self.setup_finished && self.countdown.is_none()
        {
            let tdst_peak = self.cal_tdst_peak();
            self.countdown = Some(CDemarkCountdown::new(self.dir, &self.kl_list, tdst_peak));
        }
        if let Some(countdown) = &mut self.countdown {
            if countdown.update(kl) {
                self.last_demark_index.add(
                    self.dir,
                    DemarkType::Countdown,
                    countdown.idx,
                    Rc::new(RefCell::new(self.clone())),
                );
            }
        }
        self.last_demark_index.clone()
    }

    fn add_setup(&mut self) {
        self.idx += 1;
        self.last_demark_index.add(
            self.dir,
            DemarkType::Setup,
            self.idx,
            Rc::new(RefCell::new(self.clone())),
        );
    }

    fn cal_tdst_peak(&mut self) -> f64 {
        assert_eq!(
            self.kl_list.len(),
            (CDemarkEngine::SETUP_BIAS + CDemarkEngine::DEMARK_LEN) as usize
        );
        let arr: Vec<_> = self
            .kl_list
            .iter()
            .skip(CDemarkEngine::SETUP_BIAS as usize)
            .take(CDemarkEngine::DEMARK_LEN as usize)
            .collect();
        assert_eq!(arr.len(), CDemarkEngine::DEMARK_LEN as usize);
        let res = if self.dir == BiDir::Down {
            let mut res = arr
                .iter()
                .map(|kl| kl.high)
                .fold(f64::NEG_INFINITY, f64::max);
            if CDemarkEngine::TIAOKONG_ST && arr[0].high < self.pre_kl.close {
                res = res.max(self.pre_kl.close);
            }
            res
        } else {
            let mut res = arr.iter().map(|kl| kl.low).fold(f64::INFINITY, f64::min);
            if CDemarkEngine::TIAOKONG_ST && arr[0].low > self.pre_kl.close {
                res = res.min(self.pre_kl.close);
            }
            res
        };
        self.tdst_peak = Some(res);
        res
    }
}

pub struct CDemarkEngine {
    kl_lst: Vec<CKL>,
    series: Vec<SharedCell<CDemarkSetup>>,
}

impl CDemarkEngine {
    const DEMARK_LEN: i32 = 9;
    const SETUP_BIAS: i32 = 4;
    const COUNTDOWN_BIAS: i32 = 2;
    const MAX_COUNTDOWN: i32 = 13;
    const TIAOKONG_ST: bool = true;
    const SETUP_CMP2CLOSE: bool = true;
    const COUNTDOWN_CMP2CLOSE: bool = true;

    pub fn new() -> Self {
        CDemarkEngine {
            kl_lst: Vec::new(),
            series: Vec::new(),
        }
    }

    pub fn update(&mut self, idx: i32, close: f64, high: f64, low: f64) -> CDemarkIndex {
        self.kl_lst.push(CKL::new(idx, close, high, low));
        if self.kl_lst.len() <= (Self::SETUP_BIAS + 1) as usize {
            return CDemarkIndex::new();
        }

        let last = self.kl_lst.last().unwrap();
        let compare = self.kl_lst[self.kl_lst.len() - 1 - Self::SETUP_BIAS as usize];
        if last.close < compare.close {
            if !self
                .series
                .iter()
                .any(|s| s.borrow().dir == BiDir::Down && !s.borrow().setup_finished)
            {
                let new_series = Rc::new(RefCell::new(CDemarkSetup::new(
                    BiDir::Down,
                    &self.kl_lst
                        [self.kl_lst.len() - Self::SETUP_BIAS as usize - 1..self.kl_lst.len() - 1],
                    self.kl_lst[self.kl_lst.len() - Self::SETUP_BIAS as usize - 2],
                )));
                self.series.push(new_series);
            }
            for series in &self.series {
                let mut s = series.borrow_mut();
                if s.dir == BiDir::Up && s.countdown.is_none() && !s.setup_finished {
                    s.setup_finished = true;
                }
            }
        } else if last.close > compare.close {
            if !self
                .series
                .iter()
                .any(|s| s.borrow().dir == BiDir::Up && !s.borrow().setup_finished)
            {
                let new_series = Rc::new(RefCell::new(CDemarkSetup::new(
                    BiDir::Up,
                    &self.kl_lst
                        [self.kl_lst.len() - Self::SETUP_BIAS as usize - 1..self.kl_lst.len() - 1],
                    self.kl_lst[self.kl_lst.len() - Self::SETUP_BIAS as usize - 2],
                )));
                self.series.push(new_series);
            }
            for series in &self.series {
                let mut s = series.borrow_mut();
                if s.dir == BiDir::Down && s.countdown.is_none() && !s.setup_finished {
                    s.setup_finished = true;
                }
            }
        }

        self.clear();
        self.clean_series_from_setup_finish();

        let result = self.cal_result();
        self.clear();
        result
    }

    fn cal_result(&self) -> CDemarkIndex {
        let mut demark_index = CDemarkIndex::new();
        for series in &self.series {
            demark_index.update(&series.borrow().last_demark_index);
        }
        demark_index
    }

    fn clear(&mut self) {
        self.series.retain(|s| {
            let s = s.borrow();
            !(s.setup_finished && s.countdown.is_none())
                && !(s.countdown.as_ref().map_or(false, |c| c.finish))
        });
    }

    fn clean_series_from_setup_finish(&mut self) {
        let mut finished_setup = None;
        for series in &self.series {
            let mut s = series.borrow_mut();
            let demark_idx = s.update(*self.kl_lst.last().unwrap());
            for setup_idx in demark_idx.get_setup() {
                if setup_idx.idx == Self::DEMARK_LEN {
                    assert!(finished_setup.is_none());
                    finished_setup = Some(Rc::clone(series));
                }
            }
        }
        if let Some(finished) = finished_setup {
            self.series.retain(|s| Rc::ptr_eq(s, &finished));
        }
    }
}
