use std::{
    fs::{self, File},
    path::Path,
};

use indexmap::IndexMap;

use crate::{
    AsHandle, Bar, CBSPointList, CBarList, CBi, CBiList, CBspPoint, CChanConfig, CSeg,
    CSegListChan, CZs, CZsList, Candle, CandleList, Handle, ICalcMetric, IParent, Indexable, Kline,
    LineType, SegType, ToHandle,
};
use std::io::Write;

pub const TIME_FORMAT: &str = "%Y-%m-%d %H:%M";

pub struct Analyzer {
    pub kl_type: i32,

    pub bar_list: CBarList,
    pub candle_list: CandleList,
    //bi
    pub bi_list: CBiList,
    pub seg_list: CSegListChan<CBi>,
    pub zs_list: CZsList<CBi>,
    pub bs_point_lst: CBSPointList<CBi>,
    // segseg
    pub segseg_list: CSegListChan<CBiSeg>,
    pub segzs_list: CZsList<CBiSeg>,
    pub seg_bs_point_lst: CBSPointList<CBiSeg>,
    // history bsp
    pub bs_point_history: Vec<IndexMap<String, String>>,
    pub seg_bs_point_history: Vec<IndexMap<String, String>>,
    pub last_bsp: Option<Handle<CBspPoint<CBi>>>,
    pub last_seg_bsp: Option<Handle<CBspPoint<CSeg<CBi>>>>,
}

pub type CBiSeg = CSeg<CBi>;

impl Analyzer {
    pub fn new(kl_type: i32, conf: CChanConfig) -> Self {
        Self {
            kl_type,
            bar_list: CBarList::new(),
            candle_list: CandleList::new(),
            bi_list: CBiList::new(conf.bi_conf),
            seg_list: CSegListChan::new(conf.seg_conf, SegType::Bi),
            segseg_list: CSegListChan::new(conf.seg_conf, SegType::Seg),
            zs_list: CZsList::new(conf.zs_conf),
            segzs_list: CZsList::new(conf.zs_conf),
            bs_point_lst: CBSPointList::new(conf.bs_point_conf),
            seg_bs_point_lst: CBSPointList::new(conf.seg_bs_point_conf),
            bs_point_history: Vec::new(),
            seg_bs_point_history: Vec::new(),
            last_bsp: None,
            last_seg_bsp: None,
        }
    }
    // seg
    pub fn seg_bsp_list(&self) -> &[Handle<CBspPoint<CSeg<CBi>>>] {
        &self.seg_bs_point_lst.lst
    }

    pub fn seg_zs_list(&self) -> &[CZs<CSeg<CBi>>] {
        self.segzs_list.as_slice()
    }

    pub fn seg_seg_list(&self) -> &[CSeg<CSeg<CBi>>] {
        self.segseg_list.as_slice()
    }

    // bi
    pub fn bi_bsp_list(&self) -> &[Handle<CBspPoint<CBi>>] {
        &self.bs_point_lst.lst
    }

    pub fn bi_zs_list(&self) -> &[CZs<CBi>] {
        self.zs_list.as_slice()
    }

    pub fn seg_list(&self) -> &[CSeg<CBi>] {
        self.seg_list.as_slice()
    }

    pub fn bi_list(&self) -> &[CBi] {
        self.bi_list.as_slice()
    }

    pub fn candle_list(&self) -> &[Candle] {
        self.candle_list.as_slice()
    }

    pub fn bar_list(&self) -> &[Bar] {
        self.bar_list.as_slice()
    }

    // main entry
    pub fn add_k(&mut self, k: &Kline) {
        let klu = self.bar_list.add_kline(k);
        if self.candle_list.update_candle(klu) {
            if self.bi_list.update_bi(
                &self.candle_list[self.candle_list.len() - 2],
                &self.candle_list[self.candle_list.len() - 1],
                true,
            ) {
                self.cal_seg_and_zs();
            }
        } else if self
            .bi_list
            .try_add_virtual_bi(&self.candle_list[self.candle_list.len() - 1], true)
        {
            self.cal_seg_and_zs();
        }
    }

    fn cal_bi_seg_and_zs(&mut self) {
        // bi
        cal_seg(self.bi_list.as_mut_slice(), &mut self.seg_list);

        self.zs_list
            .cal_bi_zs(self.bi_list.as_slice(), &self.seg_list);

        // 计算seg的zs_lst，以及中枢的bi_in, bi_out
        update_zs_in_seg(
            self.bi_list.as_slice(),
            &mut self.seg_list,
            &mut self.zs_list,
        );
    }

