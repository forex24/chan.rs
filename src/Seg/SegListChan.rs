use std::{
    cell::RefCell,
    ops::{Index, IndexMut},
    rc::Rc,
    slice::{Iter, IterMut},
};

use crate::Common::{
    types::Handle,
    CEnum::{BiDir, LeftSegMethod, SegType},
    ChanException::{CChanException, ErrCode},
};

use super::{linetype::Line, EigenFX::CEigenFX, Seg::CSeg, SegConfig::CSegConfig};

pub struct CSegListChan<T: Line> {
    pub lst: Vec<Handle<CSeg<T>>>,
    pub lv: SegType,
    pub config: CSegConfig,
}

impl<T: Line> CSegListChan<T> {
    pub fn new(seg_config: CSegConfig, lv: SegType) -> Self {
        let mut instance = Self {
            lst: Vec::new(),
            lv,
            config: seg_config,
        };
        instance.do_init();
        instance
    }

    pub fn do_init(&mut self) {
        // 删除末尾不确定的线段
        while !self.lst.is_empty() && !self.lst.last().unwrap().borrow().is_sure {
            let _seg = self.lst.last().unwrap().clone();
            let _seg_ref = _seg.borrow();
            // TODO:
            //for bi in &_seg_ref.bi_list {
            //    bi.borrow_mut().parent_seg = None;
            //}
            if let Some(pre) = &_seg_ref.pre {
                pre.borrow_mut().next = None;
            }
            self.lst.pop();
        }

        if !self.lst.is_empty() {
            assert!(
                self.lst.last().unwrap().borrow().eigen_fx.is_some()
                    && self
                        .lst
                        .last()
                        .unwrap()
                        .borrow()
                        .eigen_fx
                        .as_ref()
                        .unwrap()
                        .ele[2]
                        .is_some()
            );
            if !self
                .lst
                .last()
                .unwrap()
                .borrow()
                .eigen_fx
                .as_ref()
                .unwrap()
                .ele[2]
                .as_ref()
                .unwrap()
                .borrow()
                .lst
                .last()
                .unwrap()
                .borrow()
                .line_is_sure()
            {
                // 如果确定线段的分形的第三元素包含不确定笔，也需要重新算，不然线段分形元素的高低点可能不对
                self.lst.pop();
            }
        }
    }

    pub fn update(&mut self, bi_lst: &[Handle<T>]) {
        self.do_init();
        if self.lst.is_empty() {
            self.cal_seg_sure(bi_lst, 0);
        } else {
            let begin_idx =
                self.lst.last().unwrap().borrow().end_bi.borrow().line_idx() as usize + 1;
            self.cal_seg_sure(bi_lst, begin_idx);
        }
        self.collect_left_seg(bi_lst);
    }

    fn cal_seg_sure(&mut self, bi_lst: &[Handle<T>], begin_idx: usize) {
        let fx_eigen = self.cal_seg_sure_inner(bi_lst, begin_idx);
        if let Some(fx_eigen) = fx_eigen {
            self.treat_fx_eigen(fx_eigen, bi_lst);
        }
    }

    fn cal_seg_sure_inner(
        &mut self,
        bi_lst: &[Handle<T>],
        begin_idx: usize,
    ) -> Option<CEigenFX<T>> {
        let mut up_eigen = CEigenFX::new(BiDir::Up, false, self.lv); // 上升线段下降笔
        let mut down_eigen = CEigenFX::new(BiDir::Down, false, self.lv); // 下降线段上升笔
        let mut last_seg_dir = if self.lst.is_empty() {
            None
        } else {
            Some(self.lst.last().unwrap().borrow().dir)
        };

        for bi in bi_lst.iter().skip(begin_idx) {
            let mut fx_eigen_dir = None;
            match (bi.borrow().line_dir(), last_seg_dir) {
                (BiDir::Down, Some(BiDir::Down) | None) => {
                    if up_eigen.add(bi.clone()) {
                        fx_eigen_dir = Some(BiDir::Up);
                    }
                }
                (BiDir::Up, Some(BiDir::Up) | None) => {
                    if down_eigen.add(bi.clone()) {
                        fx_eigen_dir = Some(BiDir::Down);
                    }
                }
                _ => {}
            }

            if self.lst.is_empty() {
                // 尝试确定第一段方向，不要以谁先成为分形来决定
                if up_eigen.ele[1].is_some() && bi.borrow().line_is_down() {
                    last_seg_dir = Some(BiDir::Down);
                    down_eigen.clear();
                } else if down_eigen.ele[1].is_some() && bi.borrow().line_is_up() {
                    up_eigen.clear();
                    last_seg_dir = Some(BiDir::Up);
                }

                if up_eigen.ele[1].is_none()
                    && last_seg_dir == Some(BiDir::Down)
                    && bi.borrow().line_dir() == BiDir::Down
                {
                    last_seg_dir = None;
                } else if down_eigen.ele[1].is_none()
                    && last_seg_dir == Some(BiDir::Up)
                    && bi.borrow().line_dir() == BiDir::Up
                {
                    last_seg_dir = None;
                }
            }

            if let Some(dir) = fx_eigen_dir {
                //self.treat_fx_eigen(&fx_eigen, bi_lst);
                //break;
                match dir {
                    BiDir::Up => return Some(up_eigen),
                    BiDir::Down => return Some(down_eigen),
                }
            }
        }
        None
    }

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
                    is_true && fx_eigen.all_bi_is_sure(),
                    None,
                    true,
                    "normal",
                ) {
                    self.cal_seg_sure(bi_lst, end_bi_idx + 1);
                    return;
                }
                self.lst.last_mut().unwrap().borrow_mut().eigen_fx = Some(fx_eigen);
                if is_true {
                    self.cal_seg_sure(bi_lst, end_bi_idx + 1);
                }
            }
            Some(false) => {
                self.cal_seg_sure(bi_lst, fx_eigen.lst[1].borrow().line_idx());
            }
        }
    }
}

