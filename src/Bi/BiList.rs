use crate::Bi::Bi::CBi;
use crate::Bi::BiConfig::CBiConfig;
use crate::Common::types::Handle;
use crate::Common::CEnum::{FxType, KLineDir};
use crate::KLine::KLine::CKLine;
use std::cell::RefCell;
use std::rc::Rc;

pub struct CBiList {
    pub bi_list: Vec<Handle<CBi>>,
    pub last_end: Option<Handle<CKLine>>,
    pub config: CBiConfig,
    pub free_klc_lst: Vec<Handle<CKLine>>,
}

impl CBiList {
    pub fn new(bi_conf: CBiConfig) -> Self {
        CBiList {
            bi_list: Vec::new(),
            last_end: None,
            config: bi_conf,
            free_klc_lst: Vec::new(),
        }
    }

    pub fn to_string(&self) -> String {
        self.bi_list
            .iter()
            .map(|bi| bi.borrow().to_string())
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub fn len(&self) -> usize {
        self.bi_list.len()
    }

    pub fn get(&self, index: usize) -> Option<Handle<CBi>> {
        self.bi_list.get(index).cloned()
    }

    pub fn try_create_first_bi(&mut self, klc: Handle<CKLine>) -> bool {
        for exist_free_klc in &self.free_klc_lst {
            if exist_free_klc.borrow().fx == klc.borrow().fx {
                continue;
            }
            if self.can_make_bi(Rc::clone(&klc), Rc::clone(exist_free_klc), false) {
                self.add_new_bi(Rc::clone(exist_free_klc), Rc::clone(&klc), true);
                self.last_end = Some(Rc::clone(&klc));
                return true;
            }
        }
        self.free_klc_lst.push(Rc::clone(&klc));
        self.last_end = Some(klc);
        false
    }

    pub fn update_bi(
        &mut self,
        klc: Handle<CKLine>,
        last_klc: Handle<CKLine>,
        cal_virtual: bool,
    ) -> bool {
        let flag1 = self.update_bi_sure(Rc::clone(&klc));
        if cal_virtual {
            let flag2 = self.try_add_virtual_bi(Rc::clone(&last_klc), false);
            flag1 || flag2
        } else {
            flag1
        }
    }

    pub fn can_update_peak(&self, klc: &Handle<CKLine>) -> bool {
        if self.config.bi_allow_sub_peak || self.bi_list.len() < 2 {
            return false;
        }
        let last_bi = self.bi_list.last().unwrap();
        let second_last_bi = &self.bi_list[self.bi_list.len() - 2];
        if last_bi.borrow().is_down() && klc.borrow().high < last_bi.borrow().get_begin_val() {
            return false;
        }
        if last_bi.borrow().is_up() && klc.borrow().low > last_bi.borrow().get_begin_val() {
            return false;
        }
        if !end_is_peak(&second_last_bi.borrow().begin_klc(), klc) {
            return false;
        }
        if last_bi.borrow().is_down()
            && last_bi.borrow().get_end_val() < second_last_bi.borrow().get_begin_val()
        {
            return false;
        }
        if last_bi.borrow().is_up()
            && last_bi.borrow().get_end_val() > second_last_bi.borrow().get_begin_val()
        {
            return false;
        }
        true
    }

    pub fn update_peak(&mut self, klc: Handle<CKLine>, for_virtual: bool) -> bool {
        if !self.can_update_peak(&klc) {
            return false;
        }
        let tmp_last_bi = self.bi_list.pop().unwrap();
        if !self.try_update_end(Rc::clone(&klc), for_virtual) {
            self.bi_list.push(tmp_last_bi);
            false
        } else {
            if for_virtual {
                self.bi_list
                    .last_mut()
                    .unwrap()
                    .borrow_mut()
                    .append_sure_end(tmp_last_bi.borrow().end_klc());
            }
            true
        }
    }

    pub fn update_bi_sure(&mut self, klc: Handle<CKLine>) -> bool {
        let tmp_end = self.get_last_klu_of_last_bi();
        self.delete_virtual_bi();
        if klc.borrow().fx == FxType::Unknown {
            return tmp_end != self.get_last_klu_of_last_bi();
        }
        if self.last_end.is_none() || self.bi_list.is_empty() {
            return self.try_create_first_bi(klc);
        }
        if klc.borrow().fx == self.last_end.as_ref().unwrap().borrow().fx {
            return self.try_update_end(klc, false);
        } else if self.can_make_bi(
            Rc::clone(&klc),
            Rc::clone(self.last_end.as_ref().unwrap()),
            false,
        ) {
            self.add_new_bi(
                Rc::clone(self.last_end.as_ref().unwrap()),
                Rc::clone(&klc),
                true,
            );
            self.last_end = Some(klc);
            return true;
        } else if self.update_peak(klc, false) {
            return true;
        }
        tmp_end != self.get_last_klu_of_last_bi()
    }

    pub fn delete_virtual_bi(&mut self) {
        if !self.bi_list.is_empty() && !self.bi_list.last().unwrap().borrow().is_sure {
            let sure_end_list: Vec<_> = self
                .bi_list
                .last()
                .unwrap()
                .borrow()
                .sure_end()
                .iter()
                .cloned()
                .collect();
            if !sure_end_list.is_empty() {
                self.bi_list
                    .last_mut()
                    .unwrap()
                    .borrow_mut()
                    .restore_from_virtual_end(Rc::clone(&sure_end_list[0]));
                self.last_end = Some(Rc::clone(&self.bi_list.last().unwrap().borrow().end_klc()));
                for sure_end in sure_end_list.iter().skip(1) {
                    self.add_new_bi(
                        Rc::clone(self.last_end.as_ref().unwrap()),
                        Rc::clone(sure_end),
                        true,
                    );
                    self.last_end =
                        Some(Rc::clone(&self.bi_list.last().unwrap().borrow().end_klc()));
                }
            } else {
                self.bi_list.pop();
            }
        }
        self.last_end = if !self.bi_list.is_empty() {
            Some(Rc::clone(&self.bi_list.last().unwrap().borrow().end_klc()))
        } else {
            None
        };
        if !self.bi_list.is_empty() {
            self.bi_list.last_mut().unwrap().borrow_mut().next = None;
        }
    }

    pub fn try_add_virtual_bi(&mut self, klc: Handle<CKLine>, need_del_end: bool) -> bool {
        if need_del_end {
            self.delete_virtual_bi();
        }
        if self.bi_list.is_empty() {
            return false;
        }
        if klc.borrow().idx == self.bi_list.last().unwrap().borrow().end_klc().borrow().idx {
            return false;
        }
        let last_bi = self.bi_list.last().unwrap();
        if (last_bi.borrow().is_up()
            && klc.borrow().high >= last_bi.borrow().end_klc().borrow().high)
            || (last_bi.borrow().is_down()
                && klc.borrow().low <= last_bi.borrow().end_klc().borrow().low)
        {
            last_bi.borrow_mut().update_virtual_end(Rc::clone(&klc));
            return true;
        }
        let mut tmp_klc = Some(Rc::clone(&klc));
        while let Some(k) = tmp_klc {
            if k.borrow().idx <= self.bi_list.last().unwrap().borrow().end_klc().borrow().idx {
                break;
            }
            if self.can_make_bi(
                Rc::clone(&k),
                Rc::clone(&self.bi_list.last().unwrap().borrow().end_klc()),
                true,
            ) {
                self.add_new_bi(
                    Rc::clone(self.last_end.as_ref().unwrap()),
                    Rc::clone(&k),
                    false,
                );
                return true;
            } else if self.update_peak(Rc::clone(&k), true) {
                return true;
            }
            tmp_klc = k.borrow().pre.clone();
        }
        false
    }

    pub fn add_new_bi(&mut self, pre_klc: Handle<CKLine>, cur_klc: Handle<CKLine>, is_sure: bool) {
        let new_bi = Rc::new(RefCell::new(CBi::new(
            pre_klc,
            cur_klc,
            self.bi_list.len(),
            is_sure,
        )));
        if !self.bi_list.is_empty() {
            let last_bi = self.bi_list.last_mut().unwrap();
            last_bi.borrow_mut().next = Some(Rc::clone(&new_bi));
            new_bi.borrow_mut().pre = Some(Rc::clone(last_bi));
        }
        self.bi_list.push(new_bi);
    }

    pub fn satisfy_bi_span(&self, klc: &Handle<CKLine>, last_end: &Handle<CKLine>) -> bool {
        let bi_span = self.get_klc_span(klc, last_end);
        if self.config.is_strict {
            return bi_span >= 4;
        }
        let mut uint_kl_cnt = 0;
        let mut tmp_klc = last_end.borrow().next.clone();
        while let Some(k) = tmp_klc {
            uint_kl_cnt += k.borrow().lst.len();
            if k.borrow().next.is_none() {
                return false;
            }
            if k.borrow().next.as_ref().unwrap().borrow().idx < klc.borrow().idx {
                tmp_klc = k.borrow().next.clone();
            } else {
                break;
            }
        }
        bi_span >= 3 && uint_kl_cnt >= 3
    }

    pub fn get_klc_span(&self, klc: &Handle<CKLine>, last_end: &Handle<CKLine>) -> usize {
        let mut span = klc.borrow().idx - last_end.borrow().idx;
        if !self.config.gap_as_kl {
            return span;
        }
        if span >= 4 {
            return span;
        }
        let mut tmp_klc = Some(Rc::clone(last_end));
        while let Some(k) = tmp_klc {
            if k.borrow().idx >= klc.borrow().idx {
                break;
            }
            if k.borrow().has_gap_with_next() {
                span += 1;
            }
            tmp_klc = k.borrow().next.clone();
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
        let check_fx_valid =
            last_end
                .borrow()
                .check_fx_valid(&klc, self.config.bi_fx_check, for_virtual);
        if check_fx_valid.is_err() {
            return false;
        }
        if self.config.bi_end_is_peak && !end_is_peak(&last_end, &klc) {
            return false;
        }
        true
    }

    pub fn try_update_end(&mut self, klc: Handle<CKLine>, for_virtual: bool) -> bool {
        if self.bi_list.is_empty() {
            return false;
        }
        let last_bi = self.bi_list.last().unwrap();
        let check_top = |k: &Handle<CKLine>, for_virtual: bool| -> bool {
            if for_virtual {
                k.borrow().dir == KLineDir::Up
            } else {
                k.borrow().fx == FxType::Top
            }
        };
        let check_bottom = |k: &Handle<CKLine>, for_virtual: bool| -> bool {
            if for_virtual {
                k.borrow().dir == KLineDir::Down
            } else {
                k.borrow().fx == FxType::Bottom
            }
        };
        if (last_bi.borrow().is_up()
            && check_top(&klc, for_virtual)
            && klc.borrow().high >= last_bi.borrow().get_end_val())
            || (last_bi.borrow().is_down()
                && check_bottom(&klc, for_virtual)
                && klc.borrow().low <= last_bi.borrow().get_end_val())
        {
            if for_virtual {
                last_bi.borrow_mut().update_virtual_end(Rc::clone(&klc));
            } else {
                last_bi.borrow_mut().update_new_end(Rc::clone(&klc));
            }
            self.last_end = Some(Rc::clone(&klc));
            true
        } else {
            false
        }
    }

    pub fn get_last_klu_of_last_bi(&self) -> Option<usize> {
        self.bi_list
            .last()
            .map(|bi| bi.borrow().get_end_klu().borrow().idx)
    }
}

fn end_is_peak(last_end: &Handle<CKLine>, cur_end: &Handle<CKLine>) -> bool {
    match last_end.borrow().fx {
        FxType::Bottom => {
            let cmp_thred = cur_end.borrow().high;
            let mut klc = last_end.borrow().get_next();
            while let Some(k) = klc {
                if k.borrow().idx >= cur_end.borrow().idx {
                    return true;
                }
                if k.borrow().high > cmp_thred {
                    return false;
                }
                klc = k.borrow().get_next();
            }
        }
        FxType::Top => {
            let cmp_thred = cur_end.borrow().low;
            let mut klc = last_end.borrow().get_next();
            while let Some(k) = klc {
                if k.borrow().idx >= cur_end.borrow().idx {
                    return true;
                }
                if k.borrow().low < cmp_thred {
                    return false;
                }
                klc = k.borrow().get_next();
            }
        }
        _ => {}
    }
    true
}

impl std::ops::Deref for CBiList {
    type Target = Vec<Handle<CBi>>;

    fn deref(&self) -> &Self::Target {
        &self.bi_list
    }
}

impl std::ops::DerefMut for CBiList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bi_list
    }
}
