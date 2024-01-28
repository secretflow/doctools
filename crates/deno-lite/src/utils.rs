use std::path::Path;

use deno_core::snapshot_util::{create_snapshot, CreateSnapshotOptions};
use deno_web::TimersPermission;

struct NullPermissions;

impl TimersPermission for NullPermissions {
  fn allow_hrtime(&mut self) -> bool {
    unreachable!()
  }
}

pub fn build_runtime_snapshot(cargo_manifest_dir: &'static str, out_file: &str) {
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
      crate::global_this::global_this::init_ops_and_esm(),
    ],
    compression_cb: None,
    with_runtime_cb: None,
    skip_op_registration: false,
  });
  for path in output.files_loaded_during_snapshot {
    println!("cargo:rerun-if-changed={}", path.display());
  }
}
