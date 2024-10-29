//
use crate::Bi::Bi::CBi;
use crate::Bi::BiConfig::CBiConfig;
use crate::Bi::BiList::CBiList;
//use crate::BuySellPoint::BSPointList::CBSPointList;
//use crate::ChanConfig::CChanConfig;
use crate::Common::types::Handle;
use crate::Common::CEnum::{BiDir, KlType, KlineDir, SegType};
use crate::Common::ChanException::{CChanException, ErrCode};
use crate::KLine::KLine::CKLine;
use crate::KLine::KLine_Unit::CKLineUnit;
//use crate::Seg::Seg::CSeg;
//use crate::Seg::SegListChan::CSegListChan;
//use crate::Seg::SegListComm::CSegListComm;
//use crate::ZS::ZSList::CZSList;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::KLine_Unit::MetricModel;

pub struct CKLineList {
    pub kl_type: String,
    //pub config: CChanConfig,
    pub lst: Vec<Handle<CKLine>>,
    pub bi_list: CBiList,
    //pub seg_list: Handle<CSegListComm<CBi>>,
    //pub segseg_list: Handle<CSegListComm<CSeg<CBi>>>,
    //pub zs_list: CZSList,
    //pub segzs_list: CZSList,
    //pub bs_point_lst: CBSPointList<CBi, CBiList>,
    //pub seg_bs_point_lst: CBSPointList<CSeg<CBi>, CSegListComm<CBi>>,
    //pub metric_model_lst: Vec<Box<dyn MetricModel>>,
    //pub step_calculation: bool,
    //pub bs_point_history: Vec<HashMap<String, String>>,
    //pub seg_bs_point_history: Vec<HashMap<String, String>>,
}

impl CKLineList {
    pub fn new(kl_type: String) -> Self {
        //}, conf: CChanConfig) -> Self {
        //let seg_list = CSegListChan::new(Some(conf.seg_conf.clone()), SegType::Bi);
        //let segseg_list = CSegListChan::new(Some(conf.seg_conf.clone()), SegType::Seg);

        CKLineList {
            kl_type,
            //config: conf.clone(),
            lst: Vec::new(),
            bi_list: CBiList::new(CBiConfig::default()),
            //seg_list,
            //segseg_list,
            //zs_list: CZSList::new(Some(conf.zs_conf.clone())),
            //segzs_list: CZSList::new(Some(conf.zs_conf.clone())),
            //bs_point_lst: CBSPointList::new(Some(conf.bs_point_conf.clone())),
            //seg_bs_point_lst: CBSPointList::new(Some(conf.seg_bs_point_conf.clone())),
            //metric_model_lst: conf.get_metric_model(),
            //step_calculation: conf.trigger_step,
            //bs_point_history: Vec::new(),
            //seg_bs_point_history: Vec::new(),
        }
    }

    /*pub fn cal_seg_and_zs(&mut self) -> Result<(), CChanException> {
        if !self.step_calculation {
            self.bi_list
                .try_add_virtual_bi(self.lst.last().unwrap().clone(), false)?;
        }
        cal_seg(&self.bi_list, &mut self.seg_list.borrow_mut())?;
        self.zs_list
            .cal_bi_zs(&self.bi_list, &self.seg_list.borrow())?;
        update_zs_in_seg(
            &self.bi_list,
            &mut self.seg_list.borrow_mut(),
            &mut self.zs_list,
        )?;

        cal_seg(&self.seg_list.borrow(), &mut self.segseg_list.borrow_mut())?;
        self.segzs_list
            .cal_bi_zs(&self.seg_list.borrow(), &self.segseg_list.borrow())?;
        update_zs_in_seg(
            &self.seg_list.borrow(),
            &mut self.segseg_list.borrow_mut(),
            &mut self.segzs_list,
        )?;

        self.seg_bs_point_lst
            .cal(&self.seg_list.borrow(), &self.segseg_list.borrow())?;
        self.bs_point_lst
            .cal(&self.bi_list, &self.seg_list.borrow())?;
        self.record_current_bs_points();

        Ok(())
    }*/