    fn cal_seg_seg_and_zs(&mut self) {
        // seg
        cal_seg(self.seg_list.as_mut_slice(), &mut self.segseg_list);

        self.segzs_list
            .cal_bi_zs(self.seg_list.as_slice(), &self.segseg_list);

        // 计算segseg的zs_lst，以及中枢的bi_in, bi_out
        update_zs_in_seg(
            self.seg_list.as_slice(),
            &mut self.segseg_list,
            &mut self.segzs_list,
        );
    }

    fn cal_bsp(&mut self) {
        // 计算买卖点
        // 线段线段买卖点
        self.seg_bs_point_lst
            .cal(self.seg_list.as_slice(), &self.segseg_list);

        // 再算笔买卖点
        self.bs_point_lst
            .cal(self.bi_list.as_slice(), &self.seg_list);
    }
    fn cal_seg_and_zs(&mut self) {
        self.cal_bi_seg_and_zs();

        self.cal_seg_seg_and_zs();
        // 计算每一笔里面的 klc列表
        //self.update_klc_in_bi();

        self.cal_bsp();

        self.record_last_bs_points();
        self.record_last_seg_bs_points();

        // 这里有点问题，是因为klu.time是相同的，但是bsp_type不同
        // 同时也是不改python代码
        /*if let Some(last) = self.bs_point_lst.last() {
            if self.last_bsp.as_ref().map_or(true, |saved| {
                last.klu.time != saved.klu.time
            }) {
                self.last_bsp = Some(last.clone());
                self.record_last_bs_points();
            }
        }

        if let Some(last) = self.seg_bs_point_lst.last() {
            if self.last_seg_bsp.as_ref().map_or(true, |saved| {
                last.klu.time != saved.klu.time
            }) {
                self.last_seg_bsp = Some(last.clone());
                self.record_last_seg_bs_points();
            }
        }*/
    }

    //fn update_klc_in_bi(&mut self) {
    //    for bi in self.bi_list.iter_mut() {
    //        bi.set_klc_lst(&self.candle_list[bi.begin_klc.index()..=bi.end_klc.index()]);
    //    }
    //}
}

