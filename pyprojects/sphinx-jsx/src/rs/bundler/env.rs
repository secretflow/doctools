use std::{
  fs,
  io::Write,
  path::{Path, PathBuf},
};

use anyhow::Context;
use deno_lite::ESModule;
use fuzzy_sourcemap::PathResolver;
use pyo3::FromPyObject;
use relative_path::RelativePathBuf;
use swc_core::{
  common::{source_map::SourceMapGenConfig, sync::Lrc, FileLoader, FileName, SourceMap, Spanned},
  ecma::{
    ast::{Expr, Lit, Str},
    codegen::{text_writer::JsWriter, Emitter, Node},
    visit::{noop_visit_type, Visit, VisitMutWith, VisitWith},
  },
};
use swc_ecma_transform_sphinx::{render_code, render_math, render_raw, render_typography};
use swc_ecma_utils2::{
  ecma::fixes::remove_invalid,
  jsx::{
    fixes::{fix_jsx_factories, fold_fragments},
    JSXElement,
  },
};
use url::Url;

use crate::bundler::symbols::Symbols;

#[derive(FromPyObject, Clone)]
pub struct SphinxOptions {
  srcdir: String,
  outdir: String,
}

pub struct SphinxEnv {
  cwd: Url,
  src_dir: Url,
  out_dir: Url,
}

impl TryFrom<SphinxOptions> for SphinxEnv {
  type Error = anyhow::Error;

  fn try_from(options: SphinxOptions) -> anyhow::Result<Self> {
    let cwd = std::env::current_dir().context("failed to get current working directory")?;

    let SphinxOptions { srcdir, outdir } = options;

    let src_dir = cwd.join(srcdir).canonicalize()?;

    let out_dir = src_dir.join(outdir);

    let cwd = Url::from_directory_path(cwd).expect("should be a directory");
    let src_dir = Url::from_directory_path(src_dir).expect("should be a directory");
    let out_dir = Url::from_directory_path(out_dir).expect("should be a directory");

    Ok(Self {
      cwd,
      src_dir,
      out_dir,
    })
  }
}

impl SphinxEnv {
  pub fn emit_page(
    &self,
    src_name: &FileName,
    files: Lrc<SourceMap>,
    mut module: Expr,
    esm: &ESModule,
  ) -> anyhow::Result<()> {
    let FileName::Real(src_path) = src_name else {
      unreachable!()
    };

    {
      module.visit_mut_with(&mut render_code::<Symbols>(files.clone(), esm));
      module.visit_mut_with(&mut render_math::<Symbols>(files.clone(), esm));
      module.visit_mut_with(&mut render_raw::<Symbols>(files.clone()));
      module.visit_mut_with(&mut render_typography::<Symbols>());
      module.visit_mut_with(&mut fold_fragments::<Symbols>());
      module.visit_mut_with(&mut fix_jsx_factories::<Symbols>());
      module.visit_mut_with(&mut remove_invalid())
    };

    let module = module;

    struct SourceLocation(Lrc<SourceMap>);

    impl SourceLocation {
      fn maybe_emit(&self, expr: &Expr) -> Option<()> {
        let span = expr.span();
        if span.is_dummy() {
          return None;
        }
        let src = {
          let f = self.0.span_to_filename(span);
          self.0.get_source_file(&f)
        }?;
        let name = match expr {
          Expr::Call(call) => call.as_jsx_type::<Symbols>().map(|f| format!("{:?}", f)),
          Expr::Lit(Lit::Str(Str { value, .. })) => Some(value.to_string()),
          _ => None,
        }?;
        let report = miette::miette!(
          severity = miette::Severity::Advice,
          labels = vec![miette::LabeledSpan::new(
            Some(name),
            (span.lo().0 - src.start_pos.0) as usize,
            (span.hi().0 - span.lo().0) as usize,
          )],
          "source map"
        )
        .with_source_code(miette::NamedSource::new(
          src.name.to_string(),
          src.src.clone(),
        ));
        println!("{:?}", report);
        Some(())
      }
    }

    impl Visit for SourceLocation {
      noop_visit_type!();

      fn visit_expr(&mut self, expr: &Expr) {
        self.maybe_emit(expr);
        expr.visit_children_with(self);
      }
    }

    {
      module.visit_with(&mut SourceLocation(files.clone()));
    }

    let src_url = Url::from_file_path(src_path).expect("should be a file URL");

    let rel_path = self.src_dir.make_relative(&src_url).ok_or_else(|| {
      anyhow::anyhow!(
        "unexpected source file {0} outside or source directory",
        src_path.to_string_lossy().to_string()
      )
    })?;

    let rel_path = RelativePathBuf::from(rel_path);

    let out_url = self.out_dir.join(rel_path.as_str())?;

    assert!(self.out_dir.make_relative(&out_url).is_some());

    let out_js = out_url
      .to_file_path()
      .expect("should be a file")
      .with_extension("js");
    let out_js_map = out_js.with_extension("js.map");
    let out_js_map_raw = out_js.with_extension("js.map.txt");
    let out_yaml = out_js.with_extension("yaml");

    std::fs::create_dir_all(out_js.parent().expect("should have a parent"))?;

    println!("Emitting {}", out_js.to_string_lossy());

    let mut out_js_file = std::fs::File::create(out_js)?;
    let mut out_js_map_file = std::fs::File::create(out_js_map)?;
    let mut out_js_map_raw_file = std::fs::File::create(out_js_map_raw)?;
    let out_yaml_file = std::fs::File::create(out_yaml)?;

    let mut source_map_raw = vec![];

    {
      let mut emitter = Emitter {
        cfg: Default::default(),
        cm: files.clone(),
        comments: None,
        wr: Box::new(JsWriter::new(
          files.clone(),
          "\n",
          &mut out_js_file,
          Some(&mut source_map_raw),
        )),
      };
      module.emit_with(&mut emitter)?;
    }

    {
      out_js_map_raw_file.write_all(format!("{:#?}", source_map_raw).as_bytes())?;
    }

    {
      struct Config;

      impl SourceMapGenConfig for Config {
        fn file_name_to_source(&self, f: &FileName) -> String {
          format!("{}", f)
        }
        fn inline_sources_content(&self, _: &FileName) -> bool {
          true
        }
      }

      let source_map = files.build_source_map_with_config(&source_map_raw, None, Config);
      source_map.to_writer(&mut out_js_map_file)?;
    }

    {
      serde_yaml::to_writer(out_yaml_file, &module)?;
    }

    Ok(())
  }
}

