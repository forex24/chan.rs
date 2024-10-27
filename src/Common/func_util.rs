// File: chan/src/common/func_util.rs

use super::CEnum::{BiDir, KlType};

pub fn kltype_lt_day(kl_type: &KlType) -> bool {
    (*kl_type as i32) < (KlType::K_DAY as i32)
}

pub fn kltype_lte_day(kl_type: &KlType) -> bool {
    *kl_type as i32 <= KlType::K_DAY as i32
}

pub fn check_kltype_order(type_list: &[KlType]) -> Result<(), String> {
    let mut last_lv = type_list[0] as i32;
    for kl_type in &type_list[1..] {
        if *kl_type as i32 >= last_lv {
            return Err("lv_list的顺序必须从大级别到小级别".to_string());
        }
        last_lv = *kl_type as i32;
    }
    Ok(())
}

pub fn revert_bi_dir(dir: &BiDir) -> BiDir {
    match dir {
        BiDir::Up => BiDir::Down,
        BiDir::Down => BiDir::Up,
    }
}

pub fn has_overlap(l1: f64, h1: f64, l2: f64, h2: f64, equal: bool) -> bool {
    if equal {
        h2 >= l1 && h1 >= l2
    } else {
        h2 > l1 && h1 > l2
    }
}

pub fn str2float(s: &str) -> f64 {
    s.parse::<f64>().unwrap_or(0.0)
}

pub fn parse_inf(v: f64) -> String {
    if v.is_infinite() {
        if v.is_sign_positive() {
            "f64::INFINITY".to_string()
        } else {
            "f64::NEG_INFINITY".to_string()
        }
    } else {
        v.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /*#[test]
    fn test_kltype_lt_day() {
        assert!(kltype_lt_day(&KlType::K_1M));
        assert!(!kltype_lt_day(&KlType::K_DAY));
        assert!(!kltype_lt_day(&KlType::K_WEEK));
    }

    #[test]
    fn test_kltype_lte_day() {
        assert!(kltype_lte_day(&KlType::K_1M));
        assert!(kltype_lte_day(&KlType::K_DAY));
        assert!(!kltype_lte_day(&KlType::K_WEEK));
    }

    #[test]
    fn test_check_kltype_order() {
        let valid_order = vec![KlType::K_WEEK, KlType::K_DAY, KlType::K_1M];
        assert!(check_kltype_order(&valid_order).is_ok());

        let invalid_order = vec![KlType::K_DAY, KlType::K_WEEK, KlType::K_1M];
        assert!(check_kltype_order(&invalid_order).is_err());
    }
    */
    #[test]
    fn test_revert_bi_dir() {
        assert_eq!(revert_bi_dir(&BiDir::Up), BiDir::Down);
        assert_eq!(revert_bi_dir(&BiDir::Down), BiDir::Up);
    }

    #[test]
    fn test_has_overlap() {
        assert!(has_overlap(1.0, 3.0, 2.0, 4.0, false));
        assert!(has_overlap(1.0, 3.0, 2.0, 4.0, true));
        assert!(!has_overlap(1.0, 2.0, 3.0, 4.0, false));
        assert!(!has_overlap(1.0, 2.0, 3.0, 4.0, true));
        assert!(has_overlap(1.0, 3.0, 3.0, 4.0, true));
        assert!(!has_overlap(1.0, 3.0, 3.0, 4.0, false));
    }

    //#[test]
    //fn test_str2float() {
    //    assert_eq!(str2float("3.14"), 3.14);
    //    assert_eq!(str2float("invalid"), 0.0);
    //}
    //
    //#[test]
    //fn test_parse_inf() {
    //    assert_eq!(parse_inf(f64::INFINITY), "f64::INFINITY");
    //    assert_eq!(parse_inf(f64::NEG_INFINITY), "f64::NEG_INFINITY");
    //    assert_eq!(parse_inf(3.14), "3.14");
    //}
}
