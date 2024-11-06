use crate::Common::{
    handle::Handle,
    CEnum::{BiDir, LeftSegMethod, SegType},
    ChanException::{CChanException, ErrCode},
};

use super::{linetype::Line, EigenFX::CEigenFX, Seg::CSeg, SegConfig::CSegConfig};

pub struct CSegListChan<T: Line> {
    pub lst: Box<Vec<CSeg<T>>>,
    pub lv: SegType,
    pub config: CSegConfig,
}

impl<T: Line> CSegListChan<T> {
    pub fn new(seg_config: CSegConfig, lv: SegType) -> Self {
        let mut instance = Self {
            lst: Box::<Vec<CSeg<T>>>::default(),
            lv,
            config: seg_config,
        };
        instance.do_init();
        instance
    }

    // 已完备
    pub fn do_init(&mut self) {
        // 删除末尾不确定的线段
        while !self.lst.is_empty() && !self.lst.last().unwrap().is_sure {
            let _seg = self.lst.last().unwrap();

            for bi in &_seg.bi_list {
                bi.as_mut().line_set_parent_seg(None);
            }
            if let Some(pre) = &_seg.pre {
                pre.as_mut().next = None;
            }
            self.lst.pop();
        }

        if !self.lst.is_empty() {
            assert!(
                self.lst.last().unwrap().eigen_fx.is_some()
                    && self.lst.last().unwrap().eigen_fx.as_ref().unwrap().ele[2].is_some()
            );
            if !self.lst.last().unwrap().eigen_fx.as_ref().unwrap().ele[2]
                .as_ref()
                .unwrap()
                .lst
                .last()
                .unwrap()
                .line_is_sure()
            {
                // 如果确定线段的分形的第三元素包含不确定笔，也需要重新算，不然线段分形元素的高低点可能不对
                self.lst.pop();
            }
        }
    }

    // 已完备
    pub fn update(&mut self, bi_lst: &[Handle<T>]) {
        self.do_init();
        if self.lst.is_empty() {
            self.cal_seg_sure(bi_lst, 0);
        } else {
            let begin_idx = self.lst.last().unwrap().end_bi.line_idx() + 1;
            self.cal_seg_sure(bi_lst, begin_idx);
        }
        self.collect_left_seg(bi_lst);
    }

    // 已完备
    fn cal_seg_sure(&mut self, bi_lst: &[T], begin_idx: usize) {
        let fx_eigen = self.cal_seg_sure_inner(bi_lst.iter(), begin_idx);
        if let Some(fx_eigen) = fx_eigen {
            self.treat_fx_eigen(fx_eigen, bi_lst);
        }
    }

    // 已完备
    fn cal_seg_sure_inner<'a, I>(&mut self, bi_list: I, begin_idx: usize) -> Option<CEigenFX<T>>
    where
        I: Iterator<Item = &'a T>,
        T: 'a, // Add this bound to ensure T lives at least as long as 'a
    {
        let mut up_eigen = CEigenFX::new(BiDir::Up, false, self.lv);
        let mut down_eigen = CEigenFX::new(BiDir::Down, false, self.lv);
        let mut last_seg_dir = if self.lst.is_empty() {
            None
        } else {
            Some(self.lst.last().unwrap().dir)
        };

        for bi in bi_list.skip(begin_idx) {
            let mut fx_eigen_dir = None;
            match (bi.line_dir(), last_seg_dir) {
                (BiDir::Down, Some(BiDir::Down) | None) => {
                    if up_eigen.add(bi.to_handle()) {
                        fx_eigen_dir = Some(BiDir::Up);
                    }
                }
                (BiDir::Up, Some(BiDir::Up) | None) => {
                    if down_eigen.add(bi.to_handle()) {
                        fx_eigen_dir = Some(BiDir::Down);
                    }
                }
                _ => {}
            }

            if self.lst.is_empty() {
                // 尝试确定第一段方向，不要以谁先成为分形来决定
                if up_eigen.ele[1].is_some() && bi.line_is_down() {
                    last_seg_dir = Some(BiDir::Down);
                    down_eigen.clear();
                } else if down_eigen.ele[1].is_some() && bi.line_is_up() {
                    up_eigen.clear();
                    last_seg_dir = Some(BiDir::Up);
                }

                if up_eigen.ele[1].is_none()
                    && last_seg_dir == Some(BiDir::Down)
                    && bi.line_dir() == BiDir::Down
                {
                    last_seg_dir = None;
                } else if down_eigen.ele[1].is_none()
                    && last_seg_dir == Some(BiDir::Up)
                    && bi.line_dir() == BiDir::Up
                {
                    last_seg_dir = None;
                }
            }

            if let Some(dir) = fx_eigen_dir {
                match dir {
                    BiDir::Up => return Some(up_eigen),
                    BiDir::Down => return Some(down_eigen),
                }
            }
        }
        None
    }

