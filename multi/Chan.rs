use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ChanConfig::CChanConfig;
use crate::Common::func_util::check_kltype_order;
use crate::Common::CEnum::{AuType, KlType, DATA_SRC};
use crate::Common::CTime::CTime;
use crate::Common::ChanException::CChanException;
use crate::DataAPI::CommonStockAPI::CCommonStockApi;
use crate::KLine::KLine_List::CKLineList;
use crate::KLine::KLine_Unit::CKLineUnit;

pub struct CChan {
    code: String,
    begin_time: Option<String>,
    end_time: Option<String>,
    autype: AuType,
    data_src: DATA_SRC,
    lv_list: Vec<KlType>,
    conf: CChanConfig,
    kl_misalign_cnt: usize,
    kl_inconsistent_detail: HashMap<String, Vec<CTime>>,
    g_kl_iter: HashMap<KlType, Vec<Box<dyn Iterator<Item = CKLineUnit>>>>,
    kl_datas: HashMap<KlType, CKLineList>,
    klu_cache: Vec<Option<CKLineUnit>>,
    klu_last_t: Vec<CTime>,
}

impl CChan {
    pub fn new(
        code: String,
        begin_time: Option<String>,
        end_time: Option<String>,
        data_src: DATA_SRC,
        lv_list: Option<Vec<KlType>>,
        config: Option<CChanConfig>,
        autype: AuType,
    ) -> Result<Self, CChanException> {
        let lv_list = lv_list.unwrap_or_else(|| vec![KlType::K_DAY, KlType::K_60M]);
        check_kltype_order(&lv_list)?;

        let conf = config.unwrap_or_else(CChanConfig::default);

        let mut chan = CChan {
            code,
            begin_time,
            end_time,
            autype,
            data_src,
            lv_list,
            conf,
            kl_misalign_cnt: 0,
            kl_inconsistent_detail: HashMap::new(),
            g_kl_iter: HashMap::new(),
            kl_datas: HashMap::new(),
            klu_cache: Vec::new(),
            klu_last_t: Vec::new(),
        };

        chan.do_init();

        if !chan.conf.trigger_step {
            chan.load()?;
        }

        Ok(chan)
    }

    fn do_init(&mut self) {
        self.kl_datas.clear();
        for lv in &self.lv_list {
            self.kl_datas
                .insert(*lv, CKLineList::new(*lv, self.conf.clone()));
        }
    }

    fn load_stock_data(
        &self,
        stockapi_instance: &dyn CCommonStockApi,
        lv: KlType,
    ) -> impl Iterator<Item = CKLineUnit> {
        stockapi_instance
            .get_kl_data()
            .enumerate()
            .map(move |(idx, mut klu)| {
                klu.set_idx(idx as i32);
                klu.kl_type = Some(lv);
                klu
            })
    }

    fn get_load_stock_iter(
        &self,
        stockapi_cls: &dyn CCommonStockApi,
        lv: KlType,
    ) -> Box<dyn Iterator<Item = CKLineUnit>> {
        let stockapi_instance = stockapi_cls.new(
            self.code.clone(),
            lv,
            self.begin_time.clone(),
            self.end_time.clone(),
            self.autype,
        );
        Box::new(self.load_stock_data(&stockapi_instance, lv))
    }

    fn add_lv_iter(&mut self, lv: KlType, iter: Box<dyn Iterator<Item = CKLineUnit>>) {
        self.g_kl_iter.entry(lv).or_insert_with(Vec::new).push(iter);
    }

    fn get_next_lv_klu(&mut self, lv: KlType) -> Option<CKLineUnit> {
        if let Some(iters) = self.g_kl_iter.get_mut(&lv) {
            while let Some(iter) = iters.first_mut() {
                if let Some(klu) = iter.next() {
                    return Some(klu);
                }
                iters.remove(0);
            }
        }
        None
    }

    pub fn step_load(&mut self) -> impl Iterator<Item = &Self> {
        assert!(self.conf.trigger_step);
        self.do_init();
        let skip_step = self.conf.skip_step;
        self.load().enumerate().filter_map(
            move |(idx, result)| {
                if idx < skip_step {
                    None
                } else {
                    Some(self)
                }
            },
        )
    }