impl<T: Line> CSegListChan<T> {
    pub fn len(&self) -> usize {
        self.lst.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lst.is_empty()
    }

    pub fn left_bi_break(&self, bi_lst: &[Handle<T>]) -> bool {
        if self.lst.is_empty() {
            return false;
        }
        let last_seg_end_bi = &self.lst.last().unwrap().borrow().end_bi;
        for bi in bi_lst.iter().skip(last_seg_end_bi.borrow().line_idx() + 1) {
            if last_seg_end_bi.borrow().line_is_up()
                && bi.borrow().line_high() > last_seg_end_bi.borrow().line_high()
            {
                return true;
            } else if last_seg_end_bi.borrow().line_is_down()
                && bi.borrow().line_low() < last_seg_end_bi.borrow().line_low()
            {
                return true;
            }
        }
        false
    }

    pub fn collect_first_seg(&mut self, bi_lst: &[Handle<T>]) {
        if bi_lst.len() < 3 {
            return;
        }
        match self.config.left_method {
            LeftSegMethod::Peak => {
                let _high = bi_lst
                    .iter()
                    .map(|bi| bi.borrow().line_high())
                    .fold(f64::MIN, f64::max);
                let _low = bi_lst
                    .iter()
                    .map(|bi| bi.borrow().line_low())
                    .fold(f64::MAX, f64::min);
                let first_val = bi_lst.first().unwrap().borrow().line_get_begin_val();

                if (_high - first_val).abs() >= (_low - first_val).abs() {
                    if let Some(peak_bi) = find_peak_bi(bi_lst.iter(), true) {
                        self.add_new_seg(
                            bi_lst,
                            peak_bi.borrow().line_idx() as usize,
                            false,
                            Some(BiDir::Up),
                            false,
                            "0seg_find_high",
                        );
                    }
                } else if let Some(peak_bi) = find_peak_bi(bi_lst.iter(), false) {
                    self.add_new_seg(
                        bi_lst,
                        peak_bi.borrow().line_idx() as usize,
                        false,
                        Some(BiDir::Down),
                        false,
                        "0seg_find_low",
                    );
                }
                self.collect_left_as_seg(bi_lst);
            }
            LeftSegMethod::All => {
                let _dir = if bi_lst.last().unwrap().borrow().line_get_end_val()
                    >= bi_lst[0].borrow().line_get_begin_val()
                {
                    BiDir::Up
                } else {
                    BiDir::Down
                };
                self.add_new_seg(
                    bi_lst,
                    bi_lst.last().unwrap().borrow().line_idx() as usize,
                    false,
                    Some(_dir),
                    false,
                    "0seg_collect_all",
                );
            }
            _ => panic!("unknown seg left_method = {:?}", self.config.left_method),
        }
    }