    // 已完备
    fn treat_fx_eigen(&mut self, mut fx_eigen: CEigenFX<T>, bi_lst: &[Handle<T>]) {
        let _test = fx_eigen.can_be_end(bi_lst);
        let end_bi_idx = fx_eigen.get_peak_bi_idx();

        match _test {
            Some(true) | None => {
                // None表示反向分型找到尾部也没找到
                let is_true = _test.is_some(); // 如果是正常结束
                if !self.add_new_seg(
                    bi_lst,
                    end_bi_idx,
                    is_true && fx_eigen.all_bi_is_sure(), // 防止第一根线段的方向与首尾值异常
                    None,
                    true,
                    "normal",
                ) {
                    self.cal_seg_sure(bi_lst, end_bi_idx + 1);
                    return;
                }
                self.lst.last_mut().unwrap().eigen_fx = Some(fx_eigen);
                if is_true {
                    self.cal_seg_sure(bi_lst, end_bi_idx + 1);
                }
            }
            Some(false) => {
                // 从第0笔开始计算失败，尝试从第一笔开始计算
                self.cal_seg_sure(bi_lst, fx_eigen.lst[1].line_idx());
            }
        }
    }
}

// 以下为Common部分
impl<T: Line> CSegListChan<T> {
    // 已完备
    pub fn left_bi_break(&self, bi_lst: &[Handle<T>]) -> bool {
        // 最后一个确定线段之后的笔有突破该线段最后一笔的
        if self.lst.is_empty() {
            return false;
        }
        let last_seg_end_bi = &self.lst.last().unwrap().end_bi;
        for bi in bi_lst.iter().skip(last_seg_end_bi.line_idx() + 1) {
            if last_seg_end_bi.line_is_up() && bi.line_high() > last_seg_end_bi.line_high() {
                return true;
            } else if last_seg_end_bi.line_is_down() && bi.line_low() < last_seg_end_bi.line_low() {
                return true;
            }
        }
        false
    }

    // 已完备
    pub fn collect_first_seg(&mut self, bi_lst: &[Handle<T>]) {
        if bi_lst.len() < 3 {
            return;
        }
        match self.config.left_method {
            LeftSegMethod::Peak => {
                let _high = bi_lst
                    .iter()
                    .map(|bi| bi.line_high())
                    .fold(f64::MIN, f64::max);
                let _low = bi_lst
                    .iter()
                    .map(|bi| bi.line_low())
                    .fold(f64::MAX, f64::min);
                let first_val = bi_lst.first().unwrap().line_get_begin_val();

                if (_high - first_val).abs() >= (_low - first_val).abs() {
                    assert!(find_peak_bi(bi_lst.iter(), true).is_some());
                    if let Some(peak_bi) = find_peak_bi(bi_lst.iter(), true) {
                        self.add_new_seg(
                            bi_lst,
                            peak_bi.line_idx(),
                            false,
                            Some(BiDir::Up),
                            false,
                            "0seg_find_high",
                        );
                    }
                } else {
                    assert!(find_peak_bi(bi_lst.iter(), false).is_some());
                    if let Some(peak_bi) = find_peak_bi(bi_lst.iter(), false) {
                        self.add_new_seg(
                            bi_lst,
                            peak_bi.line_idx(),
                            false,
                            Some(BiDir::Down),
                            false,
                            "0seg_find_low",
                        );
                    }
                }
                self.collect_left_as_seg(bi_lst);
            }
            LeftSegMethod::All => {
                let _dir = if bi_lst.last().unwrap().line_get_end_val()
                    >= bi_lst[0].line_get_begin_val()
                {
                    BiDir::Up
                } else {
                    BiDir::Down
                };
                self.add_new_seg(
                    bi_lst,
                    bi_lst.last().unwrap().line_idx(),
                    false,
                    Some(_dir),
                    false,
                    "0seg_collect_all",
                );
            }
        }
    }

