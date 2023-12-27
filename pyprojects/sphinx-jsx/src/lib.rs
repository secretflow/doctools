use katex;
use mdxjs;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyBool;

mod constants;

#[pyfunction]
#[pyo3(signature = (source, *, development=None, mdx_provider=None, jsx_runtime=None))]
fn mdx_to_js(
    source: &str,
    development: Option<&PyBool>,
    mdx_provider: Option<&str>,
    jsx_runtime: Option<&str>,
) -> PyResult<String> {
    let options = mdxjs::Options {
        parse: mdxjs::MdxParseOptions {
            constructs: constants::JSX_ONLY,
            ..Default::default()
        },
        jsx_runtime: Some(mdxjs::JsxRuntime::Automatic),
        jsx_import_source: match jsx_runtime {
            Some(package) => Some(package.into()),
            None => Some("react".into()), // react/jsx-runtime
        },
        provider_import_source: match mdx_provider {
            Some(package) => Some(package.into()),
            None => None,
        },
        development: match development {
            Some(development) => development.is_true(),
            None => false,
        },
        ..Default::default()
    };

    let result = mdxjs::compile(source, &options);

    match result {
        Err(err) => Err(PyErr::new::<PyValueError, _>(err)),
        Ok(result) => Ok(result),
    }
}

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
    m.add_function(wrap_pyfunction!(mdx_to_js, m)?)?;
    m.add_function(wrap_pyfunction!(math_to_html, m)?)?;
    Ok(())
}
