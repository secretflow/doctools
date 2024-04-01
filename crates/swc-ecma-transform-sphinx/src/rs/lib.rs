use deno_lite::{anyhow, esm_source, DenoLite, ESModule};

esm_source!(SERVER, "render-code", "../../dist/server/index.js");

pub fn init_esm(mut deno: DenoLite) -> anyhow::Result<ESModule> {
  deno.create_module(&SERVER)
}

mod render_code;
mod render_math;

pub use render_code::render_code;
pub use render_math::render_math;
