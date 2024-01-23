use pyo3_build_utils::{assert_versions_in_sync, use_libpython_from_venv};

fn main() -> anyhow::Result<()> {
  assert_versions_in_sync()?;
  use_libpython_from_venv(env!("CARGO_PKG_NAME"), env!("CARGO_MANIFEST_DIR"));
  Ok(())
}
