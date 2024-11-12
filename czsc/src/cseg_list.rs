// 已完备
use crate::CEigenFx;
use crate::CSeg;
use crate::CSegConfig;
use crate::Handle;
use crate::ICalcMetric;
use crate::IParent;
use crate::LeftSegMethod;
use crate::LineType;
use crate::ToHandle;
use crate::{Direction, SegType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestSegError {
    SegEndValueErr,
    SegLenErr,
}

pub struct CSegListChan<T> {
    pub lst: Box<Vec<CSeg<T>>>,
    pub lv: SegType,
    pub config: CSegConfig,
}

impl<T: LineType + IParent + ToHandle + ICalcMetric> CSegListChan<T> {
    pub fn new(seg_config: CSegConfig, lv: SegType) -> Self {
        Self {
            lst: Box::<Vec<CSeg<T>>>::default(),
            lv,
            config: seg_config,
        }
    }

    // 已完备
    fn do_init(&mut self, bi_lst: &[T]) {
        // 删除末尾不确定的线段
        while !self.lst.is_empty() && !self.lst.last().unwrap().is_sure {
            let last_seg = self.lst.last().unwrap();
            for bi in &last_seg.bi_list {
                if bi.index() < bi_lst.len() {
                    // 这里检查的原因是，如果是虚笔，最后一笔可能失效
                    bi.as_mut().set_parent_seg_dir(None);
                    bi.as_mut().set_parent_seg_idx(None);
                }
            }
            self.lst.pop();
        }

        if !self.lst.is_empty() {
            assert!(self.lst.last().unwrap().eigen_fx.is_some());
            assert!(self.lst.last().unwrap().eigen_fx.as_ref().unwrap().ele[2].is_some());

            let last_bi_index = self.lst.last().unwrap().eigen_fx.as_ref().unwrap().ele[2]
                .as_ref()
                .unwrap()
                .lst
                .last()
                .unwrap();
            if !last_bi_index.is_sure() {
                {
                    // 如果确定线段的分形的第三元素包含不确定笔，也需要重新算，不然线段分形元素的高低点可能不对
                    // TODO:是否要向该线段包含的笔，设置parent_seg_dir & parent_seg_idx为None
                    self.lst.pop();
                }
            }
        }
    }

    // 已完备
    pub fn update(&mut self, bi_lst: &[T]) {
        self.do_init(bi_lst);

        let begin_idx = if self.lst.is_empty() {
            0
        } else {
            self.lst.last().unwrap().end_bi.index() + 1
        };

        self.cal_seg_sure(bi_lst, begin_idx);

        self.collect_left_seg(bi_lst);
    }

    // TODO:修改递归为loop
    fn cal_seg_sure(&mut self, bi_lst: &[T], begin_idx: usize) {
        let fx_eigen = self.cal_eigen_fx(bi_lst, begin_idx);
        if let Some(fx_eigen) = fx_eigen {
            self.treat_fx_eigen(fx_eigen, bi_lst);
        }
    }

    // 99% 完备
    fn cal_eigen_fx(&mut self, bi_lst: &[T], begin_idx: usize) -> Option<CEigenFx<T>> {
        let mut up_eigen = CEigenFx::new(Direction::Up, true, self.lv); // 上升线段下降笔
        let mut down_eigen = CEigenFx::new(Direction::Down, true, self.lv); // 下降线段上升笔
        let mut last_seg_dir = if self.lst.is_empty() {
            None
        } else {
            Some(self.lst.last().unwrap().dir)
        };

        for bi in bi_lst.iter().skip(begin_idx) {
            let mut fx_eigen_dir = None;
            match (bi.direction(), last_seg_dir) {
                (Direction::Down, Some(Direction::Down) | None) => {
                    //最后线段的方向不是向上的，意味着当前线段假设方向就是向上的，所以下降笔就是特征序列笔，分型是顶分型
                    if up_eigen.add(bi.to_handle()) {
                        fx_eigen_dir = Some(Direction::Up);
                    }
                }
                (Direction::Up, Some(Direction::Up) | None) => {
                    //最后线段的方向不是向下的，意味着当前线段假设方向就是向下的，所以上升笔就是特征序列笔，分型是底分型
                    if down_eigen.add(bi.to_handle()) {
                        fx_eigen_dir = Some(Direction::Down);
                    }
                }

                _ => {}
            };

            if self.lst.is_empty() {
                // 尝试确定第一段方向，不要以谁先成为分形来决定，反例：US.EVRG
                if up_eigen.ele[1].is_some() && bi.is_down() {
                    last_seg_dir = Some(Direction::Down);
                    down_eigen.clear();
                } else if down_eigen.ele[1].is_some() && bi.is_up() {
                    up_eigen.clear();
                    last_seg_dir = Some(Direction::Up);
                }

                if (up_eigen.ele[1].is_none()
                    && last_seg_dir == Some(Direction::Down)
                    && bi.direction() == Direction::Down)
                    || (down_eigen.ele[1].is_none()
                        && last_seg_dir == Some(Direction::Up)
                        && bi.direction() == Direction::Up)
                {
                    last_seg_dir = None;
                }
            }

            if let Some(dir) = fx_eigen_dir {
                match dir {
                    Direction::Up => return Some(up_eigen),
                    Direction::Down => return Some(down_eigen),
                }
            }
        }
        None
    }

    fn treat_fx_eigen(&mut self, mut fx_eigen: CEigenFx<T>, bi_lst: &[T]) {
        let test = fx_eigen.can_be_end(bi_lst);
        let end_bi_idx = fx_eigen.get_peak_bi_idx();

        match test {
            Some(true) | None => {
                // None表示反向分型找到尾部也没找到
                let is_true = test.is_some(); // 如果是正常结束
                if !self.add_new_seg(
                    bi_lst,
                    end_bi_idx,
                    is_true && fx_eigen.all_bi_is_sure(),
                    None,
                    true,
                    "normal",
                ) {
                    // 防止第一根线段的方向与首尾值异常
                    self.cal_seg_sure(bi_lst, end_bi_idx + 1);
                    return;
                }

                self.lst.last_mut().unwrap().eigen_fx = Some(fx_eigen);

                if is_true {
                    self.cal_seg_sure(bi_lst, end_bi_idx + 1);
                }
            }
            Some(false) => {
                self.cal_seg_sure(bi_lst, fx_eigen.lst[1].index());
            }
        }
    }
}

impl<T: LineType + IParent + ToHandle> CSegListChan<T> {
    //100% 完备
    fn collect_first_seg(&mut self, bi_lst: &[T]) {
        if bi_lst.len() < 3 {
            return;
        }
        match self.config.left_method {
            LeftSegMethod::Peak => {
                let _high = bi_lst.iter().map(|bi| bi.high()).reduce(f64::max).unwrap();
                let _low = bi_lst.iter().map(|bi| bi.low()).reduce(f64::min).unwrap();
                if (_high - bi_lst[0].get_begin_val()).abs()
                    >= (_low - bi_lst[0].get_begin_val()).abs()
                {
                    let peak_bi = Self::find_peak_bi(bi_lst, true).expect("Peak bi should exist");
                    self.add_new_seg(
                        bi_lst,
                        peak_bi.index(),
                        false,
                        Some(Direction::Up),
                        false,
                        "0seg_find_high",
                    );
                } else {
                    let peak_bi = Self::find_peak_bi(bi_lst, false).expect("Peak bi sould exist");
                    self.add_new_seg(
                        bi_lst,
                        peak_bi.index(),
                        false,
                        Some(Direction::Down),
                        false,
                        "0seg_find_low",
                    );
                }
                self.collect_left_as_seg(bi_lst);
            }

            LeftSegMethod::All => {
                let _dir = if bi_lst[bi_lst.len() - 1].get_end_val() >= bi_lst[0].get_begin_val() {
                    Direction::Up
                } else {
                    Direction::Down
                };
                self.add_new_seg(
                    bi_lst,
                    bi_lst.last().unwrap().to_handle().index(),
                    false,
                    Some(_dir),
                    false,
                    "0seg_collect_all",
                );
            }
        }
    }

    //99% 完备，见TODO
    fn collect_left_seg_peak_method(&mut self, last_seg_end_bi: &Handle<T>, bi_lst: &[T]) {
        if last_seg_end_bi.is_down() {
            if let Some(peak_bi) = Self::find_peak_bi(&bi_lst[last_seg_end_bi.index() + 3..], true)
            {
                if peak_bi.index() - last_seg_end_bi.index() >= 3 {
                    self.add_new_seg(
                        bi_lst,
                        peak_bi.index(),
                        false,
                        Some(Direction::Up),
                        true,
                        "collectleft_find_high",
                    );
                }
            }
        } else if let Some(peak_bi) =
            Self::find_peak_bi(&bi_lst[last_seg_end_bi.index() + 3..], false)
        {
            if peak_bi.index() - last_seg_end_bi.index() >= 3 {
                self.add_new_seg(
                    bi_lst,
                    peak_bi.index(),
                    false,
                    Some(Direction::Down),
                    true,
                    "collectleft_find_low",
                );
            }
        }
        // FIXME:这里修改了入参
        let _last_seg_end_bi = &self.lst[self.lst.len() - 1].end_bi;
        self.collect_left_as_seg(bi_lst);
    }

    // 已完备
    fn collect_segs(&mut self, bi_lst: &[T]) {
        let last_bi = bi_lst.last().unwrap();
        let last_seg_end_bi = self.lst.last().unwrap().end_bi;

        if last_bi.to_handle().index() - last_seg_end_bi.index() < 3 {
            return;
        }

        if last_seg_end_bi.is_down() && last_bi.get_end_val() <= last_seg_end_bi.get_end_val() {
            if let Some(peak_bi) = Self::find_peak_bi(&bi_lst[last_seg_end_bi.index() + 3..], true)
            {
                self.add_new_seg(
                    bi_lst,
                    peak_bi.index(),
                    false,
                    Some(Direction::Up),
                    true,
                    "collectleft_find_high_force",
                );
                self.collect_left_seg(bi_lst);
            }
        } else if last_seg_end_bi.is_up() && last_bi.get_end_val() >= last_seg_end_bi.get_end_val()
        {
            if let Some(peak_bi) = Self::find_peak_bi(&bi_lst[last_seg_end_bi.index() + 3..], false)
            {
                self.add_new_seg(
                    bi_lst,
                    peak_bi.index(),
                    false,
                    Some(Direction::Down),
                    true,
                    "collectleft_find_low_force",
                );
                self.collect_left_seg(bi_lst);
            }
        }
        // 剩下线段的尾部相比于最后一个线段的尾部，高低关系和最后一个虚线段的方向一致
        else {
            match self.config.left_method {
                LeftSegMethod::All => {
                    //容易找不到二类买卖点！！
                    self.collect_left_as_seg(bi_lst);
                }
                LeftSegMethod::Peak => {
                    self.collect_left_seg_peak_method(&last_seg_end_bi, bi_lst);
                }
            }
        }
    }

    // 已完备
    fn collect_left_seg(&mut self, bi_lst: &[T]) {
        if self.lst.is_empty() {
            self.collect_first_seg(bi_lst);
        } else {
            self.collect_segs(bi_lst);
        }
    }

    // 已完备
    fn collect_left_as_seg(&mut self, bi_lst: &[T]) {
        let last_bi = bi_lst.last().unwrap();
        let last_seg_end_bi = self.lst.last().unwrap().end_bi;

        if last_seg_end_bi.index() + 1 >= bi_lst.len() {
            return;
        }

        if last_seg_end_bi.direction() == last_bi.direction() {
            self.add_new_seg(
                bi_lst,
                last_bi.to_handle().index() - 1,
                false,
                None,
                true,
                "collect_left_1",
            );
        } else {
            self.add_new_seg(
                bi_lst,
                last_bi.to_handle().index(),
                false,
                None,
                true,
                "collect_left_0",
            );
        }
    }

    // 替代seg.check
    fn check_seg_valid(&self, seg: &CSeg<T>) -> Result<(), TestSegError> {
        if !seg.is_sure {
            return Ok(());
        }
        let start_bi = seg.start_bi;
        let end_bi = seg.end_bi;

        if seg.is_down() {
            if start_bi.get_begin_val() < end_bi.get_end_val() {
                println!("下降线段起始点应该高于结束点! {}", seg.to_handle().index());
                return Err(TestSegError::SegEndValueErr);
            }
        } else if start_bi.get_begin_val() > end_bi.get_end_val() {
            println!("上升线段起始点应该低于结束点! {}", seg.to_handle().index());
            return Err(TestSegError::SegEndValueErr);
        }

        if end_bi.index() - start_bi.index() < 2 {
            return Err(TestSegError::SegLenErr);
        }

        Ok(())
    }

    fn try_add_new_seg(
        &mut self,
        bi_lst: &[T],
        end_bi_idx: usize,
        is_sure: bool,
        seg_dir: Option<Direction>,
        split_first_seg: bool,
        reason: &str,
    ) -> Result<(), TestSegError> {
        if self.lst.is_empty() && split_first_seg && end_bi_idx >= 3 {
            if let Some(peak_bi) = Self::find_peak_bi(
                bi_lst[0..end_bi_idx - 2].iter().rev(),
                bi_lst[end_bi_idx].is_down(),
            ) {
                if (peak_bi.is_down() && (peak_bi.low() < bi_lst[0].low() || peak_bi.index() == 0))
                    || (peak_bi.is_up()
                        && (peak_bi.high() > bi_lst[0].high() || peak_bi.index() == 0))
                {
                    // 要比第一笔开头还高/低（因为没有比较到）
                    self.add_new_seg(
                        bi_lst,
                        peak_bi.index(),
                        false,
                        Some(peak_bi.direction()),
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
            self.lst[self.lst.len() - 1].end_bi.index() + 1
        };

        let bi1 = &bi_lst[bi1_idx];
        let bi2 = &bi_lst[end_bi_idx];

        let new_seg = CSeg::new(
            &self.lst,
            self.lst.len(),
            bi1.to_handle(),
            bi2.to_handle(),
            is_sure,
            seg_dir,
            reason,
        );

        self.check_seg_valid(&new_seg)?;

        self.lst.push(new_seg);

        self.lst
            .last_mut()
            .unwrap()
            .update_bi_list(bi_lst, bi1_idx, end_bi_idx);
        Ok(())
    }

    fn add_new_seg(
        &mut self,
        bi_lst: &[T],
        end_bi_idx: usize,
        is_sure: bool,
        seg_dir: Option<Direction>,
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
            Err(e) => !(e == TestSegError::SegEndValueErr && self.lst.is_empty()),
        }
    }

    fn find_peak_bi<'a, I>(bi_iter: I, is_high: bool) -> Option<Handle<T>>
    where
        I: IntoIterator<Item = &'a T>,
        T: 'a,
    {
        let mut peak_val = if is_high {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        };
        let mut peak_bi = None;

        for bi in bi_iter {
            let bi = bi.to_handle();
            let should_update = if is_high {
                bi.get_end_val() >= peak_val && bi.is_up()
            } else {
                bi.get_end_val() <= peak_val && bi.is_down()
            };

            if should_update {
                if let (Some(_pre), Some(pre_pre)) = (bi.prev(), bi.prev().and_then(|b| b.prev())) {
                    let should_skip = if is_high {
                        pre_pre.get_end_val() > bi.get_end_val()
                    } else {
                        pre_pre.get_end_val() < bi.get_end_val()
                    };
                    if should_skip {
                        continue;
                    }
                }
                peak_val = bi.get_end_val();
                peak_bi = Some(bi);
            }
        }
        peak_bi
    }
}

//fn find_peak_bi<T: LineType + ToHandle>(bi_lst: &[T], is_high: bool) -> Option<Handle<T>> {
//    let mut peak_val = if is_high {
//        f64::NEG_INFINITY
//    } else {
//        f64::INFINITY
//    };
//    let mut peak_bi = None;
//    for bi in bi_lst.iter().map(|bi| bi.to_handle()) {
//        if (is_high && bi.get_end_val() >= peak_val && bi.is_up())
//            || (!is_high && bi.get_end_val() <= peak_val && bi.is_down())
//        {
//            if bi.prev().is_some()
//                && bi.prev().as_ref().unwrap().prev().is_some()
//                && ((is_high
//                    && bi.prev().as_ref().unwrap().prev().unwrap().get_end_val()
//                        > bi.get_end_val())
//                    || (!is_high
//                        && bi.prev().as_ref().unwrap().prev().unwrap().get_end_val()
//                            < bi.get_end_val()))
//            {
//                continue;
//            }
//            peak_val = bi.get_end_val();
//            peak_bi = Some(bi);
//        }
//    }
//    peak_bi
//}

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
