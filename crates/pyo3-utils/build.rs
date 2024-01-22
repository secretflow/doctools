use maturin_utils::{include_libpython_in_dev, Result};

fn main() -> Result<()> {
  include_libpython_in_dev(env!("CARGO_PKG_NAME"))?;
  Ok(())
}
