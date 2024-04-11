use pyo3::prelude::*;

pub mod bundler;

#[pymodule]
pub fn _lib(m: &Bound<'_, PyModule>) -> PyResult<()> {
  env_logger::builder()
    .parse_env(env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"))
    .filter_module("tracing::span", log::LevelFilter::Off)
    .filter_module("swc_ecma_codegen", log::LevelFilter::Off)
    .format_indent(None)
    .init();

  m.add_class::<bundler::Bundler>()?;
  m.add_class::<bundler::Doctree>()?;

  Ok(())
}
