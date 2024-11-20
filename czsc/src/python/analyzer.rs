use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use crate::analyzer::Analyzer;
use crate::chan::CChanConfig;

#[pyclass(name = "Analyzer")]
pub struct PyAnalyzer {
    inner: Analyzer,
}

#[pymethods]
impl PyAnalyzer {
    #[new]
    #[pyo3(signature = (step_calculation=1, config=None))]
    fn new(step_calculation: usize, config: Option<PyObject>) -> PyResult<Self> {
        let chan_config = if let Some(py_config) = config {
            CChanConfig::from_pyobject(py_config)?
        } else {
            CChanConfig::default()
        };
        
        Ok(Self {
            inner: Analyzer::new(step_calculation, chan_config)
        })
    }

    /// 添加新的K线数据
    #[pyo3(name = "add_k")]
    fn py_add_k(&mut self, k: PyObject) -> PyResult<()> {
        let kline = k.extract::<Kline>()?;
        self.inner.add_k(&kline);
        Ok(())
    }

    // 字段访问器
    #[getter]
    fn get_bi_bsp_list(&self, py: Python<'_>) -> PyResult<PyObject> {
        let list = PyList::empty(py);
        for bsp in self.inner.bi_bsp_list() {
            list.append(bsp.to_pyobject(py)?)?;
        }
        Ok(list.into())
    }

    #[getter]
    fn get_bi_zs_list(&self, py: Python<'_>) -> PyResult<PyObject> {
        let list = PyList::empty(py);
        for zs in self.inner.bi_zs_list() {
            list.append(zs.to_pyobject(py)?)?;
        }
        Ok(list.into())
    }

    #[getter]
    fn get_seg_list(&self, py: Python<'_>) -> PyResult<PyObject> {
        let list = PyList::empty(py);
        for seg in self.inner.seg_list() {
            list.append(seg.to_pyobject(py)?)?;
        }
        Ok(list.into())
    }

    #[getter]
    fn get_bi_list(&self, py: Python<'_>) -> PyResult<PyObject> {
        let list = PyList::empty(py);
        for bi in self.inner.bi_list() {
            list.append(bi.to_pyobject(py)?)?;
        }
        Ok(list.into())
    }

    #[getter]
    fn get_candle_list(&self, py: Python<'_>) -> PyResult<PyObject> {
        let list = PyList::empty(py);
        for candle in self.inner.candle_list() {
            list.append(candle.to_pyobject(py)?)?;
        }
        Ok(list.into())
    }

    #[getter]
    fn get_bar_list(&self, py: Python<'_>) -> PyResult<PyObject> {
        let list = PyList::empty(py);
        for bar in self.inner.bar_list() {
            list.append(bar.to_pyobject(py)?)?;
        }
        Ok(list.into())
    }
}

// 为Kline实现FromPyObject
impl<'source> FromPyObject<'source> for Kline {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        let dict = ob.downcast::<PyDict>()?;
        
        Ok(Kline {
            time: dict.get_item("time")?.extract()?,
            open: dict.get_item("open")?.extract()?,
            high: dict.get_item("high")?.extract()?,
            low: dict.get_item("low")?.extract()?,
            close: dict.get_item("close")?.extract()?,
            volume: dict.get_item("volume")?.extract()?,
        })
    }
}

// 在 lib.rs 中注册模块
#[pymodule]
fn czsc(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyAnalyzer>()?;
    Ok(())
}