    // 99% 已完备，注意FIXME
    pub fn collect_left_seg_peak_method(
        &mut self,
        last_seg_end_bi: Handle<T>,
        bi_lst: &[Handle<T>],
    ) {
        if last_seg_end_bi.line_is_down() {
            if let Some(peak_bi) =
                find_peak_bi(bi_lst[last_seg_end_bi.line_idx() + 3..].iter(), true)
            {
                if peak_bi.line_idx() - last_seg_end_bi.line_idx() >= 3 {
                    self.add_new_seg(
                        bi_lst,
                        peak_bi.line_idx(),
                        false,
                        Some(BiDir::Up),
                        true,
                        "collectleft_find_high",
                    );
                }
            }
        } else if let Some(peak_bi) =
            find_peak_bi(bi_lst[last_seg_end_bi.line_idx() + 3..].iter(), false)
        {
            if peak_bi.line_idx() - last_seg_end_bi.line_idx() >= 3 {
                self.add_new_seg(
                    bi_lst,
                    peak_bi.line_idx(),
                    false,
                    Some(BiDir::Down),
                    true,
                    "collectleft_find_low",
                );
            }
        }
        //FIXME: python last_seg_end_bi = self[-1].end_bi
        self.collect_left_as_seg(bi_lst);
    }

    // 99% 已完备，理由同上
    pub fn collect_segs(&mut self, bi_lst: &[Handle<T>]) {
        let last_bi = bi_lst.last().unwrap();
        let last_seg_end_bi = self.lst.last().unwrap().end_bi.clone();

        if last_bi.line_idx() - last_seg_end_bi.line_idx() < 3 {
            return;
        }

        if last_seg_end_bi.line_is_down()
            && last_bi.line_get_end_val() <= last_seg_end_bi.line_get_end_val()
        {
            if let Some(peak_bi) =
                find_peak_bi(bi_lst[last_seg_end_bi.line_idx() + 3..].iter(), true)
            {
                self.add_new_seg(
                    bi_lst,
                    peak_bi.line_idx(),
                    false,
                    Some(BiDir::Up),
                    true,
                    "collectleft_find_high_force",
                );
                self.collect_left_seg(bi_lst);
            }
        } else if last_seg_end_bi.line_is_up()
            && last_bi.line_get_end_val() >= last_seg_end_bi.line_get_end_val()
        {
            if let Some(peak_bi) =
                find_peak_bi(bi_lst[last_seg_end_bi.line_idx() + 3..].iter(), false)
            {
                self.add_new_seg(
                    bi_lst,
                    peak_bi.line_idx(),
                    false,
                    Some(BiDir::Down),
                    true,
                    "collectleft_find_low_force",
                );
                self.collect_left_seg(bi_lst);
            }
        }
        //  剩下线段的尾部相比于最后一个线段的尾部，高低关系和最后一个虚线段的方向一致
        else if self.config.left_method == LeftSegMethod::All {
            // 容易找不到二类买卖点！！
            self.collect_left_as_seg(bi_lst);
        } else if self.config.left_method == LeftSegMethod::Peak {
            self.collect_left_seg_peak_method(last_seg_end_bi, bi_lst);
        }
    }

    // 已完备
    pub fn collect_left_seg(&mut self, bi_lst: &[Handle<T>]) {
        if self.lst.is_empty() {
            self.collect_first_seg(bi_lst);
        } else {
            self.collect_segs(bi_lst);
        }
    }

    // 已完备
    pub fn collect_left_as_seg(&mut self, bi_lst: &[Handle<T>]) {
        let last_bi = bi_lst.last().unwrap();
        let last_seg_end_bi = self.lst.last().unwrap().end_bi.clone();

        if last_seg_end_bi.line_idx() + 1 >= bi_lst.len() {
            return;
        }

        if last_seg_end_bi.line_dir() == last_bi.line_dir() {
            self.add_new_seg(
                bi_lst,
                last_bi.line_idx() - 1,
                false,
                None,
                true,
                "collect_left_1",
            );
        } else {
            self.add_new_seg(
                bi_lst,
                last_bi.line_idx(),
                false,
                None,
                true,
                "collect_left_0",
            );
        }
    }
    pub fn exist_sure_seg(&self) -> bool {
        self.lst.iter().any(|seg| seg.is_sure)
    }
}