    pub fn trigger_load(
        &mut self,
        inp: HashMap<KlType, Vec<CKLineUnit>>,
    ) -> Result<(), CChanException> {
        if self.klu_cache.is_empty() {
            self.klu_cache = vec![None; self.lv_list.len()];
        }
        if self.klu_last_t.is_empty() {
            self.klu_last_t = vec![CTime::new(1980, 1, 1, 0, 0); self.lv_list.len()];
        }

        for (lv_idx, lv) in self.lv_list.iter().enumerate() {
            if let Some(klu_list) = inp.get(lv) {
                let mut klu_list = klu_list.clone();
                for klu in &mut klu_list {
                    klu.kl_type = Some(*lv);
                }
                self.add_lv_iter(*lv, Box::new(klu_list.into_iter()));
            } else if lv_idx == 0 {
                return Err(CChanException::new(
                    &format!("最高级别{}没有传入数据", lv),
                    ErrCode::NoData,
                ));
            }
        }

        self.load_iterator(0, None, false)?;

        if !self.conf.trigger_step {
            for lv in &self.lv_list {
                self.kl_datas.get_mut(lv).unwrap().cal_seg_and_zs();
            }
        }

        Ok(())
    }

    fn init_lv_klu_iter(
        &mut self,
        stockapi_cls: &dyn CCommonStockApi,
    ) -> Result<(), CChanException> {
        let mut valid_lv_list = Vec::new();
        for lv in &self.lv_list {
            match self.get_load_stock_iter(stockapi_cls, *lv) {
                Ok(iter) => {
                    self.add_lv_iter(*lv, iter);
                    valid_lv_list.push(*lv);
                }
                Err(e) => {
                    if e.errcode == ErrCode::SrcDataNotFound && self.conf.auto_skip_illegal_sub_lv {
                        if self.conf.print_warning {
                            println!("[WARNING-{}]{}级别获取数据失败，跳过", self.code, lv);
                        }
                        self.kl_datas.remove(lv);
                    } else {
                        return Err(e);
                    }
                }
            }
        }
        self.lv_list = valid_lv_list;
        Ok(())
    }

    fn get_stock_api(&self) -> Box<dyn CCommonStockApi> {
        // This function would need to be implemented based on your specific needs
        // It should return the appropriate API based on self.data_src
        unimplemented!()
    }

    pub fn load(&mut self) -> Result<(), CChanException> {
        let stockapi_cls = self.get_stock_api();
        stockapi_cls.do_init();

        self.init_lv_klu_iter(&*stockapi_cls)?;

        self.klu_cache = vec![None; self.lv_list.len()];
        self.klu_last_t = vec![CTime::new(1980, 1, 1, 0, 0); self.lv_list.len()];

        self.load_iterator(0, None, false)?;

        if !self.conf.trigger_step {
            for lv in &self.lv_list {
                self.kl_datas.get_mut(lv).unwrap().cal_seg_and_zs();
            }
        }

        stockapi_cls.do_close();

        if self.kl_datas.get(&self.lv_list[0]).unwrap().is_empty() {
            return Err(CChanException::new(
                "最高级别没有获得任何数据",
                ErrCode::NoData,
            ));
        }

        Ok(())
    }

    fn set_klu_parent_relation(
        &mut self,
        parent_klu: &mut CKLineUnit,
        kline_unit: &mut CKLineUnit,
        cur_lv: KlType,
        lv_idx: usize,
    ) -> Result<(), CChanException> {
        if self.conf.kl_data_check
            && kltype_lte_day(cur_lv)
            && kltype_lte_day(self.lv_list[lv_idx - 1])
        {
            self.check_kl_consitent(parent_klu, kline_unit)?;
        }
        parent_klu.add_children(Rc::new(RefCell::new(kline_unit.clone())));
        kline_unit.set_parent(Rc::new(RefCell::new(parent_klu.clone())));
        Ok(())
    }

    fn add_new_kl(&mut self, cur_lv: KlType, kline_unit: CKLineUnit) -> Result<(), CChanException> {
        if let Some(kl_data) = self.kl_datas.get_mut(&cur_lv) {
            kl_data.add_single_klu(kline_unit)
        } else {
            Err(CChanException::new(
                &format!("Invalid KL type: {:?}", cur_lv),
                ErrCode::CommonError,
            ))
        }
    }

    fn try_set_klu_idx(&self, lv_idx: usize, kline_unit: &mut CKLineUnit) {
        if kline_unit.get_idx() >= 0 {
            return;
        }
        if let Some(kl_data) = self.kl_datas.get(&self.lv_list[lv_idx]) {
            if kl_data.is_empty() {
                kline_unit.set_idx(0);
            } else {
                kline_unit.set_idx(kl_data.last().unwrap().last().unwrap().get_idx() + 1);
            }
        }
    }

