#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::types::{PyDict, PyList, PyString, PyBool, PyFloat, PyLong};
#[cfg(feature = "python")]
use std::collections::HashMap;

#[cfg(feature = "python")]
use crate::{EmbeddedRuntime, EmbeddedValue, RuntimeConfig, RuntimeBuilder};

#[cfg(feature = "python")]
#[pyclass]
pub struct PyNagariRuntime {
    runtime: EmbeddedRuntime,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyNagariRuntime {
    #[new]
    #[pyo3(signature = (memory_limit=None, execution_timeout=None, allow_io=false, allow_network=false))]
    fn new(
        memory_limit: Option<usize>,
        execution_timeout: Option<u64>,
        allow_io: bool,
        allow_network: bool,
    ) -> PyResult<Self> {
        let mut builder = RuntimeBuilder::new()
            .allow_io(allow_io)
            .allow_network(allow_network);

        if let Some(limit) = memory_limit {
            builder = builder.memory_limit(limit);
        }

        if let Some(timeout) = execution_timeout {
            builder = builder.execution_timeout(timeout);
        }

        let runtime = builder.build()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;

        Ok(Self { runtime })
    }

    fn run(&mut self, script: &str) -> PyResult<PyObject> {
        let result = self.runtime.run_script(script)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;

        Python::with_gil(|py| embedded_value_to_py(py, result))
    }

    fn call(&mut self, function_name: &str, args: &PyList) -> PyResult<PyObject> {
        let embedded_args = Python::with_gil(|py| {
            let mut result = Vec::new();
            for arg in args.iter() {
                result.push(py_to_embedded_value(arg)?);
            }
            Ok::<Vec<EmbeddedValue>, PyErr>(result)
        })?;

        let result = self.runtime.call_function(function_name, embedded_args)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;

        Python::with_gil(|py| embedded_value_to_py(py, result))
    }

    fn load_module(&mut self, name: &str, code: &str) -> PyResult<()> {
        self.runtime.load_module(name, code)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))
    }

    fn set_global(&mut self, name: &str, value: PyObject) -> PyResult<()> {
        let embedded_value = Python::with_gil(|py| {
            py_to_embedded_value(value.as_ref(py))
        })?;

        self.runtime.set_global(name, embedded_value)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))
    }

    fn get_global(&self, name: &str) -> PyResult<Option<PyObject>> {
        match self.runtime.get_global(name) {
            Ok(Some(value)) => {
                Python::with_gil(|py| Ok(Some(embedded_value_to_py(py, value)?)))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e)),
        }
    }

    fn reset(&mut self) -> PyResult<()> {
        self.runtime.reset()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))
    }

    fn register_function(&mut self, name: &str, func: PyObject) -> PyResult<()> {
        // Store the Python function and create a wrapper
        let func_name = name.to_string();

        self.runtime.register_host_function(&func_name, move |args| {
            Python::with_gil(|py| {
                let py_args = PyList::empty(py);

                for arg in args {
                    let py_value = embedded_value_to_py(py, arg)?;
                    py_args.append(py_value)?;
                }

                let result = func.call1(py, (py_args,))?;
                py_to_embedded_value(result.as_ref(py))
            }).unwrap_or(EmbeddedValue::None)
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))
    }
}

#[cfg(feature = "python")]
fn py_to_embedded_value(obj: &PyAny) -> PyResult<EmbeddedValue> {
    if obj.is_none() {
        Ok(EmbeddedValue::None)
    } else if let Ok(b) = obj.extract::<bool>() {
        Ok(EmbeddedValue::Bool(b))
    } else if let Ok(i) = obj.extract::<i64>() {
        Ok(EmbeddedValue::Int(i))
    } else if let Ok(f) = obj.extract::<f64>() {
        Ok(EmbeddedValue::Float(f))
    } else if let Ok(s) = obj.extract::<String>() {
        Ok(EmbeddedValue::String(s))
    } else if let Ok(list) = obj.downcast::<PyList>() {
        let mut result = Vec::new();
        for item in list.iter() {
            result.push(py_to_embedded_value(item)?);
        }
        Ok(EmbeddedValue::Array(result))
    } else if let Ok(dict) = obj.downcast::<PyDict>() {
        let mut result = HashMap::new();
        for (key, value) in dict.iter() {
            let key_str = key.extract::<String>()?;
            let embedded_value = py_to_embedded_value(value)?;
            result.insert(key_str, embedded_value);
        }
        Ok(EmbeddedValue::Object(result))
    } else {
        Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
            "Unsupported Python type"
        ))
    }
}

#[cfg(feature = "python")]
fn embedded_value_to_py(py: Python, value: EmbeddedValue) -> PyResult<PyObject> {
    match value {
        EmbeddedValue::None => Ok(py.None()),
        EmbeddedValue::Bool(b) => Ok(b.into_py(py)),
        EmbeddedValue::Int(i) => Ok(i.into_py(py)),
        EmbeddedValue::Float(f) => Ok(f.into_py(py)),
        EmbeddedValue::String(s) => Ok(s.into_py(py)),
        EmbeddedValue::Array(arr) => {
            let py_list = PyList::empty(py);
            for item in arr {
                py_list.append(embedded_value_to_py(py, item)?)?;
            }
            Ok(py_list.into_py(py))
        }
        EmbeddedValue::Object(obj) => {
            let py_dict = PyDict::new(py);
            for (key, value) in obj {
                py_dict.set_item(key, embedded_value_to_py(py, value)?)?;
            }
            Ok(py_dict.into_py(py))
        }
    }
}

#[cfg(feature = "python")]
#[pymodule]
fn nagari(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyNagariRuntime>()?;

    // Add convenience functions
    #[pyfn(m)]
    fn create_runtime() -> PyResult<PyNagariRuntime> {
        PyNagariRuntime::new(None, None, false, false)
    }

    #[pyfn(m)]
    fn create_sandbox_runtime() -> PyResult<PyNagariRuntime> {
        PyNagariRuntime::new(Some(32 * 1024 * 1024), Some(1000), false, false)
    }

    Ok(())
}
