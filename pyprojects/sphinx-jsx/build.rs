use maturin_utils::{assert_versions, include_libpython_in_dev, Result};

fn main() -> Result<()> {
  assert_versions()?;
  include_libpython_in_dev(env!("CARGO_PKG_NAME"))?;
  Ok(())
}