impl Analyzer {
    // storage funciton
    pub fn to_dataframes(&self) -> IndexMap<String, Vec<IndexMap<String, String>>> {
        let mut dataframes = IndexMap::new();

        // KLine List
        let kline_list = self
            .candle_list
            .iter()
            .map(|kl| {
                let mut map = IndexMap::new();
                map.insert(
                    "begin_time".to_string(),
                    kl.time_begin.format(TIME_FORMAT).to_string(),
                );
                map.insert(
                    "end_time".to_string(),
                    kl.time_end.format(TIME_FORMAT).to_string(),
                );
                map.insert("idx".to_string(), kl.index().to_string());
                map.insert("dir".to_string(), kl.dir.to_string());
                map.insert("high".to_string(), kl.high.to_string());
                map.insert("low".to_string(), kl.low.to_string());
                map.insert("fx".to_string(), kl.fx_type.to_string());
                map
            })
            .collect();
        dataframes.insert("kline_list".to_string(), kline_list);

        // Bi List
        let bi_list = self
            .bi_list
            .iter()
            .map(|bi| {
                let mut map = IndexMap::new();
                map.insert(
                    "begin_time".to_string(),
                    bi.get_begin_klu().time.format(TIME_FORMAT).to_string(),
                );
                map.insert(
                    "end_time".to_string(),
                    bi.get_end_klu().time.format(TIME_FORMAT).to_string(),
                );
                map.insert("idx".to_string(), bi.index().to_string());
                map.insert("dir".to_string(), bi.dir.to_string());
                map.insert("high".to_string(), bi._high().to_string());
                map.insert("low".to_string(), bi._low().to_string());
                map.insert("type".to_string(), bi.bi_type.to_string());
                map.insert(
                    "is_sure".to_string(),
                    if bi.is_sure {
                        "True".to_string()
                    } else {
                        "False".to_string()
                    },
                );
                map.insert("seg_idx".to_string(), bi.seg_idx.unwrap_or(0).to_string());
                map.insert(
                    "parent_seg".to_string(),
                    bi.parent_seg_idx().unwrap_or(0).to_string(),
                );
                map.insert("begin_klc".to_string(), bi.begin_klc.index().to_string());
                map.insert("end_klc".to_string(), bi.end_klc.index().to_string());
                map.insert("begin_val".to_string(), bi._get_begin_val().to_string());
                map.insert("end_val".to_string(), bi._get_end_val().to_string());
                map.insert("klu_cnt".to_string(), bi._get_klu_cnt().to_string());
                map.insert("klc_cnt".to_string(), bi._get_klc_cnt().to_string());
                map
            })
            .collect();
        dataframes.insert("bi_list".to_string(), bi_list);

        // Seg List
        let seg_list = self
            .seg_list
            .iter()
            .map(|seg| {
                let mut map = IndexMap::new();
                map.insert(
                    "begin_time".to_string(),
                    seg.get_begin_klu().time.format(TIME_FORMAT).to_string(),
                );
                map.insert(
                    "end_time".to_string(),
                    seg.get_end_klu().time.format(TIME_FORMAT).to_string(),
                );
                map.insert("idx".to_string(), seg.index().to_string());
                map.insert("dir".to_string(), seg.dir.to_string());
                map.insert("high".to_string(), seg._high().to_string());
                map.insert("low".to_string(), seg._low().to_string());
                map.insert(
                    "is_sure".to_string(),
                    if seg.is_sure {
                        "True".to_string()
                    } else {
                        "False".to_string()
                    },
                );
                map.insert("start_bi_idx".to_string(), seg.start_bi.index().to_string());
                map.insert("end_bi_idx".to_string(), seg.end_bi.index().to_string());
                map.insert("zs_count".to_string(), seg.zs_lst.len().to_string());
                map.insert("bi_count".to_string(), seg.bi_list.len().to_string());
                map.insert("reason".to_string(), seg.reason.clone());
                map
            })
            .collect();
        dataframes.insert("seg_list".to_string(), seg_list);

        // ZS List
        let zs_list = self
            .zs_list
            .iter()
            .map(|zs| {
                let mut map = IndexMap::new();
                map.insert(
                    "begin_time".to_string(),
                    zs.begin_bi
                        .get_begin_klu()
                        .time
                        .format(TIME_FORMAT)
                        .to_string(),
                );
                map.insert(
                    "end_time".to_string(),
                    zs.end_bi
                        .unwrap()
                        ._get_end_klu()
                        .time
                        .format(TIME_FORMAT)
                        .to_string(),
                );
                map.insert("high".to_string(), zs.high.to_string());
                map.insert("low".to_string(), zs.low.to_string());
                map.insert("peak_high".to_string(), zs.peak_high.to_string());
                map.insert("peak_low".to_string(), zs.peak_low.to_string());
                map.insert(
                    "is_sure".to_string(),
                    if zs.is_sure {
                        "True".to_string()
                    } else {
                        "False".to_string()
                    },
                );
                map.insert("begin_bi_idx".to_string(), zs.begin_bi.index().to_string());
                map.insert(
                    "end_bi_idx".to_string(),
                    zs.end_bi.unwrap().index().to_string(),
                );
                map.insert(
                    "bi_in".to_string(),
                    zs.bi_in
                        .as_ref()
                        .map_or("".to_string(), |bi| bi.index().to_string()),
                );
                map.insert(
                    "bi_out".to_string(),
                    zs.bi_out
                        .as_ref()
                        .map_or("".to_string(), |bi| bi.index().to_string()),
                );
                map
            })
            .collect();
        dataframes.insert("zs_list".to_string(), zs_list);

        // BS Point List
        let bs_point_list = self
            .bs_point_lst
            .iter()
            .map(|bsp| {
                let mut map = IndexMap::new();
                map.insert(
                    "begin_time".to_string(),
                    bsp.klu.time.format(TIME_FORMAT).to_string(),
                );
                map.insert("bsp_type".to_string(), bsp.type2str());
                map.insert("bi_idx".to_string(), bsp.bi.index().to_string());
                map.insert(
                    "bi_begin_time".to_string(),
                    bsp.bi.get_begin_klu().time.format(TIME_FORMAT).to_string(),
                );
                map.insert(
                    "bi_end_time".to_string(),
                    bsp.bi.get_end_klu().time.format(TIME_FORMAT).to_string(),
                );
                map.insert(
                    "relate_bsp1_time".to_string(),
                    bsp.relate_bsp1.as_ref().map_or("None".to_string(), |bsp| {
                        bsp.klu.time.format(TIME_FORMAT).to_string()
                    }),
                );
                map
            })
            .collect();
        dataframes.insert("bs_point_lst".to_string(), bs_point_list);

        // SegSeg List
        let seg_seg_list = self
            .segseg_list
            .iter()
            .map(|seg| {
                let mut map = IndexMap::new();
                map.insert(
                    "begin_time".to_string(),
                    seg.get_begin_klu().time.format(TIME_FORMAT).to_string(),
                );
                map.insert(
                    "end_time".to_string(),
                    seg.get_end_klu().time.format(TIME_FORMAT).to_string(),
                );
                map.insert("idx".to_string(), seg.index().to_string());
                map.insert("dir".to_string(), seg.dir.to_string());
                map.insert("high".to_string(), seg._high().to_string());
                map.insert("low".to_string(), seg._low().to_string());
                map.insert(
                    "is_sure".to_string(),
                    if seg.is_sure {
                        "True".to_string()
                    } else {
                        "False".to_string()
                    },
                );
                map.insert("start_bi_idx".to_string(), seg.start_bi.index().to_string());
                map.insert("end_bi_idx".to_string(), seg.end_bi.index().to_string());
                map.insert("zs_count".to_string(), seg.zs_lst.len().to_string());
                map.insert("bi_count".to_string(), seg.bi_list.len().to_string());
                map.insert("reason".to_string(), seg.reason.clone());
                map
            })
            .collect();
        dataframes.insert("seg_seg_list".to_string(), seg_seg_list);

        // SegZS List
        let seg_zs_list = self
            .segzs_list
            .iter()
            .map(|zs| {
                let mut map = IndexMap::new();
                map.insert(
                    "begin_time".to_string(),
                    zs.begin_bi
                        .get_begin_klu()
                        .time
                        .format(TIME_FORMAT)
                        .to_string(),
                );
                map.insert(
                    "end_time".to_string(),
                    zs.end_bi
                        .unwrap()
                        ._get_end_klu()
                        .time
                        .format(TIME_FORMAT)
                        .to_string(),
                );
                map.insert("high".to_string(), zs.high.to_string());
                map.insert("low".to_string(), zs.low.to_string());
                map.insert("peak_high".to_string(), zs.peak_high.to_string());
                map.insert("peak_low".to_string(), zs.peak_low.to_string());
                map.insert(
                    "is_sure".to_string(),
                    if zs.is_sure {
                        "True".to_string()
                    } else {
                        "False".to_string()
                    },
                );
                map.insert("begin_bi_idx".to_string(), zs.begin_bi.index().to_string());
                map.insert(
                    "end_bi_idx".to_string(),
                    zs.end_bi.unwrap().index().to_string(),
                );
                map.insert(
                    "bi_in".to_string(),
                    zs.bi_in
                        .as_ref()
                        .map_or("".to_string(), |bi| bi.index().to_string()),
                );
                map.insert(
                    "bi_out".to_string(),
                    zs.bi_out
                        .as_ref()
                        .map_or("".to_string(), |bi| bi.index().to_string()),
                );
                map
            })
            .collect();
        dataframes.insert("seg_zs_list".to_string(), seg_zs_list);

        // Seg BS Point List
        let seg_bs_point_list = self
            .seg_bs_point_lst
            .iter()
            .map(|bsp| {
                let mut map = IndexMap::new();
                map.insert(
                    "begin_time".to_string(),
                    bsp.klu.time.format(TIME_FORMAT).to_string(),
                );
                map.insert("bsp_type".to_string(), bsp.type2str());
                map.insert("bi_idx".to_string(), bsp.bi.index().to_string());
                map.insert(
                    "bi_begin_time".to_string(),
                    bsp.bi.get_begin_klu().time.format(TIME_FORMAT).to_string(),
                );
                map.insert(
                    "bi_end_time".to_string(),
                    bsp.bi.get_end_klu().time.format(TIME_FORMAT).to_string(),
                );
                map.insert(
                    "relate_bsp1_time".to_string(),
                    bsp.relate_bsp1.as_ref().map_or("None".to_string(), |bsp| {
                        bsp.klu.time.format(TIME_FORMAT).to_string()
                    }),
                );
                map
            })
            .collect();
        dataframes.insert("seg_bs_point_lst".to_string(), seg_bs_point_list);

        let bs_point_history_no_dup = self
            .bs_point_lst
            .history
            .iter()
            .map(|bsp| {
                let mut map = IndexMap::new();
                map.insert(
                    "begin_time".to_string(),
                    bsp.klu.time.format(TIME_FORMAT).to_string(),
                );
                map.insert("idx".to_string(), bsp.index().to_string());
                map.insert("bsp_type".to_string(), bsp.type2str());
                map.insert("is_buy".to_string(), bsp.is_buy.to_string());
                map.insert(
                    "relate_bsp1".to_string(),
                    bsp.relate_bsp1.as_ref().map_or("None".to_string(), |bsp| {
                        bsp.klu.time.format(TIME_FORMAT).to_string()
                    }),
                );
                map.insert("bi_idx".to_string(), bsp.bi.index().to_string());
                map.insert(
                    "bi_begin_time".to_string(),
                    bsp.bi.get_begin_klu().time.format(TIME_FORMAT).to_string(),
                );
                map.insert(
                    "bi_end_time".to_string(),
                    bsp.bi.get_end_klu().time.format(TIME_FORMAT).to_string(),
                );
                map
            })
            .collect();

        let seg_bs_point_history_no_dup = self
            .seg_bs_point_lst
            .history
            .iter()
            .map(|bsp| {
                let mut map = IndexMap::new();
                map.insert(
                    "begin_time".to_string(),
                    bsp.klu.time.format(TIME_FORMAT).to_string(),
                );
                map.insert("idx".to_string(), bsp.index().to_string());
                map.insert("bsp_type".to_string(), bsp.type2str());
                map.insert("is_buy".to_string(), bsp.is_buy.to_string());
                map.insert(
                    "relate_bsp1".to_string(),
                    bsp.relate_bsp1.as_ref().map_or("None".to_string(), |bsp| {
                        bsp.klu.time.format(TIME_FORMAT).to_string()
                    }),
                );
                map.insert("bi_idx".to_string(), bsp.bi.index().to_string());
                map.insert(
                    "bi_begin_time".to_string(),
                    bsp.bi.get_begin_klu().time.format(TIME_FORMAT).to_string(),
                );
                map.insert(
                    "bi_end_time".to_string(),
                    bsp.bi.get_end_klu().time.format(TIME_FORMAT).to_string(),
                );
                map
            })
            .collect();

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

        dataframes.insert(
            "bs_point_history_no_dup".to_string(),
            bs_point_history_no_dup,
        );

        // Add historical seg_bs_points
        dataframes.insert(
            "seg_bs_point_history_no_dup".to_string(),
            seg_bs_point_history_no_dup,
        );

        dataframes
    }

