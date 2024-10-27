use crate::Bi::BiList::CBiList;
use crate::BuySellPoint::BSPoint::CBSPoint;
use crate::BuySellPoint::BSPointList::CBSPointList;
use crate::Common::CEnum::{BiDir, KlineDir, SegType};
use crate::Common::ChanConfig::CChanConfig;
use crate::Common::ChanException::CChanException;
use crate::KLine::KLine::CKLine;
use crate::KLine::KLine_Unit::CKLineUnit;
use crate::Seg::Seg::CSeg;
use crate::Seg::SegListComm::CSegListComm;
use crate::Zs::ZSList::CZSList;
use crate::Zs::ZS::CZS;
use chrono::NaiveDateTime;
use serde::Serialize;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct CKLineList {
    pub kl_type: String,
    pub config: CChanConfig,
    pub lst: Vec<Rc<RefCell<CKLine>>>,
    pub bi_list: CBiList,
    pub seg_list: Rc<RefCell<CSegListComm<CBi>>>,
    pub segseg_list: Rc<RefCell<CSegListComm<CSeg<CBi>>>>,
    pub zs_list: CZSList,
    pub segzs_list: CZSList,
    pub bs_point_lst: CBSPointList<CBi, CBiList>,
    pub seg_bs_point_lst: CBSPointList<CSeg<CBi>, CSegListComm<CBi>>,
    pub metric_model_lst: Vec<Box<dyn MetricModel>>,
    pub step_calculation: bool,
    pub bs_point_history: Vec<HashMap<String, String>>,
    pub seg_bs_point_history: Vec<HashMap<String, String>>,
}

impl CKLineList {
    pub fn new(kl_type: String, conf: CChanConfig) -> Self {
        let seg_list = get_seglist_instance(Some(conf.seg_conf.clone()), SegType::Bi);
        let segseg_list = get_seglist_instance(Some(conf.seg_conf.clone()), SegType::Seg);

        CKLineList {
            kl_type,
            config: conf.clone(),
            lst: Vec::new(),
            bi_list: CBiList::new(Some(conf.bi_conf.clone())),
            seg_list,
            segseg_list,
            zs_list: CZSList::new(Some(conf.zs_conf.clone())),
            segzs_list: CZSList::new(Some(conf.zs_conf.clone())),
            bs_point_lst: CBSPointList::new(Some(conf.bs_point_conf.clone())),
            seg_bs_point_lst: CBSPointList::new(Some(conf.seg_bs_point_conf.clone())),
            metric_model_lst: conf.get_metric_model(),
            step_calculation: conf.trigger_step,
            bs_point_history: Vec::new(),
            seg_bs_point_history: Vec::new(),
        }
    }