// 以下要特别注意
impl<T: Line> CSegListChan<T> {
    // 已完备
    pub fn try_add_new_seg(
        &mut self,
        bi_lst: &[T],
        end_bi_idx: usize,
        is_sure: bool,
        seg_dir: Option<BiDir>,
        split_first_seg: bool,
        reason: &str,
    ) -> Result<(), CChanException> {
        if self.lst.is_empty() && split_first_seg && end_bi_idx >= 3 {
            if let Some(peak_bi) = find_peak_bi(
                bi_lst[..end_bi_idx - 2].iter().rev(), //TODO: 需要仔细分析，潜在bug，FindPeakBi(bi_lst[end_bi_idx-3::-1], bi_lst[end_bi_idx].is_down())
                bi_lst[end_bi_idx].line_is_down(),
            ) {
                if (peak_bi.line_is_down()
                    && (peak_bi.line_low() < bi_lst[0].line_low() || peak_bi.line_idx() == 0))
                    || (peak_bi.line_is_up()
                        && (peak_bi.line_high() > bi_lst[0].line_high() || peak_bi.line_idx() == 0))
                {
                    // 要比第一笔开头还高/低（因为没有比较到）
                    self.add_new_seg(
                        bi_lst,
                        peak_bi.line_idx(),
                        false,
                        Some(peak_bi.line_dir()),
                        true,
                        "split_first_1st",
                    );
                    self.add_new_seg(bi_lst, end_bi_idx, false, None, true, "split_first_2nd");
                    return Ok(());
                }
            }
        }

        let bi1_idx = if self.lst.is_empty() {
            0
        } else {
            self.lst.last().unwrap().end_bi.line_idx() + 1
        };
        let bi1 = bi_lst[bi1_idx].clone();
        let bi2 = bi_lst[end_bi_idx].clone();

        let new_seg = CSeg::new(
            &self.lst,
            self.lst.len(),
            bi1,
            bi2,
            is_sure,
            seg_dir,
            reason,
        )?;

        if !self.lst.is_empty() {
            let last_seg = self.lst.last().unwrap();
            last_seg.next = Some(new_seg.as_handle());
            new_seg.pre = Some(last_seg);
        }

        new_seg.update_bi_list(bi_lst, bi1_idx, end_bi_idx);
        self.lst.push(new_seg);

        Ok(())
    }

    // 已完备
    pub fn add_new_seg(
        &mut self,
        bi_lst: &[Handle<T>],
        end_bi_idx: usize,
        is_sure: bool,
        seg_dir: Option<BiDir>,
        split_first_seg: bool,
        reason: &str,
    ) -> bool {
        match self.try_add_new_seg(
            bi_lst,
            end_bi_idx,
            is_sure,
            seg_dir,
            split_first_seg,
            reason,
        ) {
            Ok(_) => true,
            Err(e) => {
                if e.errcode == ErrCode::SegEndValueErr && self.lst.is_empty() {
                    false
                } else {
                    panic!("{}", e)
                }
            }
        }
    }
}

pub fn find_peak_bi<'a, T: Line + 'a, I>(bi_lst: I, is_high: bool) -> Option<Handle<T>>
where
    I: Iterator<Item = &'a T>,
{
    let mut peak_val = if is_high { f64::MIN } else { f64::MAX };
    let mut peak_bi = None;

    for bi in bi_lst {
        if (is_high && bi.line_get_end_val() >= peak_val && bi.line_is_up())
            || (!is_high && bi.line_get_end_val() <= peak_val && bi.line_is_down())
        {
            if let Some(pre) = &bi.line_pre() {
                if let Some(pre_pre) = &pre.line_pre() {
                    if (is_high && pre_pre.line_get_end_val() > bi.line_get_end_val())
                        || (!is_high && pre_pre.line_get_end_val() < bi.line_get_end_val())
                    {
                        continue;
                    }
                }
            }
            peak_val = bi.line_get_end_val();
            peak_bi = Some(bi);
        }
    }
    peak_bi
}

impl<T> std::ops::Deref for CSegListChan<T> {
    type Target = Box<Vec<CSeg<T>>>;

    fn deref(&self) -> &Self::Target {
        &self.lst
    }
}

impl<T> std::ops::DerefMut for CSegListChan<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.lst
    }
}