impl SphinxEnv {
  fn try_files(&self, path: &Path) -> anyhow::Result<Url> {
    try_files(path, [&self.src_dir, &self.cwd].iter().copied())
  }
}

#[derive(Clone)]
pub struct SphinxFileLoader {
  env: Lrc<SphinxEnv>,
}

impl From<Lrc<SphinxEnv>> for SphinxFileLoader {
  fn from(env: Lrc<SphinxEnv>) -> Self {
    Self { env }
  }
}

impl FileLoader for SphinxFileLoader {
  fn file_exists(&self, path: &Path) -> bool {
    self.env.try_files(path).is_ok()
  }

  fn abs_path(&self, path: &Path) -> Option<PathBuf> {
    self
      .env
      .try_files(path)
      .ok()
      .map(|url| url.to_file_path().expect("should be a file URL"))
  }

  fn read_file(&self, path: &Path) -> std::io::Result<String> {
    let file_url = self
      .env
      .try_files(path)
      .map_err(|e| std::io::Error::new(std::io::ErrorKind::NotFound, e))?;
    let file_path = file_url.to_file_path().expect("should be a file URL");
    fs::read_to_string(file_path)
  }
}

impl PathResolver for SphinxFileLoader {
  fn resolve(&self, path: Option<&str>, base_file: Option<&Path>) -> Option<PathBuf> {
    let path = match (path, base_file.as_ref()) {
      (None, None) => return None,
      (None, Some(_)) => return base_file.map(PathBuf::from),
      (Some(path), _) => PathBuf::from(path),
    };

    let base = base_file.and_then(|p| Url::from_file_path(p).ok());

    let bases = if let Some(base) = base {
      try_files(
        &path,
        [&base, &self.env.src_dir, &self.env.cwd].iter().copied(),
      )
      .ok()
    } else {
      try_files(&path, [&self.env.src_dir, &self.env.cwd].iter().copied()).ok()
    };

    bases.map(|url| url.to_file_path().expect("should be a file URL"))
  }
}

fn try_files<'u>(path: &Path, mut bases: impl Iterator<Item = &'u Url>) -> anyhow::Result<Url> {
  bases
    .find_map(|base| {
      let maybe_file_url = try_join_path(base, path).ok()?;
      let maybe_file = maybe_file_url.to_file_path().expect("should be a file URL");
      let metadata = std::fs::metadata(maybe_file).ok()?;
      if metadata.is_file() {
        Some(maybe_file_url)
      } else {
        None
      }
    })
    .ok_or_else(|| anyhow::anyhow!("file not found: {:?}", path))
}

fn try_join_path(u: &Url, path: &Path) -> anyhow::Result<Url> {
  Ok(
    u.join(
      path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("invalid path: {:?}", path))?,
    )?,
  )
}
