use anyhow::Context;
pub use anyhow::Result;
use serde::Deserialize;
use serde_json;
use std::fs::File;
use std::io::Read;
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
        Err(VersionOutOfSyncError {
            package_json: expected_version,
            pyproject_toml: actual_version,
        }
        .into())
    } else {
        Ok(())
    }
}
