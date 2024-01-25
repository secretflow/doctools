use anyhow::Context;
use pyo3::{
  exceptions::{PyRuntimeError, PyValueError},
  prelude::*,
};
use pyo3_utils::raise;
use swc_core::common::{sync::Lrc, FileName, SourceFile};

use swc_ecma_utils::{
  ast::json_to_expr,
  jsx::{
    builder::{DocumentBuilder, JSXDocument},
    factory::{JSXRuntime, JSXTagName},
  },
};

use super::range::SourcePosition;

#[pyclass]
#[derive(Debug)]
pub struct SphinxDocument {
  source: Lrc<SourceFile>,
  builder: Option<DocumentBuilder>,
}

impl SphinxDocument {
  pub fn new(runtime: JSXRuntime, source_file: Lrc<SourceFile>) -> Self {
    Self {
      source: source_file,
      builder: Some(DocumentBuilder::new(runtime)),
    }
  }

  pub fn seal(&mut self) -> PyResult<JSXDocument> {
    let builder = match self.builder.take() {
      Some(builder) => builder,
      None => {
        return Err(raise::<PyRuntimeError, _>(anyhow::anyhow!(
          "document is already sealed!"
        )))
      }
    };
    Ok(builder.declare())
  }

  pub fn get_filename(&self) -> &FileName {
    &self.source.name
  }

  fn get_builder(&mut self) -> PyResult<&mut DocumentBuilder> {
    match self.builder.as_mut() {
      Some(builder) => Ok(builder),
      None => Err(raise::<PyRuntimeError, _>(anyhow::anyhow!(
        "cannot mutate a sealed document!"
      ))),
    }
  }
}

#[pymethods]
impl SphinxDocument {
  #[pyo3(signature = (name, props=None, *, position=None))]
  pub fn element(
    &mut self,
    name: &str,
    props: Option<&str>,
    position: Option<SourcePosition>,
  ) -> PyResult<()> {
    let element = JSXTagName::Ident(name.into());

    let props = match props {
      None => None,
      Some(props) => Some(json_to_expr(
        serde_json::from_str(props)
          .context("failed to parse props as JSON")
          .map_err(raise::<PyValueError, _>)?,
      )),
    };

    let span = match position {
      Some(position) => match position.reify(&self.source) {
        Some(span) => Some(span),
        None => {
          return Err(raise::<PyValueError, _>(anyhow::anyhow!(
            "Invalid source position {} for file {}",
            position,
            self.source.name
          )))
        }
      },
      None => None,
    };

    let builder = self.get_builder()?;

    builder.element(&element, props, span);

    Ok(())
  }

  pub fn text(&mut self, text: &str) -> PyResult<()> {
    let builder = self.get_builder()?;
    builder.value(text.into());
    Ok(())
  }

  pub fn enter(&mut self) -> PyResult<()> {
    let builder = self.get_builder()?;
    builder.enter(&["children"]);
    Ok(())
  }

  pub fn exit(&mut self) -> PyResult<()> {
    let builder = self.get_builder()?;
    builder.exit();
    Ok(())
  }
}
