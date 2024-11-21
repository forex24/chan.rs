use std::{
    fs::{self, File},
    path::Path,
};

use indexmap::IndexMap;

use crate::{Analyzer, IParent, Indexable, LineType};
use std::io::Write;

pub const TIME_FORMAT: &str = "%Y-%m-%d %H:%M";

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
                //map.insert("seg_idx".to_string(), bi.seg_idx.unwrap_or(0).to_string());
                map.insert(
                    "seg_idx".to_string(),
                    bi.seg_idx().map_or("".to_string(), |idx| idx.to_string()),
                );
                //map.insert(
                //    "parent_seg".to_string(),
                //    bi.parent_seg_idx().unwrap_or(0).to_string(),
                //);
                map.insert("begin_klc".to_string(), bi.begin_klc.index().to_string());
                map.insert("end_klc".to_string(), bi.end_klc.index().to_string());
                map.insert("begin_val".to_string(), bi._get_begin_val().to_string());
                map.insert("end_val".to_string(), bi._get_end_val().to_string());
                map.insert("klu_cnt".to_string(), bi._get_klu_cnt().to_string());
                map.insert("klc_cnt".to_string(), bi._get_klc_cnt().to_string());
                map.insert(
                    "parent_seg_idx".to_string(),
                    bi.parent_seg_idx()
                        .map_or("".to_string(), |idx| idx.to_string()),
                );
                map.insert(
                    "parent_seg_dir".to_string(),
                    bi.parent_seg_dir()
                        .map_or("".to_string(), |idx| idx.to_string()),
                );
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
                map.insert(
                    "parent_seg_idx".to_string(),
                    seg.parent_seg_idx()
                        .map_or("".to_string(), |idx| idx.to_string()),
                );
                map.insert(
                    "parent_seg_dir".to_string(),
                    seg.parent_seg_dir()
                        .map_or("".to_string(), |dir| dir.to_string()),
                );
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
                map.insert("sub_zs_count".to_string(), zs.sub_zs_lst.len().to_string());
                map
            })
            .collect();
        dataframes.insert("zs_list".to_string(), zs_list);

        // BS Point List
        let bs_point_list = self
            .bs_point_lst.lst
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
                map.insert(
                    "start_seg_idx".to_string(),
                    seg.start_bi.index().to_string(),
                );
                map.insert("end_seg_idx".to_string(), seg.end_bi.index().to_string());
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
                map.insert("begin_seg_idx".to_string(), zs.begin_bi.index().to_string());
                map.insert(
                    "end_seg_idx".to_string(),
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
                map.insert("sub_zs_count".to_string(), zs.sub_zs_lst.len().to_string());
                map
            })
            .collect();
        dataframes.insert("seg_zs_list".to_string(), seg_zs_list);

        // Seg BS Point List
        let seg_bs_point_list = self
            .seg_bs_point_lst.lst
            .iter()
            .map(|bsp| {
                let mut map = IndexMap::new();
                map.insert(
                    "begin_time".to_string(),
                    bsp.klu.time.format(TIME_FORMAT).to_string(),
                );
                map.insert("bsp_type".to_string(), bsp.type2str());
                map.insert("seg_idx".to_string(), bsp.bi.index().to_string());
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

        /*
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
        */
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

        //dataframes.insert(
        //    "bs_point_history_no_dup".to_string(),
        //    bs_point_history_no_dup,
        //);
        //
        //// Add historical seg_bs_points
        //dataframes.insert(
        //    "seg_bs_point_history_no_dup".to_string(),
        //    seg_bs_point_history_no_dup,
        //);

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

    pub(crate) fn record_last_bs_points(&mut self) {
        if let Some(latest_bsp) = self.bs_point_lst.lst.last() {
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

    pub(crate) fn record_last_seg_bs_points(&mut self) {
        if let Some(latest_seg_bsp) = self.seg_bs_point_lst.lst.last() {
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
