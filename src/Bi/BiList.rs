use crate::Bi::Bi::CBi;
use crate::Bi::BiConfig::CBiConfig;
use crate::Common::handle::AsHandle;
use crate::Common::handle::Handle;
use crate::Common::handle::Indexable;
use crate::Common::CEnum::{FxType, KLineDir};
use crate::KLine::KLine::CKLine;
pub struct CBiList {
    pub bi_list: Box<Vec<CBi>>,
    pub last_end: Option<Handle<CKLine>>,
    pub config: CBiConfig,
    pub free_klc_lst: Vec<Handle<CKLine>>,
}

impl CBiList {
    pub fn new(bi_conf: CBiConfig) -> Self {
        CBiList {
            bi_list: Box::<Vec<CBi>>::default(),
            last_end: None,
            config: bi_conf,
            free_klc_lst: Vec::new(),
        }
    }

    pub fn try_create_first_bi(&mut self, klc: &CKLine) -> bool {
        assert!(self.bi_list.is_empty());
        assert!(klc.fx != FxType::Unknown);

        for exist_free_klc in &self.free_klc_lst {
            if exist_free_klc.fx == klc.fx {
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

    pub fn update_bi(&mut self, klc: &CKLine, last_klc: &CKLine, cal_virtual: bool) -> bool {
        // klc:倒数第二根klc
        // last_klc: 倒数第1根klc
        let flag1 = self.update_bi_sure(klc);
        if cal_virtual {
            let flag2 = self.try_add_virtual_bi(&last_klc, false);
            flag1 || flag2
        } else {
            flag1
        }
    }

    pub fn can_update_peak(&self, klc: Handle<CKLine>) -> bool {
        if self.config.bi_allow_sub_peak || self.bi_list.len() < 2 {
            return false;
        }

        let last_bi = self.bi_list.last().unwrap();
        if last_bi.is_down() && klc.high < last_bi.get_begin_val() {
            return false;
        }
        if last_bi.is_up() && klc.low > last_bi.get_begin_val() {
            return false;
        }

        let second_last_bi = &self.bi_list[self.bi_list.len() - 2];
        if !end_is_peak(second_last_bi.begin_klc, klc.as_handle()) {
            return false;
        }
        if last_bi.is_down() && last_bi.get_end_val() < second_last_bi.get_begin_val() {
            return false;
        }
        if last_bi.is_up() && last_bi.get_end_val() > second_last_bi.get_begin_val() {
            return false;
        }
        true
    }

    pub fn update_peak(&mut self, klc: Handle<CKLine>, for_virtual: bool) -> bool {
        if !self.can_update_peak(klc) {
            return false;
        }
        let tmp_last_bi = self.bi_list.pop().unwrap();
        if !self.try_update_end(klc, for_virtual) {
            self.bi_list.push(tmp_last_bi);
            false
        } else {
            if for_virtual {
                self.bi_list
                    .last_mut()
                    .unwrap()
                    .append_sure_end(tmp_last_bi.end_klc());
            }
            true
        }
    }

    pub fn update_bi_sure(&mut self, klc: &CKLine) -> bool {
        // klc:倒数第二根klc
        let tmp_end = self.get_last_klu_of_last_bi();
        self.delete_virtual_bi();

        // 返回值：是否出现新笔
        if klc.fx == FxType::Unknown {
            return tmp_end != self.get_last_klu_of_last_bi(); // 虚笔是否有变
        }
        if self.last_end.is_none() || self.bi_list.is_empty() {
            return self.try_create_first_bi(klc);
        }
        if klc.fx == self.last_end.as_ref().unwrap().fx {
            return self.try_update_end(klc.as_handle(), false);
        } else if self.can_make_bi(klc.as_handle(), self.last_end.unwrap(), false) {
            self.add_new_bi(self.last_end.unwrap(), klc.as_handle(), true);
            self.last_end = Some(klc.as_handle());
            return true;
        } else if self.update_peak(klc.as_handle(), false) {
            return true;
        }
        tmp_end != self.get_last_klu_of_last_bi()
    }

    pub fn delete_virtual_bi(&mut self) {
        if !self.bi_list.is_empty() && !self.bi_list.last().unwrap().is_sure {
            let sure_end_list = self.bi_list.last().unwrap().sure_end.to_vec();
            if !sure_end_list.is_empty() {
                self.bi_list
                    .last_mut()
                    .unwrap()
                    .restore_from_virtual_end(sure_end_list[0]);
                self.last_end = Some(self.bi_list.last().unwrap().end_klc);
                for sure_end in sure_end_list.iter().skip(1) {
                    self.add_new_bi(self.last_end.unwrap(), *sure_end, true);
                    self.last_end = Some(self.bi_list.last().unwrap().end_klc);
                }
            } else {
                self.bi_list.pop();
            }
        }
        self.last_end = if !self.bi_list.is_empty() {
            Some(self.bi_list.last().unwrap().end_klc())
        } else {
            None
        };
        if !self.bi_list.is_empty() {
            self.bi_list.last_mut().unwrap().next = None;
        }
    }

    pub fn try_add_virtual_bi(&mut self, klc: &CKLine, need_del_end: bool) -> bool {
        if need_del_end {
            self.delete_virtual_bi();
        }
        if self.bi_list.is_empty() {
            return false;
        }
        if klc.index() == self.bi_list.last().unwrap().end_klc().index() {
            return false;
        }

        let last_bi = self.bi_list.last_mut().unwrap();

        if (last_bi.is_up() && klc.high >= last_bi.end_klc().high)
            || (last_bi.is_down() && klc.low <= last_bi.end_klc().low)
        {
            // 更新最后一笔
            last_bi.update_virtual_end(klc.as_handle());
            return true;
        }

        let mut tmp_klc = Some(klc.as_handle());
        while let Some(k) = tmp_klc {
            if k.index() <= self.bi_list.last().unwrap().end_klc().idx {
                break;
            }
            if self.can_make_bi(k, self.bi_list.last().unwrap().end_klc, true) {
                self.add_new_bi(self.last_end.unwrap(), k, false);
                return true;
            } else if self.update_peak(k, true) {
                return true;
            }
            tmp_klc = k.pre;
        }
        false
    }

    pub fn add_new_bi(&mut self, pre_klc: Handle<CKLine>, cur_klc: Handle<CKLine>, is_sure: bool) {
        let mut new_bi = CBi::new(&self.bi_list, pre_klc, cur_klc, self.bi_list.len(), is_sure);
        if !self.bi_list.is_empty() {
            let last_bi = self.bi_list.last_mut().unwrap();
            last_bi.next = Some(new_bi.as_handle());
            new_bi.pre = Some(last_bi.as_handle());
        }
        self.bi_list.push(new_bi);
    }

    pub fn satisfy_bi_span(&self, klc: &Handle<CKLine>, last_end: &Handle<CKLine>) -> bool {
        let bi_span = self.get_klc_span(klc, last_end);
        if self.config.is_strict {
            return bi_span >= 4;
        }
        let mut uint_kl_cnt = 0;
        let mut tmp_klc = last_end.next.clone();
        while let Some(k) = tmp_klc {
            uint_kl_cnt += k.lst.len();
            if k.next.is_none() {
                return false;
            }
            if k.next.as_ref().unwrap().idx < klc.idx {
                tmp_klc = k.next.clone();
            } else {
                break;
            }
        }
        bi_span >= 3 && uint_kl_cnt >= 3
    }

    pub fn get_klc_span(&self, klc: &Handle<CKLine>, last_end: &Handle<CKLine>) -> usize {
        let mut span = klc.idx - last_end.idx;
        if !self.config.gap_as_kl {
            return span;
        }
        if span >= 4 {
            // 加速计算
            return span;
        }
        let mut tmp_klc = Some(last_end.as_handle());
        while let Some(k) = tmp_klc {
            if k.idx >= klc.idx {
                break;
            }
            if k.has_gap_with_next() {
                span += 1;
            }
            tmp_klc = k.next;
        }
        span
    }

    pub fn can_make_bi(
        &self,
        klc: Handle<CKLine>,
        last_end: Handle<CKLine>,
        for_virtual: bool,
    ) -> bool {
        let satisfy_span = if self.config.bi_algo == "fx" {
            true
        } else {
            self.satisfy_bi_span(&klc, &last_end)
        };
        if !satisfy_span {
            return false;
        }
        let check_fx_valid = last_end.check_fx_valid(&klc, self.config.bi_fx_check, for_virtual);
        if check_fx_valid.is_err() {
            return false;
        }
        if self.config.bi_end_is_peak && !end_is_peak(last_end, klc) {
            return false;
        }
        true
    }

    pub fn try_update_end(&mut self, klc: Handle<CKLine>, for_virtual: bool) -> bool {
        if self.bi_list.is_empty() {
            return false;
        }

        fn check_top(klc: Handle<CKLine>, for_virtual: bool) -> bool {
            if for_virtual {
                klc.dir == KLineDir::Up
            } else {
                klc.fx == FxType::Top
            }
        }

        fn check_bottom(klc: Handle<CKLine>, for_virtual: bool) -> bool {
            if for_virtual {
                klc.dir == KLineDir::Down
            } else {
                klc.fx == FxType::Bottom
            }
        }

        let last_bi = self.bi_list.last_mut().unwrap();
        if (last_bi.is_up() && check_top(klc, for_virtual) && klc.high >= last_bi.get_end_val())
            || (last_bi.is_down()
                && check_bottom(klc, for_virtual)
                && klc.low <= last_bi.get_end_val())
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

    pub fn get_last_klu_of_last_bi(&self) -> Option<usize> {
        self.bi_list.last().map(|bi| bi.get_end_klu().idx)
    }
}

fn end_is_peak(last_end: Handle<CKLine>, cur_end: Handle<CKLine>) -> bool {
    match last_end.fx {
        FxType::Bottom => {
            let cmp_thred = cur_end.high;
            let mut klc = last_end.next();
            while let Some(k) = klc {
                if k.idx >= cur_end.idx {
                    return true;
                }
                if k.high > cmp_thred {
                    return false;
                }
                klc = k.next();
            }
        }
        FxType::Top => {
            let cmp_thred = cur_end.low;
            let mut klc = last_end.next();
            while let Some(k) = klc {
                if k.idx >= cur_end.idx {
                    return true;
                }
                if k.low < cmp_thred {
                    return false;
                }
                klc = k.next();
            }
        }
        _ => {}
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
