use pyo3::prelude::*;

#[pymodule]
fn _lib(_py: Python, m: &PyModule) -> PyResult<()> {
    Ok(())
}
