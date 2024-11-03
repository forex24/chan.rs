pub enum BiType {
    CBi(CBi),
    CSeg(CSeg<CBi>), // 允许包含 CSeg<CBi>
}

pub enum SegType {
    CSeg(CSeg<CBi>),           // 当 BiType 为 CBi 时
    CSegCSeg(CSeg<CSeg<CBi>>), // 当 BiType 为 CSeg<CBi> 时
}

impl BiType {
    pub fn idx(&self) -> usize {
        match self {
            BiType::CBi(bi) => bi.idx(),
            BiType::CSeg(seg) => seg.idx(),
        }
    }

    pub fn high(&self) -> f64 {
        match self {
            BiType::CBi(bi) => bi.high(),
            BiType::CSeg(seg) => seg.high(),
        }
    }

    // 其他方法...
}

impl SegType {
    pub fn idx(&self) -> usize {
        match self {
            SegType::CSeg(seg) => seg.idx(),
            SegType::CSegCSeg(seg) => seg.idx(),
        }
    }

    pub fn high(&self) -> f64 {
        match self {
            SegType::CSeg(seg) => seg.high(),
            SegType::CSegCSeg(seg) => seg.high(),
        }
    }

    // 其他方法...
}

impl Line for BiType {
    type Parent = CSeg<CBi>; // 假设 BiType 的 Parent 是 CSeg<CBi>

    fn idx(&self) -> usize {
        match self {
            BiType::CBi(bi) => bi.idx(),
            BiType::CSeg(seg) => seg.idx(),
        }
    }

    fn high(&self) -> f64 {
        match self {
            BiType::CBi(bi) => bi.high(),
            BiType::CSeg(seg) => seg.high(),
        }
    }

    fn low(&self) -> f64 {
        match self {
            BiType::CBi(bi) => bi.low(),
            BiType::CSeg(seg) => seg.low(),
        }
    }

    fn get_begin_val(&self) -> f64 {
        match self {
            BiType::CBi(bi) => bi.get_begin_val(),
            BiType::CSeg(seg) => seg.get_begin_val(),
        }
    }

    fn get_end_val(&self) -> f64 {
        match self {
            BiType::CBi(bi) => bi.get_end_val(),
            BiType::CSeg(seg) => seg.get_end_val(),
        }
    }

    fn get_begin_klu(&self) -> Handle<CKLineUnit> {
        match self {
            BiType::CBi(bi) => bi.get_begin_klu(),
            BiType::CSeg(seg) => seg.get_begin_klu(),
        }
    }

    fn get_end_klu(&self) -> Handle<CKLineUnit> {
        match self {
            BiType::CBi(bi) => bi.get_end_klu(),
            BiType::CSeg(seg) => seg.get_end_klu(),
        }
    }

    fn dir(&self) -> BiDir {
        match self {
            BiType::CBi(bi) => bi.dir(),
            BiType::CSeg(seg) => seg.dir(),
        }
    }

    fn get_parent_seg(&self) -> Option<Handle<Self::Parent>> {
        match self {
            BiType::CBi(bi) => bi.get_parent_seg(),
            BiType::CSeg(seg) => seg.get_parent_seg(),
        }
    }

    fn set_parent_seg(&mut self, parent_seg: Option<Handle<Self::Parent>>) {
        match self {
            BiType::CBi(bi) => bi.set_parent_seg(parent_seg),
            BiType::CSeg(seg) => seg.set_parent_seg(parent_seg),
        }
    }

    fn seg_idx(&self) -> Option<usize> {
        match self {
            BiType::CBi(bi) => bi.seg_idx(),
            BiType::CSeg(seg) => seg.seg_idx(),
        }
    }

    fn set_seg_idx(&mut self, idx: usize) {
        match self {
            BiType::CBi(bi) => bi.set_seg_idx(idx),
            BiType::CSeg(seg) => seg.set_seg_idx(idx),
        }
    }

    fn set_pre(&mut self, pre: Option<Handle<Self>>) {
        match self {
            BiType::CBi(bi) => bi.set_pre(pre),
            BiType::CSeg(seg) => seg.set_pre(pre),
        }
    }

    fn set_next(&mut self, next: Option<Handle<Self>>) {
        match self {
            BiType::CBi(bi) => bi.set_next(next),
            BiType::CSeg(seg) => seg.set_next(next),
        }
    }

    fn get_begin_klc(&self) -> Handle<CKLine> {
        match self {
            BiType::CBi(bi) => bi.get_begin_klc(),
            BiType::CSeg(seg) => seg.get_begin_klc(),
        }
    }

    fn get_end_klc(&self) -> Handle<CKLine> {
        match self {
            BiType::CBi(bi) => bi.get_end_klc(),
            BiType::CSeg(seg) => seg.get_end_klc(),
        }
    }

    fn is_sure(&self) -> bool {
        match self {
            BiType::CBi(bi) => bi.is_sure(),
            BiType::CSeg(seg) => seg.is_sure(),
        }
    }

    fn next(&self) -> Option<Handle<Self>> {
        match self {
            BiType::CBi(bi) => bi.next(),
            BiType::CSeg(seg) => seg.next(),
        }
    }

    fn pre(&self) -> Option<Handle<Self>> {
        match self {
            BiType::CBi(bi) => bi.pre(),
            BiType::CSeg(seg) => seg.pre(),
        }
    }

    fn cal_macd_metric(
        &self,
        macd_algo: MacdAlgo,
        is_reverse: bool,
    ) -> Result<f64, CChanException> {
        match self {
            BiType::CBi(bi) => bi.cal_macd_metric(macd_algo, is_reverse),
            BiType::CSeg(seg) => seg.cal_macd_metric(macd_algo, is_reverse),
        }
    }

