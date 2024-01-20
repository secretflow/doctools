use std::rc::Rc;

use deno_core::{
  v8::{self},
  ModuleSpecifier, RuntimeOptions, Snapshot, StaticModuleLoader,
};
use deno_runtime_web_subset::{create_runtime, create_runtime_snapshot};
use deno_web::TimersPermission;
use tempfile::tempdir;
use tokio;

struct Permissions;

impl TimersPermission for Permissions {
  fn allow_hrtime(&mut self) -> bool {
    true
  }
}

#[test]
fn test_integrated() {
  let dir = tempdir().unwrap();

  let snapshot_dest = dir.path().join("snapshot.bin");

  create_runtime_snapshot(
    env!("CARGO_MANIFEST_DIR"),
    snapshot_dest.clone().to_str().unwrap(),
  );

  tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()
    .unwrap()
    .block_on(async {
      let snapshot = Box::new(std::fs::read(snapshot_dest).unwrap()).into_boxed_slice();
      let main_module = ModuleSpecifier::parse("main:").unwrap();

      let mut runtime = create_runtime::<Permissions>(
        RuntimeOptions {
          module_loader: Some(Rc::new(StaticModuleLoader::new(vec![(
            main_module.clone(),
            r#"
            console.log(globalThis.__version__);
            const __version__ = globalThis.__version__;
            export { __version__ };
            "#
            .to_string()
            .into(),
          )]))),
          ..Default::default()
        },
        Snapshot::Boxed(snapshot),
      );

      let main = runtime.load_main_module(&main_module, None).await?;
      let eval = runtime.mod_evaluate(main);

      runtime.run_event_loop(Default::default()).await?;

      {
        let global = runtime.get_module_namespace(main).unwrap();
        let scope = &mut runtime.handle_scope();

        let k_version = v8::String::new(scope, "__version__").unwrap();
        let k_v8 = v8::String::new(scope, "v8").unwrap();

        let v8_version = global
          .open(scope)
          .get(scope, k_version.into())
          .unwrap()
          .to_object(scope)
          .unwrap()
          .get(scope, k_v8.into())
          .unwrap()
          .to_string(scope)
          .unwrap();

        assert_eq!(
          v8_version.to_rust_string_lossy(scope),
          v8::V8::get_version()
        );
      }

      {
        let object = runtime
          .execute_script_static("", "typeof TextDecoder")
          .unwrap();
        let scope = &mut runtime.handle_scope();

        assert_eq!(
          object
            .open(scope)
            .to_string(scope)
            .unwrap()
            .to_rust_string_lossy(scope),
          "function"
        );
      }

      eval.await
    })
    .unwrap();
}