    pub fn to_csv(&self, directory: &str) -> std::io::Result<()> {
        fs::create_dir_all(directory)?;

        let dataframes = self.to_dataframes();

        for (name, data) in &dataframes {
            let file_path = Path::new(directory).join(format!("{}.csv", name));
            let mut file = File::create(file_path)?;

            // Write headers
            if let Some(first_row) = data.first() {
                let headers: Vec<String> = first_row.keys().cloned().collect();
                writeln!(file, "{}", headers.join(","))?;

                // Write data rows
                for row in data {
                    let values: Vec<String> = headers
                        .iter()
                        .map(|key| row.get(key).unwrap_or(&String::new()).clone())
                        .collect();
                    writeln!(file, "{}", values.join(","))?;
                }
            }
        }

        Ok(())
    }

    fn record_last_bs_points(&mut self) {
        if let Some(latest_bsp) = self.bs_point_lst.last() {
            //let latest_bsp = latest_bsp;
            self.bs_point_history.push(IndexMap::from([
                (
                    "begin_time".to_string(),
                    latest_bsp.klu.time.format(TIME_FORMAT).to_string(),
                ),
                ("bsp_type".to_string(), latest_bsp.type2str()),
                ("is_buy".to_string(), latest_bsp.is_buy.to_string()),
                (
                    "relate_bsp1".to_string(),
                    latest_bsp
                        .relate_bsp1
                        .as_ref()
                        .map_or("None".to_string(), |bsp| {
                            bsp.klu.time.format(TIME_FORMAT).to_string()
                        }),
                ),
                ("bi_idx".to_string(), latest_bsp.bi.index().to_string()),
                (
                    "bi_begin_time".to_string(),
                    latest_bsp
                        .bi
                        .get_begin_klu()
                        .time
                        .format(TIME_FORMAT)
                        .to_string(),
                ),
                (
                    "bi_end_time".to_string(),
                    latest_bsp
                        .bi
                        .get_end_klu()
                        .time
                        .format(TIME_FORMAT)
                        .to_string(),
                ),
            ]));
        }
    }

