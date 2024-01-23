use std::{collections::HashMap, path::PathBuf};

use pyo3::{exceptions::PyValueError, prelude::*};
use pyo3_utils::raise;
use swc_core::common::{sync::Lrc, FileName, SourceFile, SourceMap};

use swc_utils::jsx::{
  builder::{DocumentBuilder, JSXDocument},
  factory::JSXFactory,
};

mod range;

use self::range::Range;

#[pyclass]
#[derive(Debug)]
pub struct SphinxDocument {
  source: Option<Lrc<SourceFile>>,
  builder: DocumentBuilder,
}

impl SphinxDocument {
  fn real(jsx: JSXFactory, source_file: Lrc<SourceFile>) -> Self {
    Self {
      source: Some(source_file),
      builder: DocumentBuilder::new(jsx),
    }
  }

  fn anon(jsx: JSXFactory) -> Self {
    Self {
      source: None,
      builder: DocumentBuilder::new(jsx),
    }
  }
}

#[pymethods]
impl SphinxDocument {
  pub fn element(&mut self, range: Option<Range>) -> PyResult<()> {
    todo!()
  }

  pub fn html(&mut self, transclusion: Option<HashMap<String, String>>) -> PyResult<()> {
    todo!()
  }
}

#[pyclass]
pub struct SphinxBundler {
  factory: JSXFactory,
  sources: Lrc<SourceMap>,
}

#[pymethods]
impl SphinxBundler {
  #[new]
  pub fn __new__() -> Self {
    Self {
      factory: JSXFactory::new()
        .jsx("_jsx")
        .jsxs("_jsxs")
        .fragment("Fragment"),
      sources: Default::default(),
    }
  }

  pub fn add_source(&mut self, path: PathBuf, source: String) -> PyResult<SphinxDocument> {
    let filename = FileName::from(path);

    if self.sources.get_source_file(&filename).is_some() {
      return Err(raise::<PyValueError, _>(anyhow::anyhow!(
        "file at path {} has already been added",
        filename
      )));
    };

    let source_file = self.sources.new_source_file(filename.clone(), source);
    let document = SphinxDocument::real(self.factory.clone(), source_file.clone());

    Ok(document)
  }

  pub fn create_ad_hoc_document(&self) -> SphinxDocument {
    SphinxDocument::anon(self.factory.clone())
  }
}