    fn load_iterator(
        &mut self,
        lv_idx: usize,
        parent_klu: Option<&mut CKLineUnit>,
        step: bool,
    ) -> Result<(), CChanException> {
        let cur_lv = self.lv_list[lv_idx];
        let mut pre_klu = None;
        if let Some(kl_data) = self.kl_datas.get(&cur_lv) {
            if !kl_data.is_empty() && !kl_data.last().unwrap().is_empty() {
                pre_klu = Some(kl_data.last().unwrap().last().unwrap().clone());
            }
        }

        loop {
            let mut kline_unit = if let Some(klu) = self.klu_cache[lv_idx].take() {
                klu
            } else {
                match self.get_next_lv_klu(cur_lv) {
                    Some(mut klu) => {
                        self.try_set_klu_idx(lv_idx, &mut klu);
                        if klu.time <= self.klu_last_t[lv_idx] {
                            return Err(CChanException::new(
                                &format!(
                                    "kline time err, cur={}, last={}",
                                    klu.time, self.klu_last_t[lv_idx]
                                ),
                                ErrCode::KlNotMonotonous,
                            ));
                        }
                        self.klu_last_t[lv_idx] = klu.time;
                        klu
                    }
                    None => break,
                }
            };

            if let Some(parent) = parent_klu {
                if kline_unit.time > parent.time {
                    self.klu_cache[lv_idx] = Some(kline_unit);
                    break;
                }
            }

            kline_unit.set_pre_klu(
                pre_klu
                    .as_ref()
                    .map(|klu| Rc::new(RefCell::new(klu.clone()))),
            );
            pre_klu = Some(kline_unit.clone());

            self.add_new_kl(cur_lv, kline_unit.clone())?;

            if let Some(parent) = parent_klu {
                self.set_klu_parent_relation(parent, &mut kline_unit, cur_lv, lv_idx)?;
            }

            if lv_idx != self.lv_list.len() - 1 {
                self.load_iterator(lv_idx + 1, Some(&mut kline_unit), step)?;
                self.check_kl_align(&kline_unit, lv_idx)?;
            }

            if lv_idx == 0 && step {
                // Here you might want to yield self, but Rust doesn't have Python's yield.
                // You might need to use a callback or return an iterator instead.
            }
        }

        Ok(())
    }

    fn check_kl_consitent(
        &mut self,
        parent_klu: &CKLineUnit,
        sub_klu: &CKLineUnit,
    ) -> Result<(), CChanException> {
        if parent_klu.time.year() != sub_klu.time.year()
            || parent_klu.time.month() != sub_klu.time.month()
            || parent_klu.time.day() != sub_klu.time.day()
        {
            self.kl_inconsistent_detail
                .entry(parent_klu.time.to_string())
                .or_insert_with(Vec::new)
                .push(sub_klu.time);
            if self.conf.print_warning {
                println!(
                    "[WARNING-{}]父级别时间是{}，次级别时间却是{}",
                    self.code, parent_klu.time, sub_klu.time
                );
            }
            if self.kl_inconsistent_detail.len() >= self.conf.max_kl_inconsistent_cnt {
                return Err(CChanException::new(
                    &format!(
                        "父&子级别K线时间不一致条数超过{}！！",
                        self.conf.max_kl_inconsistent_cnt
                    ),
                    ErrCode::KlTimeInconsistent,
                ));
            }
        }
        Ok(())
    }

    fn check_kl_align(
        &mut self,
        kline_unit: &CKLineUnit,
        lv_idx: usize,
    ) -> Result<(), CChanException> {
        if self.conf.kl_data_check && kline_unit.sub_kl_list.is_empty() {
            self.kl_misalign_cnt += 1;
            if self.conf.print_warning {
                println!(
                    "[WARNING-{}]当前{}没在次级别{}找到K线！！",
                    self.code,
                    kline_unit.time,
                    self.lv_list[lv_idx + 1]
                );
            }
            if self.kl_misalign_cnt >= self.conf.max_kl_misalgin_cnt {
                return Err(CChanException::new(
                    &format!(
                        "在次级别找不到K线条数超过{}！！",
                        self.conf.max_kl_misalgin_cnt
                    ),
                    ErrCode::KlDataNotAlign,
                ));
            }
        }
        Ok(())
    }

    pub fn get(&self, n: KlType) -> Option<&CKLineList> {
        self.kl_datas.get(&n)
    }

    pub fn get_bsp(&self, idx: Option<usize>) -> Vec<CBSPoint> {
        if let Some(idx) = idx {
            if let Some(kl_data) = self.kl_datas.get(&self.lv_list[idx]) {
                kl_data.bs_point_lst.lst.clone()
            } else {
                Vec::new()
            }
        } else {
            assert_eq!(self.lv_list.len(), 1);
            if let Some(kl_data) = self.kl_datas.get(&self.lv_list[0]) {
                kl_data.bs_point_lst.lst.clone()
            } else {
                Vec::new()
            }
        }
    }
}
