use std::path::{Path, PathBuf};

use deno_lite::DenoLite;
use fuzzy_sourcemap::{FuzzySourceMap, PathResolver};
use indexmap::IndexMap;
use pyo3::prelude::*;
use swc_core::common::{sync::Lrc, FileLoader, FileName, SourceMap};
use swc_ecma_utils2::jsx::{DocumentBuilder, JSXRuntime};

struct Symbols;

impl JSXRuntime for Symbols {
  const JSX: &'static str = "_jsx";
  const JSXS: &'static str = "_jsxs";
  const FRAGMENT: &'static str = "_Fragment";
}

#[derive(FromPyObject, Clone)]
struct SphinxEnv {
  srcdir: PathBuf,
  outdir: PathBuf,
  confdir: PathBuf,
}

impl FileLoader for SphinxEnv {
  fn file_exists(&self, path: &Path) -> bool {
    todo!()
  }

  fn abs_path(&self, path: &Path) -> Option<PathBuf> {
    todo!()
  }

  fn read_file(&self, path: &Path) -> std::io::Result<String> {
    todo!()
  }
}

impl PathResolver for SphinxEnv {
  fn resolve(&self, path: Option<&str>, base: Option<PathBuf>) -> Option<PathBuf> {
    todo!()
  }
}

#[derive(Default)]
enum BundlerPhase {
  #[default]
  Init,
  SourceMap {
    pages: IndexMap<FileName, DocumentBuilder<Symbols>>,
    sourcemap: FuzzySourceMap,
  },
}

#[pyclass(unsendable)]
pub struct SphinxBundler {
  deno: DenoLite,
  options: SphinxEnv,
  sources: Lrc<SourceMap>,
  phase: BundlerPhase,
}

#[pymethods]
impl SphinxBundler {
  #[new]
  fn new(options: SphinxEnv) -> Self {
    Self {
      deno: Default::default(),
      options,
      sources: Default::default(),
      phase: Default::default(),
    }
  }

  fn init(&mut self) {
    match self.phase {
      BundlerPhase::Init => {
        let sourcemap = FuzzySourceMap::new(self.sources.clone(), Default::default());
        self.phase = BundlerPhase::SourceMap {
          pages: Default::default(),
          sourcemap,
        };
      }
      _ => unreachable!("Invalid phase"),
    }
  }

  #[pyo3(signature = (component, attrs, *, file_name = None, line_number = None, raw_source = None))]
  fn chunk(
    &mut self,
    component: &str,
    attrs: &str,
    file_name: Option<&str>,
    line_number: Option<usize>,
    raw_source: Option<&str>,
  ) {
    let BundlerPhase::SourceMap { pages, sourcemap } = &mut self.phase else {
      unreachable!("Invalid phase");
    };
    dbg!(file_name);
  }
}
