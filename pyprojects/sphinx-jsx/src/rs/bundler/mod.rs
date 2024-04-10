use std::path::PathBuf;

use deno_lite::{DenoLite, ESModule};
use indexmap::IndexMap;
use pyo3::prelude::*;
use swc_core::common::{sync::Lrc, FileName, SourceMap};
use swc_ecma_transform_sphinx::init_esm;
use swc_ecma_utils2::jsx::JSXDocument;

mod env;
mod symbols;
mod tree;

use self::{
  env::{SphinxEnv, SphinxFileLoader, SphinxOptions},
  symbols::Symbols,
};

pub use self::tree::Doctree;

#[derive(Default)]
enum BundlerPhase {
  #[default]
  Init,
  SourceMap {
    esm: ESModule,
    trees: IndexMap<FileName, JSXDocument>,
  },
}

#[pyclass(unsendable)]
pub struct Bundler {
  env: Lrc<SphinxEnv>,
  deno: DenoLite,
  files: Lrc<SourceMap>,
  phase: BundlerPhase,
}

macro_rules! assert_phase {
  ($match:pat, $phase:expr) => {
    let $match = $phase else {
      unreachable!("Invalid phase");
    };
  };
}

#[pymethods]
impl Bundler {
  #[new]
  fn new(options: SphinxOptions) -> PyResult<Self> {
    let env: Lrc<SphinxEnv> = Lrc::new(options.try_into()?);

    let deno = Default::default();

    let files = Lrc::new(SourceMap::with_file_loader(
      Box::new(SphinxFileLoader::from(env.clone())),
      Default::default(),
    ));

    let phase = Default::default();

    Ok(Self {
      env,
      deno,
      files,
      phase,
    })
  }

  fn init(&mut self) -> PyResult<()> {
    assert_phase!(BundlerPhase::Init, &self.phase);
    let esm = init_esm(self.deno.clone())?;
    let trees = Default::default();
    self.phase = BundlerPhase::SourceMap { esm, trees };
    Ok(())
  }

  fn open<'py>(&mut self, py: Python<'py>, path: PathBuf) -> PyResult<Bound<'py, Doctree>> {
    assert_phase!(BundlerPhase::SourceMap { trees, .. }, &mut self.phase);

    let file = self
      .files
      .get_source_file(&FileName::Real(path.clone()))
      .ok_or(())
      .or_else(|_| self.files.load_file(&path))?;

    if trees.contains_key(&file.name) {
      return Err(anyhow::anyhow!("doctree with name {} already exists", file.name.clone()).into());
    }

    Bound::new(py, Doctree::from((&*self, file.name.clone())))
  }

  fn seal<'py>(&mut self, _py: Python<'py>, tree: Bound<'py, Doctree>) -> PyResult<()> {
    assert_phase!(BundlerPhase::SourceMap { trees, .. }, &mut self.phase);

    let (file, tree) = tree.borrow_mut().close()?;

    if trees.contains_key(&file) {
      return Err(anyhow::anyhow!("doctree with name {} already exists", file).into());
    }

    trees.insert(file, tree);

    Ok(())
  }

  fn emit(&mut self) -> PyResult<()> {
    assert_phase!(BundlerPhase::SourceMap { trees, esm }, &mut self.phase);

    for (file, tree) in trees.drain(..) {
      self.env.emit_page(
        &file,
        self.files.clone(),
        tree.to_fragment::<Symbols>().into(),
        esm,
      )?;
    }

    Ok(())
  }
}
