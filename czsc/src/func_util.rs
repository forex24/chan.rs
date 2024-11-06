/*fn kltype_lt_day(kl_type: &KlType) -> bool {
    matches!(
        kl_type,
        KlType::K1M | KlType::K5M | KlType::K15M | KlType::K30M | KlType::K60M
    )
}

fn kltype_lte_day(kl_type: &KlType) -> bool {
    matches!(
        kl_type,
        KlType::K1M | KlType::K5M | KlType::K15M | KlType::K30M | KlType::K60M | KlType::KDay
    )
}

fn check_kltype_order(type_list: Vec<KlType>) {
    let mut kl_type_order: HashMap<KlType, i32> = HashMap::new();
    kl_type_order.insert(KlType::K1M, 1);
    kl_type_order.insert(KlType::K3M, 2);
    kl_type_order.insert(KlType::K5M, 3);
    kl_type_order.insert(KlType::K15M, 4);
    kl_type_order.insert(KlType::K30M, 5);
    kl_type_order.insert(KlType::K60M, 6);
    kl_type_order.insert(KlType::KDay, 7);
    kl_type_order.insert(KlType::KWeek, 8);
    kl_type_order.insert(KlType::KMon, 9);
    kl_type_order.insert(KlType::KQuarter, 10);
    kl_type_order.insert(KlType::KYear, 11);

    let mut last_lv = i32::MAX;
    for kl_type in type_list {
        let cur_lv = *kl_type_order.get(&kl_type).expect("Invalid KlType");
        assert!(
            cur_lv < last_lv,
            "Type list must be ordered from higher to lower levels"
        );
        last_lv = cur_lv;
    }
}*/

use std::cmp::Ordering;

use crate::{EqualMode, IHighLow, KlineDir};

pub fn has_overlap(l1: f64, h1: f64, l2: f64, h2: f64, equal: bool) -> bool {
    if equal {
        h2 >= l1 && h1 >= l2
    } else {
        h2 > l1 && h1 > l2
    }
}

#[allow(dead_code)]
pub fn slice_has_overlap<T: IHighLow>(list: &[T]) -> bool {
    let min_high = list
        .iter()
        .map(|item| item.high())
        .reduce(f64::min)
        .unwrap();
    let max_low = list.iter().map(|item| item.low()).reduce(f64::max).unwrap();
    min_high > max_low
}

#[allow(dead_code)]
fn test_combine<T: IHighLow>(
    lhs: &T,
    rhs: &T,
    exclude_included: bool,
    allow_top_equal: Option<EqualMode>,
) -> KlineDir {
    let high_cmp = f64::total_cmp(&lhs.high(), &rhs.high());
    let low_cmp = f64::total_cmp(&lhs.low(), &rhs.low());
    match (high_cmp, low_cmp) {
        (Ordering::Greater | Ordering::Equal, Ordering::Less | Ordering::Equal) => {
            if allow_top_equal == Some(EqualMode::TopEqual)
                && lhs.high() == rhs.high()
                && lhs.low() > rhs.low()
            {
                return KlineDir::Down;
            }
            if allow_top_equal == Some(EqualMode::BottomEqual)
                && lhs.low() == rhs.low()
                && lhs.high() < rhs.high()
            {
                return KlineDir::Up;
            }
            if exclude_included {
                KlineDir::Included
            } else {
                KlineDir::Combine
            }
        }
        (Ordering::Less | Ordering::Equal, Ordering::Greater | Ordering::Equal) => {
            KlineDir::Combine
        }
        (Ordering::Greater, Ordering::Greater) => KlineDir::Down,
        (Ordering::Less, Ordering::Less) => KlineDir::Up,
    }
}
/*pub fn parse_inf(v: f64) -> String {
    if v.is_infinite() {
        if v.is_sign_positive() {
            "float::INFINITY".to_string()
        } else {
            "float::NEG_INFINITY".to_string()
        }
    } else {
        v.to_string()
    }
}
*/
