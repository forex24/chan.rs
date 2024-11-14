use indexmap::IndexMap;

use crate::{
    AsHandle, Bar, CBSPointList, CBarList, CBi, CBiList, CBspPoint, CChanConfig, CSeg,
    CSegListChan, CZs, CZsList, Candle, CandleList, Handle, ICalcMetric, IParent, Indexable, Kline,
    LineType, SegType, ToHandle,
};

pub struct Analyzer {
    pub kl_type: i32,
    pub conf: CChanConfig,

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
            conf: conf.clone(),
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

    pub fn config(&self) -> &CChanConfig {
        &self.conf
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

#[allow(dead_code)]
fn update_bi_seg_idx<T: LineType + IParent>(bi_list: &mut [T], seg_list: &mut CSegListChan<T>) {
    // 原始版
    if seg_list.is_empty() {
        for bi in bi_list.iter_mut() {
            bi.set_seg_idx(0);
        }
        return;
    }

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
}

fn cal_seg<T: LineType + IParent + ToHandle + ICalcMetric>(
    bi_list: &mut [T],
    seg_list: &mut CSegListChan<T>,
) {
    seg_list.update(bi_list);

    update_bi_seg_idx2(bi_list, seg_list);
}

#[allow(dead_code)]
fn update_bi_seg_idx2<T: LineType + IParent + ToHandle>(
    bi_list: &mut [T],
    seg_list: &mut CSegListChan<T>,
) {
    // 性能优化版
    // 处理空线段列表的情况
    if seg_list.is_empty() {
        for bi in bi_list.iter_mut() {
            bi.set_seg_idx(0);
        }
        return;
    }

    let mut sure_seg_cnt = 0;

    // Find beginning segment
    let mut begin_seg = &seg_list[seg_list.len() - 1];
    for seg in seg_list.iter().rev() {
        if seg.is_sure {
            sure_seg_cnt += 1;
        } else {
            sure_seg_cnt = 0;
        }
        begin_seg = seg;
        if sure_seg_cnt > 2 {
            break;
        }
    }

    // Process bi_list in reverse
    let mut cur_seg = seg_list[seg_list.len() - 1].to_handle();

    for bi in bi_list.iter_mut().rev() {
        // Break if we've processed all relevant bis
        if bi.seg_idx().is_some() && bi.to_handle().index() < begin_seg.start_bi.index() {
            break;
        }

        // Handle bi index greater than current segment end
        if bi.to_handle().index() > cur_seg.end_bi.index() {
            bi.set_seg_idx(cur_seg.index() + 1);
            continue;
        }

        // Move to previous segment if necessary
        if bi.to_handle().index() < cur_seg.start_bi.index() {
            assert!(cur_seg.to_handle().prev().is_some());
            let pre = cur_seg
                .to_handle()
                .prev()
                .expect("Previous segment should exist");
            cur_seg = pre;
        }

        bi.set_seg_idx(cur_seg.to_handle().index());
    }
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