    pub fn collect_left_seg_peak_method(
        &mut self,
        last_seg_end_bi: Handle<T>,
        bi_lst: &[Handle<T>],
    ) {
        if last_seg_end_bi.borrow().line_is_down() {
            if let Some(peak_bi) = find_peak_bi(
                bi_lst[last_seg_end_bi.borrow().line_idx() as usize + 3..].iter(),
                true,
            ) {
                if peak_bi.borrow().line_idx() as usize - last_seg_end_bi.borrow().line_idx() >= 3 {
                    self.add_new_seg(
                        bi_lst,
                        peak_bi.borrow().line_idx() as usize,
                        false,
                        Some(BiDir::Up),
                        true,
                        "collectleft_find_high",
                    );
                }
            }
        } else if let Some(peak_bi) = find_peak_bi(
            bi_lst[last_seg_end_bi.borrow().line_idx() as usize + 3..].iter(),
            false,
        ) {
            if peak_bi.borrow().line_idx() as usize - last_seg_end_bi.borrow().line_idx() >= 3 {
                self.add_new_seg(
                    bi_lst,
                    peak_bi.borrow().line_idx() as usize,
                    false,
                    Some(BiDir::Down),
                    true,
                    "collectleft_find_low",
                );
            }
        }
        self.collect_left_as_seg(bi_lst);
    }

    pub fn collect_segs(&mut self, bi_lst: &[Handle<T>]) {
        let last_bi = bi_lst.last().unwrap();
        let last_seg_end_bi = self.lst.last().unwrap().borrow().end_bi.clone();

        if last_bi.borrow().line_idx() as usize - last_seg_end_bi.borrow().line_idx() < 3 {
            return;
        }

        if last_seg_end_bi.borrow().line_is_down()
            && last_bi.borrow().line_get_end_val() <= last_seg_end_bi.borrow().line_get_end_val()
        {
            if let Some(peak_bi) = find_peak_bi(
                bi_lst[last_seg_end_bi.borrow().line_idx() + 3..].iter(),
                true,
            ) {
                self.add_new_seg(
                    bi_lst,
                    peak_bi.borrow().line_idx() as usize,
                    false,
                    Some(BiDir::Up),
                    true,
                    "collectleft_find_high_force",
                );
                self.collect_left_seg(bi_lst);
            }
        } else if last_seg_end_bi.borrow().line_is_up()
            && last_bi.borrow().line_get_end_val() >= last_seg_end_bi.borrow().line_get_end_val()
        {
            if let Some(peak_bi) = find_peak_bi(
                bi_lst[last_seg_end_bi.borrow().line_idx() + 3..].iter(),
                false,
            ) {
                self.add_new_seg(
                    bi_lst,
                    peak_bi.borrow().line_idx() as usize,
                    false,
                    Some(BiDir::Down),
                    true,
                    "collectleft_find_low_force",
                );
                self.collect_left_seg(bi_lst);
            }
        } else if self.config.left_method == LeftSegMethod::All {
            self.collect_left_as_seg(bi_lst);
        } else if self.config.left_method == LeftSegMethod::Peak {
            self.collect_left_seg_peak_method(last_seg_end_bi, bi_lst);
        } else {
            panic!("unknown seg left_method = {:?}", self.config.left_method);
        }
    }

    pub fn collect_left_seg(&mut self, bi_lst: &[Handle<T>]) {
        if self.lst.is_empty() {
            self.collect_first_seg(bi_lst);
        } else {
            self.collect_segs(bi_lst);
        }
    }

    pub fn collect_left_as_seg(&mut self, bi_lst: &[Handle<T>]) {
        let last_bi = bi_lst.last().unwrap();
        let last_seg_end_bi = self.lst.last().unwrap().borrow().end_bi.clone();

        if last_seg_end_bi.borrow().line_idx() + 1 >= bi_lst.len() {
            return;
        }

        if last_seg_end_bi.borrow().line_dir() == last_bi.borrow().line_dir() {
            self.add_new_seg(
                bi_lst,
                last_bi.borrow().line_idx() as usize - 1,
                false,
                None,
                true,
                "collect_left_1",
            );
        } else {
            self.add_new_seg(
                bi_lst,
                last_bi.borrow().line_idx() as usize,
                false,
                None,
                true,
                "collect_left_0",
            );
        }
    }

