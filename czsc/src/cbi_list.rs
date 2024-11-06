use crate::AsHandle;
use crate::CBi;
use crate::CBiConfig;
use crate::Candle;
use crate::Handle;
use crate::Indexable;

use crate::{FxType, KlineDir};

pub struct CBiList {
    pub bi_list: Box<Vec<CBi>>,
    pub last_end: Option<Handle<Candle>>,
    pub config: CBiConfig,
    pub free_klc_lst: Vec<Handle<Candle>>, // 仅仅用作第一笔未画出来之前的缓存，为了获得更精准的结果而已，不加这块逻辑其实对后续计算没太大影响
}

impl CBiList {
    pub fn new(bi_conf: CBiConfig) -> Self {
        CBiList {
            bi_list: Box::new(Vec::with_capacity(10240)),
            last_end: None,
            config: bi_conf,
            free_klc_lst: Vec::new(),
        }
    }

    // 已完备
    fn try_create_first_bi(&mut self, klc: &Candle) -> bool {
        assert!(self.bi_list.is_empty());
        assert!(klc.fx_type != FxType::Unknown);

        for exist_free_klc in &self.free_klc_lst {
            if exist_free_klc.fx_type == klc.fx_type {
                continue;
            }
            if self.can_make_bi(klc.as_handle(), *exist_free_klc, false) {
                self.add_new_bi(exist_free_klc.as_handle(), klc.as_handle(), true);
                self.last_end = Some(klc.as_handle());
                return true;
            }
        }
        self.free_klc_lst.push(klc.as_handle());
        self.last_end = Some(klc.as_handle());
        false
    }

    // 已完备
    pub fn update_bi(&mut self, klc: &Candle, last_klc: &Candle, cal_virtual: bool) -> bool {
        // klc:倒数第二根klc
        // last_klc: 倒数第1根klc
        let flag1 = self.update_bi_sure(klc);
        if cal_virtual {
            let flag2 = self.try_add_virtual_bi(last_klc, false);
            flag1 || flag2
        } else {
            flag1
        }
    }

    fn can_update_peak(&self, klc: Handle<Candle>) -> bool {
        if self.config.bi_allow_sub_peak || self.bi_list.len() < 2 {
            return false;
        }
        if self.bi_list.last().unwrap()._is_down()
            && klc.high < self.bi_list.last().unwrap()._get_begin_val()
        {
            return false;
        }
        if self.bi_list.last().unwrap()._is_up()
            && klc.low > self.bi_list.last().unwrap()._get_begin_val()
        {
            return false;
        }
        if !end_is_peak(
            self.bi_list[self.bi_list.len() - 2].begin_klc,
            klc.as_handle(),
        ) {
            return false;
        }
        if self.bi_list.last().unwrap()._is_down()
            && self.bi_list.last().unwrap()._get_end_val()
                < self.bi_list[self.bi_list.len() - 2]._get_begin_val()
        {
            return false;
        }
        if self.bi_list.last().unwrap()._is_up()
            && self.bi_list.last().unwrap()._get_end_val()
                > self.bi_list[self.bi_list.len() - 2]._get_begin_val()
        {
            return false;
        }
        true
    }

    // 已完备
    fn update_peak(&mut self, klc: Handle<Candle>, for_virtual: bool) -> bool {
        if !self.can_update_peak(klc) {
            return false;
        }
        let _tmp_last_bi = self.bi_list.pop().unwrap();
        if !self.try_update_end(klc.as_handle(), for_virtual) {
            self.bi_list.push(_tmp_last_bi);
            false
        } else {
            if for_virtual {
                self.bi_list
                    .last_mut()
                    .unwrap()
                    .append_sure_end(_tmp_last_bi.end_klc.as_handle());
            }
            true
        }
    }