    fn record_last_seg_bs_points(&mut self) {
        if let Some(latest_seg_bsp) = self.seg_bs_point_lst.last() {
            //let latest_seg_bsp = latest_seg_bsp;
            self.seg_bs_point_history.push(IndexMap::from([
                (
                    "begin_time".to_string(),
                    latest_seg_bsp.klu.time.format(TIME_FORMAT).to_string(),
                ),
                ("bsp_type".to_string(), latest_seg_bsp.type2str()),
                ("is_buy".to_string(), latest_seg_bsp.is_buy.to_string()),
                (
                    "relate_bsp1".to_string(),
                    latest_seg_bsp
                        .relate_bsp1
                        .as_ref()
                        .map_or("None".to_string(), |bsp| {
                            bsp.klu.time.format(TIME_FORMAT).to_string()
                        }),
                ),
                ("seg_idx".to_string(), latest_seg_bsp.bi.index().to_string()),
                (
                    "bi_begin_time".to_string(),
                    latest_seg_bsp
                        .bi
                        .get_begin_klu()
                        .time
                        .format(TIME_FORMAT)
                        .to_string(),
                ),
                (
                    "bi_end_time".to_string(),
                    latest_seg_bsp
                        .bi
                        .get_end_klu()
                        .time
                        .format(TIME_FORMAT)
                        .to_string(),
                ),
            ]));
        }
    }
}

