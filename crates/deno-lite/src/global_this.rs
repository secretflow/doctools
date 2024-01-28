use deno_core::{extension, op2};
use serde::Serialize;

extension!(
  global_this,
  deps = [deno_webidl, deno_console, deno_url, deno_web],
  ops = [op_snapshot_versions],
  esm_entry_point = "ext:global_this/index.js",
  esm = [dir "static/main", "index.js"],
);

#[derive(Serialize)]
struct SnapshotVersions {
  // pub deno: &'static str,
  pub v8: &'static str,
  pub target: String,
}

impl Default for SnapshotVersions {
  fn default() -> Self {
    Self {
      // deno: "",
      v8: deno_core::v8_version(),
      target: std::env::var("TARGET").unwrap_or_default(),
    }
  }
}

#[op2]
#[serde]
fn op_snapshot_versions() -> SnapshotVersions {
  Default::default()
}
