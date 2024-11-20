use pyo3::prelude::*;
use pyo3::types::PyDict;

impl ToPyObject for Candle {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        let dict = PyDict::new(py);
        dict.set_item("time", self.time).unwrap();
        dict.set_item("open", self.open).unwrap();
        dict.set_item("high", self.high).unwrap();
        dict.set_item("low", self.low).unwrap();
        dict.set_item("close", self.close).unwrap();
        dict.set_item("volume", self.volume).unwrap();
        dict.into()
    }
}

// 为其他类型实现类似的转换...
