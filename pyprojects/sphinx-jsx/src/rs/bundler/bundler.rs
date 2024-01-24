use std::{
  collections::HashSet,
  fs,
  io::Write as _,
  option,
  path::{Path, PathBuf},
};

use anyhow::Context;
use base64::prelude::{Engine, BASE64_STANDARD};
use itertools;
use pyo3::{
  exceptions::{PyOSError, PyRuntimeError, PyUnicodeDecodeError, PyValueError},
  prelude::*,
};
use relative_path::RelativePathBuf;
use serde::{Deserialize, Serialize};
use swc_core::{
  common::{
    source_map::{DefaultSourceMapGenConfig, SourceMapGenConfig},
    sync::Lrc,
    FileName, SourceMap,
  },
  ecma::codegen::{text_writer::JsWriter, Emitter, Node as _},
};

use pyo3_utils::raise;
use swc_ecma_lints::undefined_bindings::LintUndefinedBindings;
use swc_ecma_utils::{
  jsx::{builder::JSXDocument, factory::JSXFactory},
  testing::print_one,
};

use super::{document::SphinxDocument, symbols::WellKnownSymbols};

#[pyclass]
pub struct SphinxBundler {
  symbols: WellKnownSymbols,
  sources: Lrc<SourceMap>,
  pages: Vec<(FileName, JSXDocument)>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromPyObject)]
pub struct BuildOptions {
  srcdir: PathBuf,
  outdir: PathBuf,
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
      .with_jsx(&self.symbols.jsx)
      .with_jsxs(&self.symbols.jsxs)
      .with_fragment(&self.symbols.fragment);

    let document = SphinxDocument::new(factory, source_file.clone());

    Ok(document)
  }

  pub fn seal_document(&mut self, document: &PyCell<SphinxDocument>) -> PyResult<()> {
    let filename = document.borrow().get_filename().clone();
    let document = document.borrow_mut().seal()?;
    self.pages.push((filename, document));
    Ok(())
  }

  pub fn build(&mut self, options: BuildOptions) -> PyResult<()> {
    // sanity check

    if options.srcdir == options.outdir {
      return Err(raise::<PyValueError, _>(anyhow::anyhow!(
        "source and output directories must be different"
      )));
    }

    if options.outdir.is_dir() {
      // TODO: refuse to remove dirs that are not empty and not ignored
      fs::remove_dir_all(&options.outdir)?;
    } else {
      match options.outdir.try_exists() {
        Ok(true) => {
          return Err(raise::<PyValueError, _>(anyhow::anyhow!(
            "output path {} already exists and is not a directory",
            options.outdir.display()
          )));
        }
        Ok(false) => {
          fs::create_dir_all(&options.outdir)?;
        }
        Err(err) => {
          return Err(raise::<PyOSError, _>(err));
        }
      }
    }

    // end sanity check

    let src_dir = abspath_to_relpath(&options.srcdir)
      .context("failed to parse source path")
      .map_err(raise::<PyOSError, _>)?;

    let out_dir = abspath_to_relpath(&options.outdir)
      .context("failed to parse output path")
      .map_err(raise::<PyOSError, _>)?;

    let pages: Vec<(PathBuf, String)> = vec![];

    let undefined_bindings =
      LintUndefinedBindings::new(vec![include_str!("../../js/theme.d.ts").to_string()])
        .map_err(raise::<PyRuntimeError, _>)?;

    let mut unsupported_components = HashSet::new();

    for (docname, document) in self.pages.drain(..) {
      unsupported_components.extend(undefined_bindings.lint(&document.body));

      let mut code_buffer = vec![];
      let mut source_mapping = vec![];
      let mut source_map_buffer = vec![];

      let writer = JsWriter::new(
        self.sources.clone(),
        "\n",
        &mut code_buffer,
        Some(&mut source_mapping),
      );

      let mut emitter = Emitter {
        cfg: Default::default(),
        cm: self.sources.clone(),
        comments: None,
        wr: Box::new(writer),
      };

      document
        .body
        .emit_with(&mut emitter)
        .context("failed to generate ECMAScript")
        .map_err(raise::<PyRuntimeError, _>)?;

      self
        .sources
        .build_source_map_with_config(&source_mapping, None, DefaultSourceMapGenConfig {})
        .to_writer(&mut source_map_buffer)
        .context("failed to generate source map")
        .map_err(raise::<PyRuntimeError, _>)?;

      let mut code = String::from_utf8(code_buffer)
        .context("failed to decode result as UTF-8")
        .map_err(raise::<PyUnicodeDecodeError, _>)?;

      code.push_str("\n//# sourceMappingURL=data:application/json;base64,");
      BASE64_STANDARD.encode_string(&source_map_buffer, &mut code);

      let source_file = self
        .sources
        .get_source_file(&docname)
        .expect("source file unexpectedly not found");

      let docname = file_name_to_relpath(&docname)
        .context("failed to parse output path")
        .map_err(raise::<PyOSError, _>)?;

      let docname = src_dir.relative(&docname);

      let out_path = out_dir.join(docname.with_extension("js")).to_path("/");

      let parent_dir = out_path
        .parent()
        .context("failed to get parent directory")
        .map_err(raise::<PyOSError, _>)?;

      fs::create_dir_all(parent_dir)
        .context("failed to create parent directory")
        .map_err(raise::<PyOSError, _>)?;

      {
        let mut out_file = fs::File::create(&out_path)
          .context("failed to create output file")
          .map_err(raise::<PyOSError, _>)?;

        out_file.write_all(code.as_bytes())?;
      }
    }

    println!(
      "unsupported components: {}",
      itertools::join(itertools::sorted(unsupported_components), ", ")
    );

    Ok(())
  }
}

fn abspath_to_relpath(abspath: &PathBuf) -> anyhow::Result<RelativePathBuf> {
  let path = abspath
    .strip_prefix("/")
    .context("failed to strip prefix")?;
  Ok(RelativePathBuf::from_path(path).context("failed to parse as relative path")?)
}

fn file_name_to_relpath(filename: &FileName) -> anyhow::Result<RelativePathBuf> {
  match filename {
    FileName::Real(path_buf) => abspath_to_relpath(path_buf),
    _ => unreachable!("unexpected filename type {:?}", filename),
  }
}
