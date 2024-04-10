use pyo3::prelude::*;

pub mod bundler;

#[pymodule]
pub fn _lib(m: &Bound<'_, PyModule>) -> PyResult<()> {
  m.add_class::<bundler::Bundler>()?;
  m.add_class::<bundler::Doctree>()?;
  Ok(())
}
