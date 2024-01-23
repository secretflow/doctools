use std::fmt;

use pyo3::{PyErr, PyTypeInfo};

pub fn raise<T, E>(err: E) -> PyErr
where
  T: PyTypeInfo,
  E: fmt::Debug,
{
  PyErr::new::<T, _>(format!("{:?}", err))
}
