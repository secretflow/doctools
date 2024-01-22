use pyo3::prelude::*;

mod testing;

#[pymodule]
fn _lib(_py: Python, m: &PyModule) -> PyResult<()> {
  m.add_class::<testing::Testing>()?;
  Ok(())
}
