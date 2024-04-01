use deno_lite::{anyhow, esm_source, DenoLite, ESModule};

esm_source!(SERVER, "server", "../../dist/server/index.js");

pub fn init_esm(mut deno: DenoLite) -> anyhow::Result<ESModule> {
  deno.create_module(&SERVER)
}

mod code;
mod html;
mod math;
mod utils;

pub use code::render_code;
pub use math::render_math;
