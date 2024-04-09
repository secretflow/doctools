use pyo3::prelude::*;

pub mod bundler;

#[pymodule]
pub fn _lib(_py: Python, m: &PyModule) -> PyResult<()> {
  m.add_class::<bundler::SphinxBundler>()?;
  Ok(())
}
