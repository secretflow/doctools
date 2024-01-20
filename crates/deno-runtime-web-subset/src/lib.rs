use std::{path::Path, vec};

use deno_core::{
  extension, op2,
  snapshot_util::{create_snapshot, CreateSnapshotOptions},
  JsRuntime, RuntimeOptions, Snapshot,
};
use deno_web::TimersPermission;
use serde::Serialize;

extension!(
  runtime,
  deps = [deno_webidl, deno_console, deno_url, deno_web],
  ops = [op_snapshot_versions],
  esm_entry_point = "ext:runtime/index.js",
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

struct NullPermissions;

impl TimersPermission for NullPermissions {
  fn allow_hrtime(&mut self) -> bool {
    unreachable!()
  }
}

pub fn create_runtime_snapshot(cargo_manifest_dir: &'static str, out_file: &str) {
  let output = create_snapshot(CreateSnapshotOptions {
    cargo_manifest_dir,
    snapshot_path: Path::new(cargo_manifest_dir).join(out_file),
    startup_snapshot: None,
    extensions: vec![
      deno_webidl::deno_webidl::init_ops_and_esm(),
      deno_console::deno_console::init_ops_and_esm(),
      deno_url::deno_url::init_ops_and_esm(),
      deno_web::deno_web::init_ops_and_esm::<NullPermissions>(
        Default::default(),
        Default::default(),
      ),
      deno_crypto::deno_crypto::init_ops_and_esm(None),
      runtime::init_ops_and_esm(),
    ],
    compression_cb: None,
    with_runtime_cb: None,
    skip_op_registration: false,
  });
  for path in output.files_loaded_during_snapshot {
    println!("cargo:rerun-if-changed={}", path.display());
  }
}

pub fn create_runtime<P>(options: RuntimeOptions, snapshot: Snapshot) -> JsRuntime
where
  P: TimersPermission + 'static,
{
  JsRuntime::new(RuntimeOptions {
    extensions: vec![
      deno_webidl::deno_webidl::init_ops(),
      deno_console::deno_console::init_ops(),
      deno_url::deno_url::init_ops(),
      deno_web::deno_web::init_ops::<P>(Default::default(), Default::default()),
      deno_crypto::deno_crypto::init_ops(None),
      runtime::init_ops(),
    ],
    startup_snapshot: Some(snapshot),
    ..options
  })
}
