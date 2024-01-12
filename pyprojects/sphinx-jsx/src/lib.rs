use katex;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;

mod math;

#[pyfunction]
#[pyo3(signature = (source, *, mode="inline"))]
fn math_to_html(source: &str, mode: &str) -> PyResult<String> {
    let options = katex::Opts::builder()
        .throw_on_error(true)
        .display_mode(mode == "block")
        .output_type(katex::OutputType::Html)
        .build()
        .map_err(|err| PyErr::new::<PyRuntimeError, _>(err.to_string()))?;

    let result = katex::render_with_opts(source, options);

    match result {
        Err(err) => Err(PyErr::new::<PyValueError, _>(err.to_string())),
        Ok(result) => Ok(result),
    }
}

#[pymodule]
fn _lib(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(math_to_html, m)?)?;
    Ok(())
}
