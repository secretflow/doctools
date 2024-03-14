use std::{env, error, fmt, fs::File, io::Read, path::PathBuf};

use anyhow::Context;
use serde::Deserialize;
use serde_json;
use toml;

#[derive(Deserialize)]
struct CargoManifest {
  workspace_root: String,
}

// https://github.com/mitsuhiko/insta/blob/b113499249584cb650150d2d01ed96ee66db6b30/src/runtime.rs#L67-L88
pub fn get_cargo_workspace(manifest_dir: &str) -> anyhow::Result<PathBuf> {
  let output = std::process::Command::new(env!("CARGO"))
    .arg("metadata")
    .arg("--format-version=1")
    .current_dir(manifest_dir)
    .output()?;
  let manifest: CargoManifest = serde_json::from_slice(&output.stdout)?;
  Ok(PathBuf::from(manifest.workspace_root))
}

#[derive(Debug, Clone)]
struct VersionOutOfSyncError {
  package_json: String,
  pyproject_toml: String,
}

impl fmt::Display for VersionOutOfSyncError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(
      f,
      "💥 version in package.json ({}) does not match project.version ({}) in pyproject.toml",
      self.package_json, self.pyproject_toml
    )
  }
}

impl error::Error for VersionOutOfSyncError {}

#[derive(Deserialize, Debug)]
struct PackageJSON {
  version: Option<String>,
}

fn read_package_json() -> anyhow::Result<PackageJSON> {
  let mut buffer = Vec::new();
  let _ = File::open("package.json")?.read_to_end(&mut buffer)?;
  let package_json: PackageJSON = serde_json::from_slice(&buffer)?;
  Ok(package_json)
}

#[derive(Deserialize, Debug)]
struct PyProjectProjectTable {
  version: Option<String>,
}

#[derive(Deserialize, Debug)]
struct PyProjectTOML {
  project: Option<PyProjectProjectTable>,
}

fn read_pyproject_toml() -> anyhow::Result<PyProjectTOML> {
  let mut buffer = String::new();
  let _ = File::open("pyproject.toml")?.read_to_string(&mut buffer)?;
  let pyproject_toml: PyProjectTOML = toml::from_str(&buffer)?;
  Ok(pyproject_toml)
}

/// For publishable wheels, asserts that the version in package.json
/// matches the version in pyproject.toml
pub fn assert_versions_in_sync() -> anyhow::Result<()> {
  let package_json = read_package_json()?;

  let expected_version = package_json
    .version
    .with_context(|| "Cannot read `version` from package.json")?;

  let pyproject_toml = read_pyproject_toml()?;

  let actual_version = pyproject_toml
    .project
    .with_context(|| "Cannot read [project] from pyproject.toml")?
    .version
    .with_context(|| "Cannot read `project.version` from pyproject.toml")?;

  if expected_version != actual_version {
    Err(
      VersionOutOfSyncError {
        package_json: expected_version,
        pyproject_toml: actual_version,
      }
      .into(),
    )
  } else {
    Ok(())
  }
}

/// Set PYO3_PYTHON to the python interpreter in the virtualenv if it isn't already set
pub fn use_libpython_from_venv(package_name: &str, manifest_dir: &str) {
  let interpreter_path = if let Ok(envvar) = env::var("PYO3_PYTHON") {
    PathBuf::from(envvar)
  } else {
    match get_cargo_workspace(manifest_dir) {
      Err(_) => {
        println!(
          "cargo:warning={}: failed to get cargo workspace. will not configure PYO3_PYTHON",
          package_name
        );
        return;
      }
      Ok(workspace) => match workspace.join(".venv/bin/python").canonicalize() {
        Err(_) => {
          println!(
            "cargo:warning={}: cannot resolve virtualenv executable. will not configure PYO3_PYTHON",
            package_name,
          );
          return;
        }
        Ok(path) => path,
      },
    }
  };

  let lib_path = match interpreter_path.join("../../lib").canonicalize() {
    Err(_) => {
      println!(
        "cargo:warning={}: cannot resolve virtualenv lib path. will not configure PYO3_PYTHON",
        package_name,
      );
      return;
    }
    Ok(path) => path,
  };

  println!(
    "cargo:warning={}: pyo3: using interpreter at {}",
    package_name,
    interpreter_path.display()
  );

  println!("cargo:rerun-if-env-changed=PYO3_PYTHON");
  // for cargo build
  println!("cargo:rustc-env=PYO3_PYTHON={}", interpreter_path.display());
  println!("cargo:rustc-link-search=crate={}", lib_path.display());
  // for cargo run/test
  println!("cargo:rustc-env=DYLD_LIBRARY_PATH={}", lib_path.display());
  // for cargo run/test
  println!("cargo:rustc-env=LD_LIBRARY_PATH={}", lib_path.display());
}