    // 已完备
    fn update_bi_sure(&mut self, klc: &Candle) -> bool {
        // klc:倒数第二根klc
        let _tmp_end = self.get_last_klu_of_last_bi();
        self.delete_virtual_bi();

        // 返回值：是否出现新笔
        if klc.fx_type == FxType::Unknown {
            return _tmp_end != self.get_last_klu_of_last_bi(); // 虚笔是否有变
        }

        if self.last_end.is_none() || self.bi_list.is_empty() {
            return self.try_create_first_bi(klc);
        }

        if klc.fx_type == self.last_end.unwrap().fx_type {
            return self.try_update_end(klc.as_handle(), false);
        }

        if self.can_make_bi(klc.as_handle(), self.last_end.unwrap(), false) {
            self.add_new_bi(self.last_end.unwrap(), klc.as_handle(), true);
            self.last_end = Some(klc.as_handle());
            return true;
        }

        if self.update_peak(klc.as_handle(), false) {
            return true;
        }
        _tmp_end != self.get_last_klu_of_last_bi()
    }

    pub fn delete_virtual_bi(&mut self) {
        if !self.bi_list.is_empty() && !self.bi_list.last().unwrap().is_sure {
            let sure_end_list = self.bi_list.last().unwrap().sure_end.clone();

            if !sure_end_list.is_empty() {
                let last_bi = self.bi_list.last_mut().unwrap();
                last_bi.restore_from_virtual_end(sure_end_list[0]);
                self.last_end = Some(last_bi.end_klc);

                for sure_end in &sure_end_list[1..] {
                    self.add_new_bi(self.last_end.unwrap(), *sure_end, true);
                    self.last_end = Some(self.bi_list.last().unwrap().end_klc);
                }
            } else {
                self.bi_list.pop();
            }

            //if self.bi_list.last().unwrap().is_virtual_end() {
            //    self.bi_list.last_mut().unwrap().restore_from_virtual_end();
            //} else {
            //    self.bi_list.pop();
            //}
        }

        self.last_end = if !self.bi_list.is_empty() {
            Some(self.bi_list.last().unwrap().end_klc)
        } else {
            None
        };

        // TODO: 这里要理解下，是不是破坏了Handle的Prev/Next假设
        // 应该没有，如果没有的话，CBi的prev/next就没有必要了
        //if !self.bi_list.is_empty() {
        //    self.bi_list.last_mut().unwrap().next = None;
        //}
    }

    // 已完备
    pub fn try_add_virtual_bi(&mut self, klc: &Candle, need_del_end: bool) -> bool {
        if need_del_end {
            self.delete_virtual_bi();
        }
        if self.bi_list.is_empty() {
            return false;
        }
        if klc.index() == self.bi_list.last().unwrap().end_klc.index() {
            return false;
        }

        if (self.bi_list.last().unwrap()._is_up()
            && klc.high >= self.bi_list.last().unwrap().end_klc.high)
            || (self.bi_list.last().unwrap()._is_down()
                && klc.low <= self.bi_list.last().unwrap().end_klc.low)
        {
            // 更新最后一笔
            self.bi_list
                .last_mut()
                .unwrap()
                .update_virtual_end(klc.as_handle());
            return true;
        }

        let mut tmp_klc = Some(klc.as_handle());
        while let Some(current_klc) = tmp_klc {
            if current_klc.index() <= self.bi_list.last().unwrap().end_klc.index() {
                break;
            }

            if self.can_make_bi(current_klc, self.bi_list.last().unwrap().end_klc, true) {
                // 新增一笔
                self.add_new_bi(self.last_end.unwrap(), current_klc, false);
                return true;
            }

            if self.update_peak(current_klc, true) {
                return true;
            }

            tmp_klc = current_klc.prev();
        }

        false
    }

    // 已完备
    fn add_new_bi(&mut self, pre_klc: Handle<Candle>, cur_klc: Handle<Candle>, is_sure: bool) {
        self.bi_list.push(CBi::new(
            &self.bi_list,
            pre_klc,
            cur_klc,
            self.bi_list.len(),
            is_sure,
        ));
    }

    // 已完备
    fn satisfy_bi_span(&self, klc: Handle<Candle>, last_end: Handle<Candle>) -> bool {
        let bi_span = self.get_klc_span(klc, last_end);
        if self.config.is_strict {
            return bi_span >= 4;
        }
        let mut uint_kl_cnt = 0;
        let mut tmp_klc = last_end.next();
        while let Some(current_klc) = tmp_klc {
            uint_kl_cnt += current_klc.lst.len();
            //  最后尾部虚笔的时候，可能klc.idx == last_end.idx+1
            if current_klc.next().is_none() {
                return false;
            }
            if current_klc.next().unwrap().index() < klc.index() {
                tmp_klc = current_klc.next();
            } else {
                break;
            }
        }
        bi_span >= 3 && uint_kl_cnt >= 3
    }

