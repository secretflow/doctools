use mdxjs;
use pyo3::prelude::*;

#[pyfunction]
fn compile_mdx(text: &str) -> PyResult<String> {
    match mdxjs::compile(
        text,
        &mdxjs::Options {
            provider_import_source: Some("@mdx-js/react".to_string()),
            jsx_runtime: Some(mdxjs::JsxRuntime::Automatic),
            ..Default::default()
        },
    ) {
        Ok(result) => Ok(result),
        Err(err) => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            err.to_string(),
        )),
    }
}

#[pymodule]
fn _lib(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(compile_mdx, m)?)?;
    Ok(())
}