    pub fn add_single_klu(&mut self, klu: CKLineUnit) -> Result<(), CChanException> {
        //klu.set_metric(&self.metric_model_lst);
        let klu = Rc::new(RefCell::new(klu));
        if self.lst.is_empty() {
            self.lst.push(CKLine::new(Rc::clone(&klu), 0, KlineDir::Up));
        } else {
            let dir = CKLine::try_add(self.lst.last().as_ref().unwrap(), &klu)?;
            if dir != KlineDir::Combine {
                let new_kline = CKLine::new(Rc::clone(&klu), self.lst.len() as i32, dir);
                self.lst.push(new_kline.clone());
                if self.lst.len() >= 3 {
                    let len = self.lst.len();
                    CKLine::update_fx(&self.lst[len - 2], &self.lst[len - 3], &self.lst[len - 1]);
                }
                self.bi_list.update_bi(
                    Rc::clone(&self.lst[self.lst.len() - 2]),
                    Rc::clone(&self.lst[self.lst.len() - 1]),
                    true, //self.step_calculation,
                ); //&& self.step_calculation
                   //{
                   //    self.cal_seg_and_zs()?;
                   //}
            } /*else if self.step_calculation
                  && self
                      .bi_list
                      .try_add_virtual_bi(self.lst.last().unwrap().clone(), true)?
              {
                  self.cal_seg_and_zs()?;
              }*/
        }
        Ok(())
    }

    //pub fn klu_iter(&self, klc_begin_idx: usize) -> impl Iterator<Item = &Handle<CKLineUnit>> {
    //    self.lst[klc_begin_idx..]
    //        .iter()
    //        .flat_map(|klc| klc.borrow().lst.iter())
    //}
}
/*
pub fn cal_seg(bi_list: &CBiList, seg_list: &mut CSegListComm<CBi>) -> Result<(), CChanException> {
    seg_list.update(bi_list)?;

    let mut sure_seg_cnt = 0;
    if seg_list.is_empty() {
        for bi in bi_list.iter() {
            bi.borrow_mut().set_seg_idx(0);
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
        if bi.borrow().seg_idx.is_some()
            && bi.borrow().idx < begin_seg.borrow().start_bi.borrow().idx
        {
            break;
        }
        if bi.borrow().idx > cur_seg.borrow().end_bi.borrow().idx {
            bi.borrow_mut().set_seg_idx(cur_seg.borrow().idx + 1);
            continue;
        }
        if bi.borrow().idx < cur_seg.borrow().start_bi.borrow().idx {
            assert!(cur_seg.borrow().pre.is_some());
            cur_seg = cur_seg.borrow().pre.as_ref().unwrap().clone();
        }
        bi.borrow_mut().set_seg_idx(cur_seg.borrow().idx);
    }

    Ok(())
}

pub fn update_zs_in_seg(
    bi_list: &CBiList,
    seg_list: &mut CSegListComm<CBi>,
    zs_list: &mut CZSList,
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
            let mut zs = zs.borrow_mut();
            if zs.end.borrow().idx < seg.start_bi.borrow().get_begin_klu().idx {
                break;
            }
            if zs.is_inside(&seg) {
                seg.add_zs(zs.clone());
            }
            assert!(zs.begin_bi.as_ref().unwrap().borrow().idx > 0);
            zs.set_bi_in(bi_list[zs.begin_bi.as_ref().unwrap().borrow().idx as usize - 1].clone());
            if zs.end_bi.as_ref().unwrap().borrow().idx + 1 < bi_list.len() as i32 {
                zs.set_bi_out(
                    bi_list[zs.end_bi.as_ref().unwrap().borrow().idx as usize + 1].clone(),
                );
            }
            zs.set_bi_lst(
                bi_list[zs.begin_bi.as_ref().unwrap().borrow().idx as usize
                    ..=zs.end_bi.as_ref().unwrap().borrow().idx as usize]
                    .to_vec(),
            );
        }

        if sure_seg_cnt > 2 && !seg.ele_inside_is_sure {
            seg.ele_inside_is_sure = true;
        }
    }

    Ok(())
}
*/

mod test {
    use std::time::Instant;

    use chrono::{Duration, NaiveDateTime};
    use rand::Rng;

    use crate::{Common::CTime::CTime, KLine::KLine_Unit::CKLineUnit};

    use super::CKLineList;

    #[test]
    fn test_insert_large_data() {
        // 创建一个随机数生成器
        let mut rng = rand::thread_rng();
        let start_time = NaiveDateTime::new(
            chrono::NaiveDate::from_ymd(2023, 1, 1),
            chrono::NaiveTime::from_hms(0, 0, 0),
        );

        let mut list = CKLineList::new("test".to_string());

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
