use crate::{
    CChanException, CSeg, CSegListChan, Direction, ErrCode, Handle, IParent, LeftSegMethod,
    LineType, TestSegError, ToHandle,
};

impl<T: LineType + IParent + ToHandle> CSegListChan<T> {
    pub fn left_bi_break(&self, bi_lst: &[T]) -> bool {
        if self.lst.is_empty() {
            return false;
        }
        let last_seg_end_bi = &self.lst.last().unwrap().end_bi;
        for bi in bi_lst.iter().skip(last_seg_end_bi.index() as usize + 1) {
            if last_seg_end_bi.is_up() && bi.high() > last_seg_end_bi.high() {
                return true;
            } else if last_seg_end_bi.is_down() && bi.low() < last_seg_end_bi.low() {
                return true;
            }
        }
        false
    }

    //100% 完备
    /// 收集第一个线段
    ///
    /// # Arguments
    /// * `bi_lst` - 笔列表
    pub(crate) fn collect_first_seg(&mut self, bi_lst: &[T]) -> Result<(), CChanException> {
        if bi_lst.len() < 3 {
            return Ok(());
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
                    )?;
                } else {
                    let peak_bi = Self::find_peak_bi(bi_lst, false).expect("Peak bi sould exist");
                    self.add_new_seg(
                        bi_lst,
                        peak_bi.index(),
                        false,
                        Some(Direction::Down),
                        false,
                        "0seg_find_low",
                    )?;
                }
                self.collect_left_as_seg(bi_lst)?;
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
                )?;
            }
        }

        Ok(())
    }

    //99% 完备，见TODO
    /// 使用峰值方法收集剩余笔构成的线段
    ///
    /// # Arguments
    /// * `last_seg_end_bi` - 最后一个线段的结束笔
    /// * `bi_lst` - 笔列表
    pub(crate) fn collect_left_seg_peak_method(
        &mut self,
        last_seg_end_bi: &Handle<T>,
        bi_lst: &[T],
    ) -> Result<(), CChanException> {
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
                    )?;
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
                )?;
            }
        }
        // FIXME:这里修改了入参
        let _last_seg_end_bi = &self.lst[self.lst.len() - 1].end_bi;
        self.collect_left_as_seg(bi_lst)?;

        Ok(())
    }

    // 已完备
    /// 收集剩余笔构成线段
    ///
    /// # Arguments
    /// * `bi_lst` - 笔列表
    pub(crate) fn collect_segs(&mut self, bi_lst: &[T]) -> Result<(), CChanException> {
        let last_bi = bi_lst.last().unwrap();
        let last_seg_end_bi = self.lst.last().unwrap().end_bi;

        if last_bi.to_handle().index() - last_seg_end_bi.index() < 3 {
            return Ok(());
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
                )?;
                self.collect_left_seg(bi_lst)?;
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
                )?;
                self.collect_left_seg(bi_lst)?;
            }
        }
        // 剩下线段的尾部相比于最后一个线段的尾部，高低关系和最后一个虚线段的方向一致
        else {
            match self.config.left_method {
                LeftSegMethod::All => {
                    //容易找不到二类买卖点！！
                    self.collect_left_as_seg(bi_lst)?;
                }
                LeftSegMethod::Peak => {
                    self.collect_left_seg_peak_method(&last_seg_end_bi, bi_lst)?;
                }
            }
        }

        Ok(())
    }

    // 已完备
    /// 收集剩余笔构成线段的主入口
    ///
    /// # Arguments
    /// * `bi_lst` - 笔列表
    pub(crate) fn collect_left_seg(&mut self, bi_lst: &[T]) -> Result<(), CChanException> {
        if self.lst.is_empty() {
            self.collect_first_seg(bi_lst)?;
        } else {
            self.collect_segs(bi_lst)?;
        }

        Ok(())
    }

    // 已完备
    /// 将剩余笔直接构成一个线段
    ///
    /// # Arguments
    /// * `bi_lst` - 笔列表
    pub(crate) fn collect_left_as_seg(&mut self, bi_lst: &[T]) -> Result<(), CChanException> {
        let last_bi = bi_lst.last().unwrap();
        let last_seg_end_bi = self.lst.last().unwrap().end_bi;

        if last_seg_end_bi.index() + 1 >= bi_lst.len() {
            return Ok(());
        }

        if last_seg_end_bi.direction() == last_bi.direction() {
            self.add_new_seg(
                bi_lst,
                last_bi.to_handle().index() - 1,
                false,
                None,
                true,
                "collect_left_1",
            )?;
        } else {
            self.add_new_seg(
                bi_lst,
                last_bi.to_handle().index(),
                false,
                None,
                true,
                "collect_left_0",
            )?;
        }

        Ok(())
    }

    /// 检查线段是否有效
    ///
    /// # Arguments
    /// * `seg` - 待检查的线段
    ///
    /// # Returns
    /// 返回检查结果，Ok(())表示有效，Err表示无效
    //pub(crate) fn check_seg_valid(&self, seg: &CSeg<T>) -> Result<(), CChanException> {
    //    if !seg.is_sure {
    //        return Ok(());
    //    }
    //    let start_bi = seg.start_bi;
    //    let end_bi = seg.end_bi;
    //
    //    if seg.is_down() {
    //        if start_bi.get_begin_val() < end_bi.get_end_val() {
    //            println!("下降线段起始点应该高于结束点! {}", seg.to_handle().index());
    //            return Err(TestSegError::SegEndValueErr);
    //        }
    //    } else if start_bi.get_begin_val() > end_bi.get_end_val() {
    //        println!("上升线段起始点应该低于结束点! {}", seg.to_handle().index());
    //        return Err(TestSegError::SegEndValueErr);
    //    }
    //
    //    if end_bi.index() - start_bi.index() < 2 {
    //        return Err(TestSegError::SegLenErr);
    //    }
    //
    //    Ok(())
    //}

    /// 尝试添加新线段
    ///
    /// # Arguments
    /// * `bi_lst` - 笔列表
    /// * `end_bi_idx` - 结束笔索引
    /// * `is_sure` - 是否确定线段
    /// * `seg_dir` - 线段方向
    /// * `split_first_seg` - 是否分割第一个线段
    /// * `reason` - 添加原因
    ///
    /// # Returns
    /// 返回添加结果
    pub(crate) fn try_add_new_seg(
        &mut self,
        bi_lst: &[T],
        end_bi_idx: usize,
        is_sure: bool,
        seg_dir: Option<Direction>,
        split_first_seg: bool,
        reason: &str,
    ) -> Result<(), CChanException> {
        if self.lst.is_empty() && split_first_seg && end_bi_idx >= 3 {
            if let Some(peak_bi) = Self::find_peak_bi(
                bi_lst[..end_bi_idx - 2].iter().rev(),
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
                    )?;
                    self.add_new_seg(bi_lst, end_bi_idx, false, None, true, "split_first_2nd")?;
                    return Ok(());
                }
            }
        }

        let bi1_idx = if self.lst.is_empty() {
            0
        } else {
            self.lst[self.lst.len() - 1].end_bi.index() + 1
        };

        assert!(bi1_idx < bi_lst.len());

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
        )?;

        //self.check_seg_valid(&new_seg)?;

        self.lst.push(new_seg);

        self.lst
            .last_mut()
            .unwrap()
            .update_bi_list(bi_lst, bi1_idx, end_bi_idx);

        Ok(())
    }

    /// 添加新线段的包装方法
    ///
    /// # Arguments
    /// * `bi_lst` - 笔列表
    /// * `end_bi_idx` - 结束笔索引
    /// * `is_sure` - 是否确定线段
    /// * `seg_dir` - 线段方向
    /// * `split_first_seg` - 是否分割第一个线段
    /// * `reason` - 添加原因
    ///
    /// # Returns
    /// 返回是否添加成功
    pub(crate) fn add_new_seg(
        &mut self,
        bi_lst: &[T],
        end_bi_idx: usize,
        is_sure: bool,
        seg_dir: Option<Direction>,
        split_first_seg: bool,
        reason: &str,
    ) -> Result<bool, CChanException> {
        match self.try_add_new_seg(
            bi_lst,
            end_bi_idx,
            is_sure,
            seg_dir,
            split_first_seg,
            reason,
        ) {
            Ok(_) => Ok(true),
            //Err(e) => !(e == TestSegError::SegEndValueErr && self.lst.is_empty()),
            Err(e) => {
                if e.errcode == ErrCode::SegEndValueErr && self.lst.is_empty() {
                    Ok(false)
                } else {
                    Err(e)
                }
            }
        }
    }

    pub fn exist_sure_seg(&self) -> bool {
        self.lst.iter().any(|seg| seg.is_sure)
    }
    /// 查找峰值笔
    ///
    /// # Arguments
    /// * `bi_iter` - 笔迭代器
    /// * `is_high` - 是否查找高点
    ///
    /// # Returns
    /// 返回可能的峰值笔
    pub(crate) fn find_peak_bi<'a, I>(bi_iter: I, is_high: bool) -> Option<Handle<T>>
    where
        I: IntoIterator<Item = &'a T>,
        T: 'a,
    {
        //bi_iter.into_iter().fold(None, |peak_bi, bi| {
        //    let bi_val = bi.get_end_val();
        //    let peak_val = peak_bi.map_or(
        //        if is_high {
        //            f64::NEG_INFINITY
        //        } else {
        //            f64::INFINITY
        //        },
        //        |b| b.get_end_val(),
        //    );
        //
        //    if (is_high && bi_val >= peak_val && bi.is_up())
        //        || (!is_high && bi_val <= peak_val && bi.is_down())
        //    {
        //        if let Some(pre) = bi.to_handle().prev() {
        //            if let Some(pre_pre) = pre.to_handle().prev() {
        //                let pre_pre_val = pre_pre.get_end_val();
        //                if (is_high && pre_pre_val > bi_val) || (!is_high && pre_pre_val < bi_val) {
        //                    return peak_bi;
        //                }
        //            }
        //        }
        //        return Some(bi.to_handle());
        //    }
        //    peak_bi
        //})
        let mut peak_val = if is_high {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        };
        let mut peak_bi = None;
        for bi in bi_iter {
            let bi_ref = bi.to_handle();
            if (is_high && bi_ref.get_end_val() >= peak_val && bi_ref.is_up())
                || (!is_high && bi_ref.get_end_val() <= peak_val && bi_ref.is_down())
            {
                if let Some(pre) = &bi_ref.prev() {
                    if let Some(pre_pre) = &pre.prev() {
                        if (is_high && pre_pre.get_end_val() > bi_ref.get_end_val())
                            || (!is_high && pre_pre.get_end_val() < bi_ref.get_end_val())
                        {
                            continue;
                        }
                    }
                }
                peak_val = bi_ref.get_end_val();
                peak_bi = Some(bi_ref);
            }
        }
        peak_bi

        //for bi in bi_iter {
        //    let bi = bi.to_handle();
        //    let should_update = if is_high {
        //        bi.get_end_val() >= peak_val && bi.is_up()
        //    } else {
        //        bi.get_end_val() <= peak_val && bi.is_down()
        //    };
        //
        //    if should_update {
        //        if let (Some(_pre), Some(pre_pre)) = (bi.prev(), bi.prev().and_then(|b| b.prev())) {
        //            let should_skip = if is_high {
        //                pre_pre.get_end_val() > bi.get_end_val()
        //            } else {
        //                pre_pre.get_end_val() < bi.get_end_val()
        //            };
        //            if should_skip {
        //                continue;
        //            }
        //        }
        //        peak_val = bi.get_end_val();
        //        peak_bi = Some(bi);
        //    }
        //}
        //peak_bi
    }
}

//pub(crate) fn find_peak_bi<T: LineType + ToHandle>(bi_lst: &[T], is_high: bool) -> Option<Handle<T>> {
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
