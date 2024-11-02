//
use crate::Bi::Bi::CBi;
use crate::Bi::BiConfig::CBiConfig;
use crate::Bi::BiList::CBiList;
use crate::BuySellPoint::BSPointList::CBSPointList;
use crate::ChanConfig::CChanConfig;
use crate::Common::types::Handle;
use crate::Common::CEnum::{BiDir, KLineDir, KlType, SegType};
use crate::Common::ChanException::{CChanException, ErrCode};
use crate::KLine::KLine::CKLine;
use crate::KLine::KLine_Unit::CKLineUnit;
use crate::Seg::linetype::Line;
use crate::Seg::Seg::CSeg;
use crate::Seg::SegConfig::CSegConfig;
use crate::Seg::SegListChan::CSegListChan;
use crate::ZS::ZSList::CZSList;
use crate::ZS::ZS::CZS;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::KLine_Unit::MetricModel;

pub struct CKLineList {
    pub kl_type: String,
    pub config: CChanConfig,
    pub lst: Vec<Handle<CKLine>>,
    pub bi_list: CBiList,
    pub seg_list: CSegListChan<CBi>,
    pub segseg_list: CSegListChan<CSeg<CBi>>,
    pub zs_list: CZSList<CBi>,
    pub segzs_list: CZSList<CSeg<CBi>>,
    pub bs_point_lst: CBSPointList<CBi>,
    pub seg_bs_point_lst: CBSPointList<CSeg<CBi>>,
    pub metric_model_lst: Vec<Box<dyn MetricModel>>,
    pub step_calculation: bool,
    pub bs_point_history: Vec<HashMap<String, String>>,
    pub seg_bs_point_history: Vec<HashMap<String, String>>,
}

impl CKLineList {
    pub fn new(kl_type: String, conf: CChanConfig) -> Self {
        let seg_list = CSegListChan::new(conf.seg_conf.clone(), SegType::Bi);
        let segseg_list = CSegListChan::new(conf.seg_conf.clone(), SegType::Seg);

        CKLineList {
            kl_type,
            config: conf.clone(),
            lst: Vec::new(),
            bi_list: CBiList::new(CBiConfig::default()),
            seg_list,
            segseg_list,
            zs_list: CZSList::new(Some(conf.zs_conf.clone())),
            segzs_list: CZSList::new(Some(conf.zs_conf.clone())),
            bs_point_lst: CBSPointList::new(conf.bs_point_conf.clone()),
            seg_bs_point_lst: CBSPointList::new(conf.seg_bs_point_conf.clone()),
            metric_model_lst: conf.get_metric_model(),
            step_calculation: true,
            bs_point_history: Vec::new(),
            seg_bs_point_history: Vec::new(),
        }
    }

    pub fn cal_seg_and_zs(&mut self) -> Result<(), CChanException> {
        if !self.step_calculation {
            self.bi_list
                .try_add_virtual_bi(self.lst.last().unwrap().clone(), false);
        }
        cal_seg(&mut self.bi_list, &mut self.seg_list);
        self.zs_list.cal_bi_zs(&self.bi_list, &mut self.seg_list);
        update_zs_in_seg(&self.bi_list, &mut self.seg_list.lst, &mut self.zs_list)?;

        cal_seg(&self.seg_list.lst, &mut self.segseg_list);
        self.segzs_list
            .cal_bi_zs(&self.seg_list.lst, &self.segseg_list);
        update_zs_in_seg(
            &self.seg_list.lst,
            &mut self.segseg_list.lst,
            &mut self.segzs_list,
        )?;

        self.seg_bs_point_lst
            .cal(&self.seg_list.lst, &self.segseg_list);
        self.bs_point_lst.cal(&self.bi_list, &self.seg_list);
        //self.record_current_bs_points();

        Ok(())
    }

    pub fn add_single_klu(&mut self, mut klu: CKLineUnit) -> Result<(), CChanException> {
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
    }

    //pub fn klu_iter(&self, klc_begin_idx: usize) -> impl Iterator<Item = &Handle<CKLineUnit>> {
    //    self.lst[klc_begin_idx..]
    //        .iter()
    //        .flat_map(|klc| klc.borrow().lst.iter())
    //}
}

