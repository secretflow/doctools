use std::path::PathBuf;

use deno_lite::DenoLite;
use pyo3::prelude::*;
use swc_core::common::{sync::Lrc, SourceMap};

mod diagnostics;
mod emit;
mod env;
mod linkcheck;
mod lint;
mod sourcemap;
mod symbols;
mod transform;
mod tree;

pub use self::tree::Doctree;
use self::{
  env::{Environment, SphinxFileLoader, SphinxOptions},
  sourcemap::SourceMapper,
};

#[pyclass(unsendable)]
pub struct Bundler {
  env: Lrc<Environment>,
  deno: DenoLite,
  sourcemap: Lrc<SourceMap>,
}

#[pymethods]
impl Bundler {
  #[new]
  fn new(options: SphinxOptions) -> PyResult<Self> {
    let env: Lrc<Environment> = Lrc::new(options.try_into()?);

    let deno = Default::default();

    let sourcemap = Lrc::new(SourceMap::with_file_loader(
      Box::new(SphinxFileLoader::from(env.clone())),
      Default::default(),
    ));

    Ok(Self {
      env,
      deno,
      sourcemap,
    })
  }

  fn sourcemap(&mut self) -> PyResult<SourceMapper> {
    Ok(SourceMapper::new(self))
  }

  fn build(&mut self, py: Python, mapper: &mut SourceMapper) -> PyResult<()> {
    let abort = Abort(py);

    mapper
      .detach()
      .into_transformer(self)?
      .transform(&abort)?
      .into_linter(self)
      .lint(&abort)?;

    Ok(())
  }
}

pub struct Abort<'py>(Python<'py>);

impl Abort<'_> {
  fn check(&self) -> PyResult<()> {
    self.0.check_signals()
  }
}
