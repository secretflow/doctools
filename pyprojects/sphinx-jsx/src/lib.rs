use katex;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use swc_core::ecma::ast::Str;

mod jsx;
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

#[derive(FromPyObject)]
#[pyo3(transparent)]
struct Intrinsic(String);

#[derive(FromPyObject)]
#[pyo3(transparent)]
struct Component(Option<String>);

#[derive(FromPyObject)]
enum JSXElement {
    Intrinsic(Intrinsic),
    Component(Component),
}

#[pyfunction]
fn test(e: JSXElement) -> PyResult<()> {
    match e {
        JSXElement::Intrinsic(Intrinsic(s)) => println!("A: {}", s),
        JSXElement::Component(Component(s)) => println!("B: {:?}", s),
    }
    Ok(())
}

#[pymodule]
fn _lib(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(math_to_html, m)?)?;
    m.add_function(wrap_pyfunction!(test, m)?)?;
    Ok(())
}
