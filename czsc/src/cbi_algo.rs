use crate::{BiAlgo, CBiConfig, Candle, FxType, Handle};

/// 判断两个分型之间的K线数量是否满足笔的要求
///
/// # Arguments
/// * `end_fx` - 结束分型的k2
/// * `start_fx` - 起始分型的k2
/// * `config` - 笔的配置参数
///
/// # Returns
/// * `bool` - 是否满足笔的跨度要求
///
/// 在严格模式下，要求跨度至少为4，非严格模式下要求：
/// 1. 跨度至少为3
/// 2. 实际K线数量至少为3
pub(crate) fn satisfy_bi_span(
    end_fx: Handle<Candle>,
    start_fx: Handle<Candle>,
    config: &CBiConfig,
) -> bool {
    // 计算两个分型之间的K线跨度
    let bi_span = get_klc_span(end_fx, start_fx, config.gap_as_kl);

    // 严格模式下直接判断跨度是否大于等于4
    if config.is_strict {
        return bi_span >= 4;
    }

    // 非严格模式下，计算实际K线数量
    let mut uint_kl_cnt = 0;
    let mut tmp_klc = start_fx.next();
    while let Some(current_klc) = tmp_klc {
        // 累加每个K线包含的实际K线数量
        uint_kl_cnt += current_klc.lst.len();

        // 如果到达最后一根K线，返回false
        if current_klc.next().is_none() {
            return false;
        }

        // 如果下一根K线的索引小于结束分型的索引，继续遍历
        if current_klc.next().unwrap().index() < end_fx.index() {
            tmp_klc = current_klc.next();
        } else {
            break;
        }
    }

    // 返回是否同时满足跨度和实际K线数量的要求
    bi_span >= 3 && uint_kl_cnt >= 3
}

/// 计算两个分型之间的K线跨度
///
/// # Arguments
/// * `end_fx` - 结束分型的k2
/// * `start_fx` - 起始分型的k2
/// * `gap_as_kl` - 是否将缺口计入K线跨度
///
/// # Returns
/// * `usize` - K线跨度数量
///
/// 如果gap_as_kl为true，则直接返回索引差；
/// 否则需要考虑K线之间的缺口，每个缺口额外计数1
pub(crate) fn get_klc_span(
    end_fx: Handle<Candle>,
    start_fx: Handle<Candle>,
    gap_as_kl: bool,
) -> usize {
    // 计算基础跨度（索引差）
    let mut span = end_fx.index() - start_fx.index();

    // 如果不需要考虑缺口，直接返回索引差
    if gap_as_kl {
        return span;
    }

    // 优化：如果跨度已经>=4，无需计算缺口
    if span >= 4 {
        return span;
    }

    // 遍历K线，统计缺口数量
    let mut tmp_klc = Some(start_fx);
    while let Some(current_klc) = tmp_klc {
        // 到达结束分型，停止遍历
        if current_klc.index() >= end_fx.index() {
            break;
        }

        // 如果与下一根K线有缺口，跨度加1
        if current_klc.has_gap_with_next() {
            span += 1;
        }

        tmp_klc = current_klc.next();
    }
    span
}

/// 判断是否可以构成笔
///
/// # Arguments
/// * `end_fx` - 结束分型的k2
/// * `start_fx` - 起始分型的k2
/// * `for_virtual` - 是否为虚笔判断
/// * `config` - 笔的配置参数
///
/// # Returns
/// * `bool` - 是否可以构成笔
///
/// 需要同时满足以下条件：
/// 1. 如果不是Fx算法，需要满足跨度要求
/// 2. 分型检查有效
/// 3. 如果配置了bi_end_is_peak，需要满足端点是峰值的要求
#[allow(dead_code)]
pub(crate) fn can_make_bi(
    end_fx: Handle<Candle>,
    start_fx: Handle<Candle>,
    for_virtual: bool,
    config: &CBiConfig,
) -> bool {
    // 判断是否满足跨度要求（Fx算法不需要判断）
    let satisify_span = if config.bi_algo == BiAlgo::Fx {
        true
    } else {
        satisfy_bi_span(end_fx, start_fx, config)
    };

    if !satisify_span {
        return false;
    }

    // 检查分型的有效性
    if !Candle::check_fx_valid(start_fx, end_fx, config.bi_fx_check, for_virtual) {
        return false;
    }

    // 如果配置了bi_end_is_peak，检查端点是否为峰值
    if config.bi_end_is_peak && !end_is_peak(start_fx, end_fx) {
        return false;
    }
    true
}

/// 判断结束分型是否为峰值
///
/// # Arguments
/// * `last_end` - 起始分型
/// * `cur_end` - 结束分型
///
/// # Returns
/// * `bool` - 是否为峰值
///
/// 对于底分型，要求中间的K线高点都不超过结束分型的高点
/// 对于顶分型，要求中间的K线低点都不低于结束分型的低点
fn end_is_peak(last_end: Handle<Candle>, cur_end: Handle<Candle>) -> bool {
    if last_end.fx_type == FxType::Bottom {
        // 对于底分型，比较高点
        let cmp_thred = cur_end.high;
        let mut klc = last_end.next();
        while let Some(k) = klc {
            if k.index() >= cur_end.index() {
                return true;
            }
            // 如果中间有K线的高点超过了结束分型的高点，不是峰值
            if k.high > cmp_thred {
                return false;
            }
            klc = k.next();
        }
    } else if last_end.fx_type == FxType::Top {
        // 对于顶分型，比较低点
        let cmp_thred = cur_end.low;
        let mut klc = last_end.next();
        while let Some(k) = klc {
            if k.index() >= cur_end.index() {
                return true;
            }
            // 如果中间有K线的低点低于了结束分型的低点，不是峰值
            if k.low < cmp_thred {
                return false;
            }
            klc = k.next();
        }
    }
    true
}