    pub fn cal_seg_and_zs(&mut self) -> Result<(), CChanException> {
        if !self.step_calculation {
            self.bi_list
                .try_add_virtual_bi(self.lst.last().unwrap().clone())?;
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
    }

    pub fn add_single_klu(&mut self, klu: CKLineUnit) -> Result<(), CChanException> {
        klu.set_metric(&self.metric_model_lst);
        if self.lst.is_empty() {
            self.lst
                .push(Rc::new(RefCell::new(CKLine::new(klu, 0, None))));
        } else {
            let dir = self.lst.last().unwrap().borrow_mut().try_add(klu)?;
            if dir != KlineDir::Combine {
                let new_kline = Rc::new(RefCell::new(CKLine::new(
                    klu,
                    self.lst.len() as i32,
                    Some(dir),
                )));
                self.lst.push(new_kline.clone());
                if self.lst.len() >= 3 {
                    let len = self.lst.len();
                    self.lst[len - 2]
                        .borrow_mut()
                        .update_fx(&self.lst[len - 3], &self.lst[len - 1]);
                }
                if self.bi_list.update_bi(
                    &self.lst[self.lst.len() - 2],
                    &self.lst[self.lst.len() - 1],
                    self.step_calculation,
                )? && self.step_calculation
                {
                    self.cal_seg_and_zs()?;
                }
            } else if self.step_calculation
                && self
                    .bi_list
                    .try_add_virtual_bi(self.lst.last().unwrap().clone(), true)?
            {
                self.cal_seg_and_zs()?;
            }
        }
        Ok(())
    }

    pub fn klu_iter(&self, klc_begin_idx: usize) -> impl Iterator<Item = &CKLineUnit> {
        self.lst[klc_begin_idx..]
            .iter()
            .flat_map(|klc| klc.borrow().lst.iter())
    }

    pub fn to_dataframes(&self) -> HashMap<String, Vec<HashMap<String, String>>> {
        let mut dataframes = HashMap::new();

        // Convert segseg_list to DataFrame
        dataframes.insert(
            "segseg_list".to_string(),
            self.segseg_list
                .borrow()
                .iter()
                .map(|segseg| {
                    let segseg = segseg.borrow();
                    HashMap::from([
                        (
                            "begin_time".to_string(),
                            segseg.get_begin_klu().time.to_string(),
                        ),
                        (
                            "end_time".to_string(),
                            segseg.get_end_klu().time.to_string(),
                        ),
                        ("idx".to_string(), segseg.idx.to_string()),
                        ("dir".to_string(), format!("{:?}", segseg.dir)),
                        ("high".to_string(), segseg._high().to_string()),
                        ("low".to_string(), segseg._low().to_string()),
                        ("is_sure".to_string(), segseg.is_sure.to_string()),
                        (
                            "start_seg_idx".to_string(),
                            segseg
                                .start_bi
                                .as_ref()
                                .map_or("None".to_string(), |bi| bi.borrow().idx.to_string()),
                        ),
                        (
                            "end_seg_idx".to_string(),
                            segseg
                                .end_bi
                                .as_ref()
                                .map_or("None".to_string(), |bi| bi.borrow().idx.to_string()),
                        ),
                        ("zs_count".to_string(), segseg.zs_lst.len().to_string()),
                        ("bi_count".to_string(), segseg.bi_list.len().to_string()),
                        ("reason".to_string(), segseg.reason.clone()),
                    ])
                })
                .collect(),
        );

        // Convert zs_list to DataFrame
        dataframes.insert(
            "zs_list".to_string(),
            self.zs_list
                .iter()
                .map(|zs| {
                    let zs = zs.borrow();
                    HashMap::from([
                        (
                            "begin_time".to_string(),
                            zs.begin_bi.as_ref().map_or("None".to_string(), |bi| {
                                bi.borrow().get_begin_klu().time.to_string()
                            }),
                        ),
                        (
                            "end_time".to_string(),
                            zs.end_bi.as_ref().map_or("None".to_string(), |bi| {
                                bi.borrow().get_end_klu().time.to_string()
                            }),
                        ),
                        ("high".to_string(), zs.high.to_string()),
                        ("low".to_string(), zs.low.to_string()),
                        ("peak_high".to_string(), zs.peak_high.to_string()),
                        ("peak_low".to_string(), zs.peak_low.to_string()),
                        ("is_sure".to_string(), zs.is_sure.to_string()),
                        (
                            "begin_bi_idx".to_string(),
                            zs.begin_bi
                                .as_ref()
                                .map_or("None".to_string(), |bi| bi.borrow().idx.to_string()),
                        ),
                        (
                            "end_bi_idx".to_string(),
                            zs.end_bi
                                .as_ref()
                                .map_or("None".to_string(), |bi| bi.borrow().idx.to_string()),
                        ),
                        (
                            "bi_in".to_string(),
                            zs.bi_in
                                .as_ref()
                                .map_or("None".to_string(), |bi| bi.borrow().idx.to_string()),
                        ),
                        (
                            "bi_out".to_string(),
                            zs.bi_out
                                .as_ref()
                                .map_or("None".to_string(), |bi| bi.borrow().idx.to_string()),
                        ),
                        (
                            "begin_bi_time".to_string(),
                            zs.begin_bi.as_ref().map_or("None".to_string(), |bi| {
                                bi.borrow().get_begin_klu().time.to_string()
                            }),
                        ),
                        (
                            "end_bi_time".to_string(),
                            zs.end_bi.as_ref().map_or("None".to_string(), |bi| {
                                bi.borrow().get_begin_klu().time.to_string()
                            }),
                        ),
                        (
                            "bi_in_time".to_string(),
                            zs.bi_in.as_ref().map_or("None".to_string(), |bi| {
                                bi.borrow().get_begin_klu().time.to_string()
                            }),
                        ),
                        (
                            "bi_out_time".to_string(),
                            zs.bi_out.as_ref().map_or("None".to_string(), |bi| {
                                bi.borrow().get_begin_klu().time.to_string()
                            }),
                        ),
                    ])
                })
                .collect(),
        );

        // Convert segzs_list to DataFrame
        dataframes.insert(
            "segzs_list".to_string(),
            self.segzs_list
                .iter()
                .map(|segzs| {
                    let segzs = segzs.borrow();
                    HashMap::from([
                        (
                            "begin_time".to_string(),
                            segzs.begin_bi.as_ref().map_or("None".to_string(), |bi| {
                                bi.borrow().get_begin_klu().time.to_string()
                            }),
                        ),
                        (
                            "end_time".to_string(),
                            segzs.end_bi.as_ref().map_or("None".to_string(), |bi| {
                                bi.borrow().get_end_klu().time.to_string()
                            }),
                        ),
                        ("high".to_string(), segzs.high.to_string()),
                        ("low".to_string(), segzs.low.to_string()),
                        ("peak_high".to_string(), segzs.peak_high.to_string()),
                        ("peak_low".to_string(), segzs.peak_low.to_string()),
                        ("is_sure".to_string(), segzs.is_sure.to_string()),
                        (
                            "begin_seg_idx".to_string(),
                            segzs
                                .begin_bi
                                .as_ref()
                                .map_or("None".to_string(), |bi| bi.borrow().idx.to_string()),
                        ),
                        (
                            "end_seg_idx".to_string(),
                            segzs
                                .end_bi
                                .as_ref()
                                .map_or("None".to_string(), |bi| bi.borrow().idx.to_string()),
                        ),
                        (
                            "bi_in".to_string(),
                            segzs
                                .bi_in
                                .as_ref()
                                .map_or("None".to_string(), |bi| bi.borrow().idx.to_string()),
                        ),
                        (
                            "bi_out".to_string(),
                            segzs
                                .bi_out
                                .as_ref()
                                .map_or("None".to_string(), |bi| bi.borrow().idx.to_string()),
                        ),
                        (
                            "begin_bi_time".to_string(),
                            segzs.begin_bi.as_ref().map_or("None".to_string(), |bi| {
                                bi.borrow().get_begin_klu().time.to_string()
                            }),
                        ),
                        (
                            "end_bi_time".to_string(),
                            segzs.end_bi.as_ref().map_or("None".to_string(), |bi| {
                                bi.borrow().get_begin_klu().time.to_string()
                            }),
                        ),
                        (
                            "bi_in_time".to_string(),
                            segzs.bi_in.as_ref().map_or("None".to_string(), |bi| {
                                bi.borrow().get_begin_klu().time.to_string()
                            }),
                        ),
                        (
                            "bi_out_time".to_string(),
                            segzs.bi_out.as_ref().map_or("None".to_string(), |bi| {
                                bi.borrow().get_begin_klu().time.to_string()
                            }),
                        ),
                    ])
                })
                .collect(),
        );

        // Convert bs_point_lst to DataFrame
        dataframes.insert(
            "bs_point_lst".to_string(),
            self.bs_point_lst
                .iter()
                .map(|bsp| {
                    let bsp = bsp.borrow();
                    HashMap::from([
                        ("begin_time".to_string(), bsp.klu.time.to_string()),
                        ("bsp_type".to_string(), bsp.type2str()),
                        (
                            "bi_idx".to_string(),
                            bsp.bi
                                .as_ref()
                                .map_or("None".to_string(), |bi| bi.borrow().idx.to_string()),
                        ),
                        (
                            "bi_begin_time".to_string(),
                            bsp.bi.as_ref().map_or("None".to_string(), |bi| {
                                bi.borrow().get_begin_klu().time.to_string()
                            }),
                        ),
                        (
                            "bi_end_time".to_string(),
                            bsp.bi.as_ref().map_or("None".to_string(), |bi| {
                                bi.borrow().get_end_klu().time.to_string()
                            }),
                        ),
                    ])
                })
                .collect(),
        );

        // Convert seg_bs_point_lst to DataFrame
        dataframes.insert(
            "seg_bs_point_lst".to_string(),
            self.seg_bs_point_lst
                .iter()
                .map(|seg_bsp| {
                    let seg_bsp = seg_bsp.borrow();
                    HashMap::from([
                        ("begin_time".to_string(), seg_bsp.klu.time.to_string()),
                        ("bsp_type".to_string(), seg_bsp.type2str()),
                        (
                            "seg_idx".to_string(),
                            seg_bsp
                                .bi
                                .as_ref()
                                .map_or("None".to_string(), |bi| bi.borrow().idx.to_string()),
                        ),
                        (
                            "bi_begin_time".to_string(),
                            seg_bsp.bi.as_ref().map_or("None".to_string(), |bi| {
                                bi.borrow().get_begin_klu().time.to_string()
                            }),
                        ),
                        (
                            "bi_end_time".to_string(),
                            seg_bsp.bi.as_ref().map_or("None".to_string(), |bi| {
                                bi.borrow().get_end_klu().time.to_string()
                            }),
                        ),
                    ])
                })
                .collect(),
        );

        // Add historical bs_points
        dataframes.insert(
            "bs_point_history".to_string(),
            self.bs_point_history.clone(),
        );

        // Add historical seg_bs_points
        dataframes.insert(
            "seg_bs_point_history".to_string(),
            self.seg_bs_point_history.clone(),
        );

        dataframes
    }

    pub fn to_csv(&self, directory: &str) -> std::io::Result<()> {
        std::fs::create_dir_all(directory)?;

        let dataframes = self.to_dataframes();

        for (name, df) in dataframes.iter() {
            let file_path = format!("{}/{}.csv", directory, name);
            let mut wtr = csv::Writer::from_path(file_path)?;

            if !df.is_empty() {
                let headers: Vec<String> = df[0].keys().cloned().collect();
                wtr.write_record(&headers)?;

                for row in df {
                    let values: Vec<String> = headers
                        .iter()
                        .map(|h| row.get(h).unwrap_or(&String::new()).clone())
                        .collect();
                    wtr.write_record(&values)?;
                }
            }

            wtr.flush()?;
            println!("Saved {} to {}/{}.csv", name, directory, name);
        }

        Ok(())
    }

    fn record_current_bs_points(&mut self) {
        if let Some(latest_bsp) = self.bs_point_lst.last() {
            let latest_bsp = latest_bsp.borrow();
            self.bs_point_history.push(HashMap::from([
                ("begin_time".to_string(), latest_bsp.klu.time.to_string()),
                ("bsp_type".to_string(), latest_bsp.type2str()),
                ("is_buy".to_string(), latest_bsp.is_buy.to_string()),
                (
                    "relate_bsp1".to_string(),
                    latest_bsp
                        .relate_bsp1
                        .as_ref()
                        .map_or("None".to_string(), |bsp| bsp.borrow().klu.time.to_string()),
                ),
                (
                    "bi_idx".to_string(),
                    latest_bsp
                        .bi
                        .as_ref()
                        .map_or("None".to_string(), |bi| bi.borrow().idx.to_string()),
                ),
                (
                    "bi_begin_time".to_string(),
                    latest_bsp.bi.as_ref().map_or("None".to_string(), |bi| {
                        bi.borrow().get_begin_klu().time.to_string()
                    }),
                ),
                (
                    "bi_end_time".to_string(),
                    latest_bsp.bi.as_ref().map_or("None".to_string(), |bi| {
                        bi.borrow().get_end_klu().time.to_string()
                    }),
                ),
            ]));
        }

        if let Some(latest_seg_bsp) = self.seg_bs_point_lst.last() {
            let latest_seg_bsp = latest_seg_bsp.borrow();
            self.seg_bs_point_history.push(HashMap::from([
                (
                    "begin_time".to_string(),
                    latest_seg_bsp.klu.time.to_string(),
                ),
                ("bsp_type".to_string(), latest_seg_bsp.type2str()),
                ("is_buy".to_string(), latest_seg_bsp.is_buy.to_string()),
                (
                    "relate_bsp1".to_string(),
                    latest_seg_bsp
                        .relate_bsp1
                        .as_ref()
                        .map_or("None".to_string(), |bsp| bsp.borrow().klu.time.to_string()),
                ),
                (
                    "seg_idx".to_string(),
                    latest_seg_bsp
                        .bi
                        .as_ref()
                        .map_or("None".to_string(), |bi| bi.borrow().idx.to_string()),
                ),
                (
                    "bi_begin_time".to_string(),
                    latest_seg_bsp.bi.as_ref().map_or("None".to_string(), |bi| {
                        bi.borrow().get_begin_klu().time.to_string()
                    }),
                ),
                (
                    "bi_end_time".to_string(),
                    latest_seg_bsp.bi.as_ref().map_or("None".to_string(), |bi| {
                        bi.borrow().get_end_klu().time.to_string()
                    }),
                ),
            ]));
        }
    }
}

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
