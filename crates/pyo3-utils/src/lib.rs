use std::fmt::Debug;

use pyo3::{PyErr, PyTypeInfo};

pub fn raise<T, E>(err: E) -> PyErr
where
  T: PyTypeInfo,
  E: Debug,
{
  PyErr::new::<T, _>(format!("{:?}", err))
}
