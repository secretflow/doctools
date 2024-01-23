use pyo3::FromPyObject;

#[derive(Debug, Clone, FromPyObject)]
pub struct Range {
  start_line: usize,
  start_column: usize,
  end_line: usize,
  end_column: usize,
}
