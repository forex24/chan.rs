use crate::{BiAlgo, CBiConfig, Candle, FxType, Handle};

// 已完备
pub(crate) fn satisfy_bi_span(
    end_fx: Handle<Candle>,
    start_fx: Handle<Candle>,
    config: &CBiConfig,
) -> bool {
    let bi_span = get_klc_span(end_fx, start_fx, config.gap_as_kl);
    //if self.config.is_strict {
    if config.is_strict {
        return bi_span >= 4;
    }
    let mut uint_kl_cnt = 0;
    let mut tmp_klc = start_fx.next();
    while let Some(current_klc) = tmp_klc {
        uint_kl_cnt += current_klc.lst.len();
        //  最后尾部虚笔的时候，可能klc.idx == last_end.idx+1
        if current_klc.next().is_none() {
            return false;
        }
        if current_klc.next().unwrap().index() < end_fx.index() {
            tmp_klc = current_klc.next();
        } else {
            break;
        }
    }
    bi_span >= 3 && uint_kl_cnt >= 3
}

// 已完备
pub(crate) fn get_klc_span(
    end_fx: Handle<Candle>,
    start_fx: Handle<Candle>,
    gap_as_kl: bool,
) -> usize {
    let mut span = end_fx.index() - start_fx.index();

    //if !self.config.gap_as_kl {
    if gap_as_kl {
        return span;
    }

    // 加速运算，如果span需要真正精确的值，需要去掉这一行
    if span >= 4 {
        return span;
    }

    let mut tmp_klc = Some(start_fx);
    while let Some(current_klc) = tmp_klc {
        if current_klc.index() >= end_fx.index() {
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
// klc:结束分型的k2
// last_end 起始分型的k2
#[allow(dead_code)]
pub(crate) fn can_make_bi(
    end_fx: Handle<Candle>,
    start_fx: Handle<Candle>,
    for_virtual: bool,
    config: &CBiConfig,
) -> bool {
    let satisify_span = if config.bi_algo == BiAlgo::Fx {
        true
    } else {
        satisfy_bi_span(end_fx, start_fx, config)
    };

    if !satisify_span {
        return false;
    }

    if !Candle::check_fx_valid(start_fx, end_fx, config.bi_fx_check, for_virtual) {
        return false;
    }

    if config.bi_end_is_peak && !end_is_peak(start_fx, end_fx) {
        return false;
    }
    true
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
