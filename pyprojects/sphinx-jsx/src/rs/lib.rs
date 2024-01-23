use pyo3::prelude::*;

pub mod bundler;
pub mod testing;

#[pymodule]
pub fn _lib(_py: Python, m: &PyModule) -> PyResult<()> {
  m.add_class::<bundler::SphinxBundler>()?;
  m.add_class::<testing::Testing>()?;
  Ok(())
}
