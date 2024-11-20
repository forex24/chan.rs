use chrono::{DateTime, NaiveDateTime, Utc};
use pyo3::prelude::*;
use pyo3::types::{PyDateTime, PyDict, PyList};
use std::collections::HashMap;

/// 将 Python 字典转换为 Rust HashMap
pub fn py_dict_to_hashmap(dict: &PyDict) -> PyResult<HashMap<String, f64>> {
    let mut map = HashMap::new();
    for (key, value) in dict.iter() {
        let key = key.extract::<String>()?;
        let value = value.extract::<f64>()?;
        map.insert(key, value);
    }
    Ok(map)
}

/// 将 Rust HashMap 转换为 Python 字典
pub fn hashmap_to_py_dict(py: Python, map: &HashMap<String, f64>) -> PyResult<PyObject> {
    let dict = PyDict::new(py);
    for (key, value) in map {
        dict.set_item(key, value)?;
    }
    Ok(dict.into())
}

/// 将 Python datetime 转换为 Rust NaiveDateTime
pub fn py_datetime_to_naive(dt: &PyDateTime) -> PyResult<NaiveDateTime> {
    let timestamp = dt.call_method0("timestamp")?.extract::<i64>()?;
    Ok(NaiveDateTime::from_timestamp_opt(timestamp, 0)
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>("Invalid timestamp"))?)
}

/// 将 Rust NaiveDateTime 转换为 Python datetime
pub fn naive_to_py_datetime(py: Python, dt: NaiveDateTime) -> PyResult<PyObject> {
    let timestamp = dt.timestamp();
    let datetime_type = py.import("datetime")?.getattr("datetime")?;
    let args = (timestamp,);
    datetime_type
        .call_method1("fromtimestamp", args)?
        .into_py(py)
}

/// 将 Python K线数据字典转换为 Rust KLine 结构
pub fn py_dict_to_kline(dict: &PyDict) -> PyResult<KLine> {
    let time_str = dict
        .get_item("time")
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("Missing 'time' field"))?
        .extract::<String>()?;

    let time = NaiveDateTime::parse_from_str(&time_str, "%Y-%m-%d %H:%M:%S")
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

    Ok(KLine {
        time,
        open: dict
            .get_item("open")
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("Missing 'open' field"))?
            .extract()?,
        high: dict
            .get_item("high")
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("Missing 'high' field"))?
            .extract()?,
        low: dict
            .get_item("low")
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("Missing 'low' field"))?
            .extract()?,
        close: dict
            .get_item("close")
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("Missing 'close' field"))?
            .extract()?,
        volume: dict
            .get_item("volume")
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("Missing 'volume' field"))?
            .extract()?,
    })
}

/// 将 Rust KLine 结构转换为 Python 字典
pub fn kline_to_py_dict(py: Python, kline: &KLine) -> PyResult<PyObject> {
    let dict = PyDict::new(py);
    dict.set_item("time", kline.time.format("%Y-%m-%d %H:%M:%S").to_string())?;
    dict.set_item("open", kline.open)?;
    dict.set_item("high", kline.high)?;
    dict.set_item("low", kline.low)?;
    dict.set_item("close", kline.close)?;
    dict.set_item("volume", kline.volume)?;
    Ok(dict.into())
}

/// 将 Python 列表转换为 Rust Vec
pub fn py_list_to_vec<T>(list: &PyList) -> PyResult<Vec<T>>
where
    T: for<'a> pyo3::FromPyObject<'a>,
{
    list.iter().map(|item| item.extract()).collect()
}

/// 将 Rust Vec 转换为 Python 列表
pub fn vec_to_py_list<T>(py: Python, vec: &[T]) -> PyResult<PyObject>
where
    T: pyo3::ToPyObject,
{
    let list = PyList::empty(py);
    for item in vec {
        list.append(item.to_object(py))?;
    }
    Ok(list.into())
}

/// 验证 K线数据的有效性
pub fn validate_kline(kline: &KLine) -> PyResult<()> {
    if kline.high < kline.low {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "High price cannot be lower than low price",
        ));
    }
    if kline.open < kline.low || kline.open > kline.high {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "Open price must be between high and low prices",
        ));
    }
    if kline.close < kline.low || kline.close > kline.high {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "Close price must be between high and low prices",
        ));
    }
    if kline.volume < 0.0 {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "Volume cannot be negative",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pyo3::Python;

    #[test]
    fn test_hashmap_conversion() {
        Python::with_gil(|py| {
            let mut map = HashMap::new();
            map.insert("test".to_string(), 1.0);

            let py_dict = hashmap_to_py_dict(py, &map).unwrap();
            let dict = py_dict.downcast::<PyDict>(py).unwrap();

            let converted_map = py_dict_to_hashmap(dict).unwrap();
            assert_eq!(map, converted_map);
        });
    }

    #[test]
    fn test_kline_conversion() {
        Python::with_gil(|py| {
            let dict = PyDict::new(py);
            dict.set_item("time", "2023-01-01 00:00:00").unwrap();
            dict.set_item("open", 100.0).unwrap();
            dict.set_item("high", 105.0).unwrap();
            dict.set_item("low", 95.0).unwrap();
            dict.set_item("close", 103.0).unwrap();
            dict.set_item("volume", 1000.0).unwrap();

            let kline = py_dict_to_kline(dict).unwrap();
            let converted_dict = kline_to_py_dict(py, &kline).unwrap();

            assert!(converted_dict.is_instance_of::<PyDict>());
        });
    }

    #[test]
    fn test_validate_kline() {
        let valid_kline = KLine {
            time: NaiveDateTime::parse_from_str("2023-01-01 00:00:00", "%Y-%m-%d %H:%M:%S")
                .unwrap(),
            open: 100.0,
            high: 105.0,
            low: 95.0,
            close: 103.0,
            volume: 1000.0,
        };
        assert!(validate_kline(&valid_kline).is_ok());

        let invalid_kline = KLine {
            time: NaiveDateTime::parse_from_str("2023-01-01 00:00:00", "%Y-%m-%d %H:%M:%S")
                .unwrap(),
            open: 100.0,
            high: 95.0, // Invalid: high < low
            low: 105.0,
            close: 103.0,
            volume: 1000.0,
        };
        assert!(validate_kline(&invalid_kline).is_err());
    }
}
