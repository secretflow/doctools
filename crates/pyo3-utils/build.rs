use pyo3_build_utils::use_libpython_from_venv;

fn main() {
  use_libpython_from_venv(env!("CARGO_PKG_NAME"), env!("CARGO_MANIFEST_DIR"))
}
