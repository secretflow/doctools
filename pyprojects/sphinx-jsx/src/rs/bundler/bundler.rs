use std::path::PathBuf;

use pyo3::{exceptions::PyValueError, prelude::*};
use pyo3_utils::raise;
use swc_core::common::{sync::Lrc, FileName, SourceMap};

use swc_utils::jsx::{builder::JSXDocument, factory::JSXFactory};

use super::{document::SphinxDocument, symbols::WellKnownSymbols};

#[pyclass]
pub struct SphinxBundler {
  symbols: WellKnownSymbols,
  sources: Lrc<SourceMap>,
  pages: Vec<(FileName, JSXDocument)>,
}

#[pymethods]
impl SphinxBundler {
  #[new]
  #[pyo3(signature = (symbols))]
  pub fn __new__(symbols: WellKnownSymbols) -> Self {
    Self {
      symbols,
      sources: Default::default(),
      pages: Default::default(),
    }
  }

  pub fn make_document(&mut self, path: PathBuf, source: String) -> PyResult<SphinxDocument> {
    let filename = FileName::from(path);

    if self.sources.get_source_file(&filename).is_some() {
      return Err(raise::<PyValueError, _>(anyhow::anyhow!(
        "file at path {} has already been added",
        filename
      )));
    };

    let source_file = self.sources.new_source_file(filename.clone(), source);

    let factory = JSXFactory::new()
      .jsx(&self.symbols.jsx)
      .jsxs(&self.symbols.jsxs)
      .fragment(&self.symbols.fragment);

    let document = SphinxDocument::new(factory, source_file.clone());

    Ok(document)
  }

  pub fn seal_document(&mut self, document: &PyCell<SphinxDocument>) -> PyResult<()> {
    let filename = document.borrow().get_filename().clone();
    let document = document.borrow_mut().seal()?;
    self.pages.push((filename, document));
    Ok(())
  }
}