pub fn cal_seg<U: Line>(
    bi_list: &[Handle<U>],
    seg_list: &mut CSegListChan<U>,
) -> Result<(), CChanException> {
    seg_list.update(bi_list);

    let mut sure_seg_cnt = 0;
    if seg_list.is_empty() {
        for bi in bi_list.iter() {
            bi.borrow_mut()._set_seg_idx(0);
        }
        return Ok(());
    }
    let mut begin_seg = seg_list.last().unwrap().clone();
    for seg in seg_list.iter().rev() {
        if seg.borrow().is_sure {
            sure_seg_cnt += 1;
        } else {
            sure_seg_cnt = 0;
        }
        begin_seg = seg.clone();
        if sure_seg_cnt > 2 {
            break;
        }
    }

    let mut cur_seg = seg_list.last().unwrap().clone();
    for bi in bi_list.iter().rev() {
        if bi.borrow()._seg_idx().is_some()
            && bi.borrow()._idx() < begin_seg.borrow().start_bi.borrow()._idx()
        {
            break;
        }
        if bi.borrow()._idx() > cur_seg.borrow().end_bi.borrow()._idx() {
            bi.borrow_mut()._set_seg_idx(cur_seg.borrow().idx + 1);
            continue;
        }
        if bi.borrow()._idx() < cur_seg.borrow().start_bi.borrow()._idx() {
            assert!(cur_seg.borrow().pre.is_some());
            let pre_seg = cur_seg.borrow().pre.as_ref().unwrap().clone();
            cur_seg = pre_seg;
        }
        bi.borrow_mut()._set_seg_idx(cur_seg.borrow().idx);
    }

    Ok(())
}

pub fn update_zs_in_seg<T: Line>(
    bi_list: &[Handle<T>],
    seg_list: &mut [Handle<CSeg<T>>], //CSegListChan<CBi>,
    zs_list: &mut [Handle<CZS<T>>],   //CZSList,
) -> Result<(), CChanException> {
    let mut sure_seg_cnt = 0;
    for seg in seg_list.iter().rev() {
        let mut seg = seg.borrow_mut();
        if seg.ele_inside_is_sure {
            break;
        }
        if seg.is_sure {
            sure_seg_cnt += 1;
        }
        seg.clear_zs_lst();
        for zs in zs_list.iter().rev() {
            let zs_ref = zs.borrow();
            if zs_ref.end.as_ref().unwrap().borrow().idx
                < seg.start_bi.borrow()._get_begin_klu().borrow().idx
            {
                break;
            }
            if zs_ref.is_inside(&seg) {
                seg.add_zs(Rc::clone(zs));
            }
            assert!(zs_ref.begin_bi.as_ref().unwrap().borrow()._idx() > 0);

            //let zs_ref = ;
            zs.borrow_mut().set_bi_in(
                bi_list[zs_ref.begin_bi.as_ref().unwrap().borrow()._idx() as usize - 1].clone(),
            );
            if zs.borrow_mut().end_bi.as_ref().unwrap().borrow()._idx() + 1 < bi_list.len() {
                zs.borrow_mut().set_bi_out(
                    bi_list[zs_ref.end_bi.as_ref().unwrap().borrow()._idx() as usize + 1].clone(),
                );
            }
            zs.borrow_mut().set_bi_lst(
                &bi_list[zs_ref.begin_bi.as_ref().unwrap().borrow()._idx()
                    ..=zs_ref.end_bi.as_ref().unwrap().borrow()._idx()]
                    .to_vec(),
            );
        }

        if sure_seg_cnt > 2 && !seg.ele_inside_is_sure {
            seg.ele_inside_is_sure = true;
        }
    }

    Ok(())
}

mod test {
    use std::time::Instant;

    use chrono::{Duration, NaiveDateTime};
    use rand::Rng;

    use crate::{ChanConfig::CChanConfig, Common::CTime::CTime, KLine::KLine_Unit::CKLineUnit};

    use super::CKLineList;

    #[test]
    fn test_insert_large_data() {
        // 创建一个随机数生成器
        let mut rng = rand::thread_rng();
        let start_time = NaiveDateTime::new(
            chrono::NaiveDate::from_ymd(2023, 1, 1),
            chrono::NaiveTime::from_hms(0, 0, 0),
        );

        let mut list = CKLineList::new("test".to_string(), CChanConfig::default());

        // 记录开始时间
        let start = Instant::now();

        for i in 0..10_000_000 {
            let time = start_time + Duration::minutes(i as i64);
            let time = CTime::from_naive_date_time(time, true, i as f64);
            let open_price: f64 = rng.gen_range(100.0..200.0);
            let high_price: f64 = rng.gen_range(open_price..200.0);
            let low_price: f64 = rng.gen_range(100.0..open_price);
            let close_price: f64 = rng.gen_range(low_price..high_price);
            let klu = CKLineUnit::new(time, open_price, high_price, low_price, close_price, false)
                .unwrap();
            list.add_single_klu(klu);
        }

        // 记录结束时间
        let end = start.elapsed();
        // 打印执行时间
        println!("耗时: {:?}", end); // 30s
    }
}
