use anyhow::Context;
use pyo3::{exceptions::PyValueError, prelude::*};
use serde_json;
use swc_core::ecma::ast::Expr;

use pyo3_utils::raise;
use swc_ecma_utils::testing::print_one;

#[pyclass]
#[pyo3(name = "testing")]
pub(crate) struct Testing;

#[pymethods]
impl Testing {
  #[staticmethod]
  pub fn ast_string_to_ecma(ast: &str) -> PyResult<String> {
    let expr: Expr = serde_json::from_str(ast)
      .context("cannot deserialize JSON string as SWC AST")
      .map_err(raise::<PyValueError, _>)?;

    let code = print_one(&Box::new(expr), None, Default::default())
      .context("failed to generate ECMAScript")
      .map_err(raise::<PyValueError, _>)?;

    Ok(code)
  }
}