fn update_bi_seg_idx<T: LineType + IParent>(bi_list: &mut [T], seg_list: &mut CSegListChan<T>) {
    if !seg_list.is_empty() {
        //计算每一笔属于哪个线段
        for seg in seg_list.iter() {
            for bi in bi_list[seg.start_bi.index()..=seg.end_bi.index()].iter_mut() {
                bi.set_seg_idx(seg.index());
            }
        }

        // 最后一个线段最后一笔之后的笔都算是最后一个线段的
        for bi in bi_list
            .iter_mut()
            .skip(seg_list.last().unwrap().end_bi.index() + 1)
        {
            bi.set_seg_idx(seg_list.len());
        }

        //第一个线段起始笔之前的笔都算是第一个线段的
        for bi in bi_list[0..seg_list.lst[0].start_bi.index()].iter_mut() {
            bi.set_seg_idx(0);
        }
    } else {
        for bi in bi_list.iter_mut() {
            bi.set_seg_idx(0);
        }
    }
}

fn cal_seg<T: LineType + IParent + ToHandle + ICalcMetric>(
    bi_list: &mut [T],
    seg_list: &mut CSegListChan<T>,
) {
    seg_list.update(bi_list);

    update_bi_seg_idx(bi_list, seg_list);
}

fn update_zs_in_seg<T: LineType + IParent + ToHandle + ICalcMetric>(
    bi_list: &[T],
    seg_list: &mut CSegListChan<T>,
    zs_list: &mut CZsList<T>,
) {
    let mut sure_seg_cnt = 0;
    for seg in seg_list.iter_mut().rev() {
        if seg.ele_inside_is_sure {
            break;
        }
        if seg.is_sure {
            sure_seg_cnt += 1;
        }
        seg.clear_zs_lst();
        for zs in zs_list.iter_mut().rev() {
            assert!(zs.end.is_some());
            if zs.end.unwrap().index() < seg.start_bi.get_begin_klu().as_handle().index() {
                break;
            }
            if zs.is_inside(seg) {
                seg.add_zs(zs.as_handle());
            }
            //assert!(zs.begin_bi.index() > 0);
            zs.set_bi_in(bi_list[zs.begin_bi.index() - 1].to_handle());
            if zs.end_bi.unwrap().index() + 1 < bi_list.len() {
                zs.set_bi_out(bi_list[zs.end_bi.unwrap().index() + 1].to_handle());
            }
            zs.set_bi_lst(&bi_list[zs.begin_bi.index()..=zs.end_bi.unwrap().index()]);
        }

        if sure_seg_cnt > 2 && !seg.ele_inside_is_sure {
            seg.ele_inside_is_sure = true;
        }
    }
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new(1, CChanConfig::default())
    }
}
