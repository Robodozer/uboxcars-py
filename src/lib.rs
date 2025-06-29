use pyo3::prelude::*;
use pyo3::{exceptions, wrap_pyfunction};
use serde_json::Value;
use std::collections::BTreeMap;

#[pyfunction]
fn parse_replay(py: Python, data: &[u8]) -> PyResult<PyObject> {
    let replay = boxcars::ParserBuilder::new(data)
        .must_parse_network_data()
        .on_error_check_crc()
        .parse()
        .map_err(to_py_error)?;
    let replay = serde_json::to_value(replay).map_err(to_py_error)?;
    let replay = convert_to_py(py, &replay);
    Ok(replay)
}

#[pymodule]
fn uboxcars_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse_replay, m)?)?;
    Ok(())
}

fn to_py_error<E: std::error::Error>(e: E) -> PyErr {
    PyErr::new::<exceptions::PyException, _>(format!("{e}"))
}

fn convert_to_py(py: Python, value: &Value) -> PyObject {
    match value {
        Value::Null => py.None(),
        Value::Bool(b) => b.into_pyobject(py).unwrap().to_owned().into(),
        Value::Number(n) => match n {
            n if n.is_u64() => n.as_u64().unwrap().into_pyobject(py).unwrap().to_owned().into(),
            n if n.is_i64() => n.as_i64().unwrap().into_pyobject(py).unwrap().to_owned().into(),
            n if n.is_f64() => n.as_f64().unwrap().into_pyobject(py).unwrap().to_owned().into(),
            _ => py.None(),
        },
        Value::String(s) => s.into_pyobject(py).unwrap().to_owned().into(),
        Value::Array(list) => {
            let list: Vec<PyObject> = list.iter().map(|e| convert_to_py(py, e)).collect();
            list.into_pyobject(py).unwrap().to_owned().into()
        }
        Value::Object(m) => {
            let mut map = BTreeMap::new();
            m.iter().for_each(|(k, v)| {
                map.insert(k, convert_to_py(py, v));
            });
            map.into_pyobject(py).unwrap().to_owned().into()
        }
    }
}