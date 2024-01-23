use std::env;
use std::fs::File;
use std::io::Read;

use anyhow::Context;
pub use anyhow::Result;
use serde::Deserialize;
use serde_json;
use toml;

#[derive(Deserialize, Debug)]
struct PackageJSON {
  version: Option<String>,
}

#[derive(Deserialize, Debug)]
struct PyProjectProjectTable {
  version: Option<String>,
}

#[derive(Deserialize, Debug)]
struct PyProjectTOML {
  project: Option<PyProjectProjectTable>,
}

#[derive(Debug, Clone)]
struct VersionOutOfSyncError {
  package_json: String,
  pyproject_toml: String,
}

impl std::fmt::Display for VersionOutOfSyncError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(
      f,
      "💥 version in package.json ({}) does not match project.version ({}) in pyproject.toml",
      self.package_json, self.pyproject_toml
    )
  }
}

impl std::error::Error for VersionOutOfSyncError {}

fn read_package_json() -> Result<PackageJSON> {
  let mut buffer = Vec::new();
  let _ = File::open("package.json")?.read_to_end(&mut buffer)?;
  let package_json: PackageJSON = serde_json::from_slice(&buffer)?;
  Ok(package_json)
}

fn read_pyproject_toml() -> Result<PyProjectTOML> {
  let mut buffer = String::new();
  let _ = File::open("pyproject.toml")?.read_to_string(&mut buffer)?;
  let pyproject_toml: PyProjectTOML = toml::from_str(&buffer)?;
  Ok(pyproject_toml)
}

pub fn assert_versions() -> Result<()> {
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

pub fn include_libpython_in_dev(name: &str) -> Result<()> {
  match env::var("PYO3_PYTHON") {
    // building with maturin
    Ok(_) => return Ok(()),
    _ => (),
  };

  // figure out sys.base_prefix by executing python. this is icky.
  // (ignoring venv since lib won't be there)
  let proc = std::process::Command::new("python")
    .arg("-c")
    .arg("import sys\nsys.stdout.write(sys.base_prefix)")
    .output();

  let output = match proc {
    Ok(output) => output,
    Err(err) => match err.kind() {
      std::io::ErrorKind::NotFound => {
        println!(
          "cargo:warning={}: python not found, skipping libpython linking",
          name
        );
        return Ok(());
      }
      _ => return Err(err.into()),
    },
  };

  let base_prefix = String::from_utf8(output.stdout)?.trim().to_string();
  let lib_path = format!("{}/lib", base_prefix);
  println!("cargo:warning={}: Using libpython from {}", name, lib_path);
  // for cargo build
  println!("cargo:rustc-link-search=crate={}", lib_path);
  // for cargo run/test
  println!("cargo:rustc-env=DYLD_LIBRARY_PATH={}", lib_path);
  // for cargo run/test
  println!("cargo:rustc-env=LD_LIBRARY_PATH={}", lib_path);

  Ok(())
}
