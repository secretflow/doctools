use std::path::PathBuf;

use indexmap::IndexMap;
use pyo3::{pyclass, pymethods};
use swc_core::{
  common::{sync::Lrc, FileName, SourceMap},
  ecma::ast::Expr,
};
use swc_ecma_utils2::jsx::JSXDocument;

use super::{env::Environment, symbols::Symbols, transform::Transformer, Bundler, Doctree};

#[pyclass]
pub struct SourceMapper {
  sourcemap: Lrc<SourceMap>,
  env: Lrc<Environment>,
  trees: IndexMap<FileName, JSXDocument>,
}

impl SourceMapper {
  pub fn new(bundler: &Bundler) -> Self {
    Self {
      sourcemap: bundler.sourcemap.clone(),
      env: bundler.env.clone(),
      trees: Default::default(),
    }
  }

  pub fn detach(&mut self) -> Self {
    let trees = std::mem::take(&mut self.trees);
    Self {
      sourcemap: self.sourcemap.clone(),
      env: self.env.clone(),
      trees,
    }
  }

  pub fn into_transformer(self, bundler: &Bundler) -> anyhow::Result<Transformer> {
    let trees = self
      .trees
      .into_iter()
      .map(|(file, tree)| (file, Expr::from(tree.to_fragment::<Symbols>())))
      .collect::<Vec<_>>();
    Transformer::new(bundler, trees)
  }
}

#[pymethods]
impl SourceMapper {
  pub fn open(&self, path: PathBuf) -> anyhow::Result<Doctree> {
    let file = self
      .sourcemap
      .get_source_file(&FileName::Real(path.clone()))
      .ok_or(())
      .or_else(|_| self.sourcemap.load_file(&path))?;

    if self.trees.contains_key(&file.name) {
      return Err(anyhow::anyhow!(
        "doctree with name {} already exists",
        file.name.clone()
      ));
    }

    Ok(Doctree::from((
      self.sourcemap.clone(),
      self.env.clone(),
      file.name.clone(),
    )))
  }

  pub fn seal(&mut self, tree: &mut Doctree) -> anyhow::Result<()> {
    let (file, tree) = tree.close()?;

    if self.trees.contains_key(&file) {
      return Err(anyhow::anyhow!("doctree with name {} already exists", file));
    }

    self.trees.insert(file, tree);

    Ok(())
  }
}
