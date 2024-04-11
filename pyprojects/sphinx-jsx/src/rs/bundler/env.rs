use std::{
  fs,
  path::{Path, PathBuf},
};

use anyhow::Context;
use fuzzy_sourcemap::PathResolver;
use pyo3::FromPyObject;
use swc_core::common::FileLoader;
use swc_core::common::{sync::Lrc, FileName};
use url::Url;

#[derive(FromPyObject, Clone)]
pub struct SphinxOptions {
  srcdir: String,
  outdir: String,
  conf: SphinxConfig,
}

#[derive(FromPyObject, Default, Clone)]
pub struct SphinxConfig {
  pub extensions: Option<Vec<String>>,
  pub myst_enable_extensions: Option<Vec<String>>,
}

pub struct Environment {
  pub cwd: Url,
  pub src_dir: Url,
  pub out_dir: Url,
  pub conf: SphinxConfig,
}

impl TryFrom<SphinxOptions> for Environment {
  type Error = anyhow::Error;

  fn try_from(options: SphinxOptions) -> anyhow::Result<Self> {
    let cwd = std::env::current_dir().context("failed to get current working directory")?;

    let SphinxOptions {
      srcdir,
      outdir,
      conf,
    } = options;

    let src_dir = cwd.join(srcdir).canonicalize()?;

    let out_dir = src_dir.join(outdir);

    let cwd = Url::from_directory_path(cwd).expect("should be a directory");
    let src_dir = Url::from_directory_path(src_dir).expect("should be a directory");
    let out_dir = Url::from_directory_path(out_dir).expect("should be a directory");

    Ok(Self {
      cwd,
      src_dir,
      out_dir,
      conf,
    })
  }
}

impl Environment {
  fn try_files(&self, path: &Path) -> anyhow::Result<Url> {
    try_files(path, [&self.src_dir, &self.cwd].iter().copied())
  }

  pub fn docname(&self, name: &FileName) -> String {
    let name = name.to_string();
    name
      .strip_prefix(self.src_dir.path())
      .map(str::to_string)
      .unwrap_or(name)
  }
}

#[derive(Clone)]
pub struct SphinxFileLoader {
  env: Lrc<Environment>,
}

impl From<Lrc<Environment>> for SphinxFileLoader {
  fn from(env: Lrc<Environment>) -> Self {
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
