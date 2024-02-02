use std::path::PathBuf;

use glob::glob;
use once_cell::sync::Lazy;
use swc_core::common::{sync::Lrc, FileName, SourceFile, SourceMap};

static SOURCES: Lazy<Vec<(FileName, String)>> = Lazy::new(load_fixture);

fn load_fixture() -> Vec<(FileName, String)> {
  let mut sources = vec![];
  for entry in glob(format!("{}/tests/fixtures/**/*", env!("CARGO_MANIFEST_DIR")).as_str()).unwrap()
  {
    let entry = match entry {
      Ok(entry) => entry,
      Err(_) => continue,
    };
    if !entry.is_file() {
      continue;
    }
    let content = std::fs::read_to_string(entry.clone()).unwrap();
    sources.push((FileName::Real(entry), content));
  }
  sources
}

pub fn get_fixture() -> Lrc<SourceMap> {
  let sources: Lrc<SourceMap> = Default::default();
  for (file_name, content) in SOURCES.iter() {
    sources.new_source_file(file_name.clone(), content.clone());
  }
  sources
}

pub fn get_fixture_file(sources: &Lrc<SourceMap>, file_name: &str) -> Lrc<SourceFile> {
  let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("tests/fixtures")
    .join(file_name);
  sources.get_source_file(&FileName::Real(path)).unwrap()
}
