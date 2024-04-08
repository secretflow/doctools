use deno_lite::{anyhow, esm_source, DenoLite, ESModule};

esm_source!(SERVER, "server", "../../dist/server/index.js");

pub fn init_esm(mut deno: DenoLite) -> anyhow::Result<ESModule> {
  deno.create_module(&SERVER)
}

mod code;
mod components;
mod math;
mod raw;
mod typography;
mod utils;

pub mod macros;

pub use code::render_code;
pub use math::render_math;
pub use raw::render_raw;
pub use typography::render_typograph;