    pub fn try_add_new_seg(
        &mut self,
        bi_lst: &[Handle<T>],
        end_bi_idx: usize,
        is_sure: bool,
        seg_dir: Option<BiDir>,
        split_first_seg: bool,
        reason: &str,
    ) -> Result<(), CChanException> {
        if self.lst.is_empty() && split_first_seg && end_bi_idx >= 3 {
            if let Some(peak_bi) = find_peak_bi(
                bi_lst[..end_bi_idx - 2].iter().rev(),
                bi_lst[end_bi_idx].borrow().line_is_down(),
            ) {
                let peak_bi_ref = peak_bi.borrow();
                if (peak_bi_ref.line_is_down()
                    && (peak_bi_ref.line_low() < bi_lst[0].borrow().line_low()
                        || peak_bi_ref.line_idx() == 0))
                    || (peak_bi_ref.line_is_up()
                        && (peak_bi_ref.line_high() > bi_lst[0].borrow().line_high()
                            || peak_bi_ref.line_idx() == 0))
                {
                    // 要比第一笔开头还高/低（因为没有比较到）
                    self.add_new_seg(
                        bi_lst,
                        peak_bi_ref.line_idx() as usize,
                        false,
                        Some(peak_bi_ref.line_dir()),
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
            self.lst.last().unwrap().borrow().end_bi.borrow().line_idx() + 1
        };
        let bi1 = bi_lst[bi1_idx].clone();
        let bi2 = bi_lst[end_bi_idx].clone();

        let new_seg = Rc::new(RefCell::new(CSeg::new(
            self.lst.len(),
            bi1,
            bi2,
            is_sure,
            seg_dir,
            reason,
        )?));

        if self.lst.len() >= 2 {
            let last_seg = self.lst.last().unwrap().clone();
            last_seg.borrow_mut().next = Some(Rc::clone(&new_seg));
            new_seg.borrow_mut().pre = Some(last_seg);
        }

        new_seg
            .borrow_mut()
            .update_bi_list(bi_lst, bi1_idx, end_bi_idx);
        self.lst.push(new_seg);

        Ok(())
    }

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

    pub fn exist_sure_seg(&self) -> bool {
        self.lst.iter().any(|seg| seg.borrow().is_sure)
    }

    // 实现 iter() 方法返回不可变迭代器
    pub fn iter(&self) -> Iter<'_, Handle<CSeg<T>>> {
        self.lst.iter()
    }

    // 实现 iter_mut() 方法返回可变迭代器
    pub fn iter_mut(&mut self) -> IterMut<'_, Handle<CSeg<T>>> {
        self.lst.iter_mut()
    }

    // last() 方法已经通过 Vec 的方法自动获得
    pub fn last(&self) -> Option<&Handle<CSeg<T>>> {
        self.lst.last()
    }

    // 如果需要可变的 last()
    pub fn last_mut(&mut self) -> Option<&mut Handle<CSeg<T>>> {
        self.lst.last_mut()
    }
}

pub fn find_peak_bi<'a, T: Line + 'a, I>(bi_lst: I, is_high: bool) -> Option<Handle<T>>
where
    I: Iterator<Item = &'a Handle<T>>,
{
    let mut peak_val = if is_high { f64::MIN } else { f64::MAX };
    let mut peak_bi = None;

    for bi in bi_lst {
        let bi_ref = bi.borrow();
        if (is_high && bi_ref.line_get_end_val() >= peak_val && bi_ref.line_is_up())
            || (!is_high && bi_ref.line_get_end_val() <= peak_val && bi_ref.line_is_down())
        {
            if let Some(pre) = &bi_ref.line_pre() {
                if let Some(pre_pre) = &pre.borrow().line_pre() {
                    if (is_high && pre_pre.borrow().line_get_end_val() > bi_ref.line_get_end_val())
                        || (!is_high
                            && pre_pre.borrow().line_get_end_val() < bi_ref.line_get_end_val())
                    {
                        continue;
                    }
                }
            }
            peak_val = bi_ref.line_get_end_val();
            peak_bi = Some(bi.clone());
        }
    }
    peak_bi
}

// 实现 Index 和 IndexMut traits 以支持索引访问
impl<T: Line> Index<usize> for CSegListChan<T> {
    type Output = Handle<CSeg<T>>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.lst[index]
    }
}

impl<T: Line> IndexMut<usize> for CSegListChan<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.lst[index]
    }
}

// 实现 IntoIterator 以支持迭代
impl<T: Line> IntoIterator for CSegListChan<T> {
    type Item = Handle<CSeg<T>>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.lst.into_iter()
    }
}

// 实现 IntoIterator 以支持 for 循环直接迭代引用
impl<'a, T: Line> IntoIterator for &'a CSegListChan<T> {
    type Item = &'a Handle<CSeg<T>>;
    type IntoIter = Iter<'a, Handle<CSeg<T>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

// 实现 IntoIterator 以支持 for 循环直接迭代可变引用
impl<'a, T: Line> IntoIterator for &'a mut CSegListChan<T> {
    type Item = &'a mut Handle<CSeg<T>>;
    type IntoIter = IterMut<'a, Handle<CSeg<T>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