    fn set_bsp(&mut self, bsp: Option<Handle<CBSPoint<Self>>>)
    where
        Self: Sized,
    {
        match self {
            BiType::CBi(bi) => bi.set_bsp(bsp),
            BiType::CSeg(seg) => seg.set_bsp(bsp),
        }
    }

    fn amp(&self) -> Option<f64> {
        match self {
            BiType::CBi(bi) => bi.amp(),
            BiType::CSeg(seg) => seg.amp(),
        }
    }
}

impl Line for SegType {
    type Parent = CSeg<CBi>; // 假设 SegType 的 Parent 是 CSeg<CBi>

    fn idx(&self) -> usize {
        match self {
            SegType::CSeg(seg) => seg.idx(),
            SegType::CSegCSeg(seg) => seg.idx(),
        }
    }

    fn high(&self) -> f64 {
        match self {
            SegType::CSeg(seg) => seg.high(),
            SegType::CSegCSeg(seg) => seg.high(),
        }
    }

    fn low(&self) -> f64 {
        match self {
            SegType::CSeg(seg) => seg.low(),
            SegType::CSegCSeg(seg) => seg.low(),
        }
    }

    fn get_begin_val(&self) -> f64 {
        match self {
            SegType::CSeg(seg) => seg.get_begin_val(),
            SegType::CSegCSeg(seg) => seg.get_begin_val(),
        }
    }

    fn get_end_val(&self) -> f64 {
        match self {
            SegType::CSeg(seg) => seg.get_end_val(),
            SegType::CSegCSeg(seg) => seg.get_end_val(),
        }
    }

    fn get_begin_klu(&self) -> Handle<CKLineUnit> {
        match self {
            SegType::CSeg(seg) => seg.get_begin_klu(),
            SegType::CSegCSeg(seg) => seg.get_begin_klu(),
        }
    }

    fn get_end_klu(&self) -> Handle<CKLineUnit> {
        match self {
            SegType::CSeg(seg) => seg.get_end_klu(),
            SegType::CSegCSeg(seg) => seg.get_end_klu(),
        }
    }

    fn dir(&self) -> BiDir {
        match self {
            SegType::CSeg(seg) => seg.dir(),
            SegType::CSegCSeg(seg) => seg.dir(),
        }
    }

    fn get_parent_seg(&self) -> Option<Handle<Self::Parent>> {
        match self {
            SegType::CSeg(seg) => seg.get_parent_seg(),
            SegType::CSegCSeg(seg) => seg.get_parent_seg(),
        }
    }

    fn set_parent_seg(&mut self, parent_seg: Option<Handle<Self::Parent>>) {
        match self {
            SegType::CSeg(seg) => seg.set_parent_seg(parent_seg),
            SegType::CSegCSeg(seg) => seg.set_parent_seg(parent_seg),
        }
    }

    fn seg_idx(&self) -> Option<usize> {
        match self {
            SegType::CSeg(seg) => seg.seg_idx(),
            SegType::CSegCSeg(seg) => seg.seg_idx(),
        }
    }

    fn set_seg_idx(&mut self, idx: usize) {
        match self {
            SegType::CSeg(seg) => seg.set_seg_idx(idx),
            SegType::CSegCSeg(seg) => seg.set_seg_idx(idx),
        }
    }

    fn set_pre(&mut self, pre: Option<Handle<Self>>) {
        match self {
            SegType::CSeg(seg) => seg.set_pre(pre),
            SegType::CSegCSeg(seg) => seg.set_pre(pre),
        }
    }

    fn set_next(&mut self, next: Option<Handle<Self>>) {
        match self {
            SegType::CSeg(seg) => seg.set_next(next),
            SegType::CSegCSeg(seg) => seg.set_next(next),
        }
    }

    fn get_begin_klc(&self) -> Handle<CKLine> {
        match self {
            SegType::CSeg(seg) => seg.get_begin_klc(),
            SegType::CSegCSeg(seg) => seg.get_begin_klc(),
        }
    }

    fn get_end_klc(&self) -> Handle<CKLine> {
        match self {
            SegType::CSeg(seg) => seg.get_end_klc(),
            SegType::CSegCSeg(seg) => seg.get_end_klc(),
        }
    }

    fn is_sure(&self) -> bool {
        match self {
            SegType::CSeg(seg) => seg.is_sure(),
            SegType::CSegCSeg(seg) => seg.is_sure(),
        }
    }

    fn next(&self) -> Option<Handle<Self>> {
        match self {
            SegType::CSeg(seg) => seg.next(),
            SegType::CSegCSeg(seg) => seg.next(),
        }
    }

    fn pre(&self) -> Option<Handle<Self>> {
        match self {
            SegType::CSeg(seg) => seg.pre(),
            SegType::CSegCSeg(seg) => seg.pre(),
        }
    }

    fn cal_macd_metric(
        &self,
        macd_algo: MacdAlgo,
        is_reverse: bool,
    ) -> Result<f64, CChanException> {
        match self {
            SegType::CSeg(seg) => seg.cal_macd_metric(macd_algo, is_reverse),
            SegType::CSegCSeg(seg) => seg.cal_macd_metric(macd_algo, is_reverse),
        }
    }

    fn set_bsp(&mut self, bsp: Option<Handle<CBSPoint<Self>>>)
    where
        Self: Sized,
    {
        match self {
            SegType::CSeg(seg) => seg.set_bsp(bsp),
            SegType::CSegCSeg(seg) => seg.set_bsp(bsp),
        }
    }

    fn amp(&self) -> Option<f64> {
        match self {
            SegType::CSeg(seg) => seg.amp(),
            SegType::CSegCSeg(seg) => seg.amp(),
        }
    }
}