    // 已完备
    fn get_klc_span(&self, klc: Handle<Candle>, last_end: Handle<Candle>) -> usize {
        let mut span = klc.index() - last_end.index();

        if !self.config.gap_as_kl {
            return span;
        }

        if span >= 4 {
            // 加速运算，如果span需要真正精确的值，需要去掉这一行
            return span;
        }

        let mut tmp_klc = Some(last_end);
        while let Some(current_klc) = tmp_klc {
            if current_klc.index() >= klc.index() {
                break;
            }

            if current_klc.has_gap_with_next() {
                span += 1;
            }

            tmp_klc = current_klc.next();
        }
        span
    }

    // 已完备
    fn can_make_bi(
        &self,
        klc: Handle<Candle>,
        last_end: Handle<Candle>,
        for_virtual: bool,
    ) -> bool {
        let satisify_span = if self.config.bi_algo == "fx" {
            true
        } else {
            self.satisfy_bi_span(klc, last_end)
        };

        if !satisify_span {
            return false;
        }

        if !Candle::check_fx_valid(last_end, klc, self.config.bi_fx_check, for_virtual) {
            return false;
        }

        if self.config.bi_end_is_peak && !end_is_peak(last_end, klc) {
            return false;
        }
        true
    }

    // 已完备
    fn try_update_end(&mut self, klc: Handle<Candle>, for_virtual: bool) -> bool {
        fn check_top(klc: Handle<Candle>, for_virtual: bool) -> bool {
            if for_virtual {
                klc.dir == KlineDir::Up
            } else {
                klc.fx_type == FxType::Top
            }
        }

        fn check_bottom(klc: Handle<Candle>, for_virtual: bool) -> bool {
            if for_virtual {
                klc.dir == KlineDir::Down
            } else {
                klc.fx_type == FxType::Bottom
            }
        }

        if self.bi_list.is_empty() {
            return false;
        }

        let last_bi = self.bi_list.last_mut().unwrap();
        if (last_bi._is_up() && check_top(klc, for_virtual) && klc.high >= last_bi._get_end_val())
            || (last_bi._is_down()
                && check_bottom(klc, for_virtual)
                && klc.low <= last_bi._get_end_val())
        {
            if for_virtual {
                last_bi.update_virtual_end(klc);
            } else {
                last_bi.update_new_end(klc);
            }
            self.last_end = Some(klc);
            true
        } else {
            false
        }
    }

    // 已完备
    fn get_last_klu_of_last_bi(&self) -> Option<usize> {
        self.bi_list.last().map(|bi| bi._get_end_klu().index())
    }
}

fn end_is_peak(last_end: Handle<Candle>, cur_end: Handle<Candle>) -> bool {
    if last_end.fx_type == FxType::Bottom {
        let cmp_thred = cur_end.high; // 或者严格点选择get_klu_max_high()
        let mut klc = last_end.next();
        while let Some(k) = klc {
            if k.index() >= cur_end.index() {
                return true;
            }
            if k.high > cmp_thred {
                return false;
            }
            klc = k.next();
        }
    } else if last_end.fx_type == FxType::Top {
        let cmp_thred = cur_end.low; // 或者严格点选择get_klu_min_low()
        let mut klc = last_end.next();
        while let Some(k) = klc {
            if k.index() >= cur_end.index() {
                return true;
            }
            if k.low < cmp_thred {
                return false;
            }
            klc = k.next();
        }
    }
    true
}

impl std::ops::Deref for CBiList {
    type Target = Box<Vec<CBi>>;

    fn deref(&self) -> &Self::Target {
        &self.bi_list
    }
}

impl std::ops::DerefMut for CBiList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bi_list
    }
}

impl std::fmt::Display for CBiList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for bi in self.bi_list.iter() {
            writeln!(f, "{}", bi)?;
        }
        Ok(())
    }
}
