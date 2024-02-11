use deno_core::anyhow;
use deno_web::TimersPermission;

use deno_lite::{define_deno_export, DenoLite};
use serde::Serialize;

struct Permissions;

impl TimersPermission for Permissions {
  fn allow_hrtime(&mut self) -> bool {
    true
  }
}

#[test]
fn test_function() -> anyhow::Result<()> {
  let mut deno = DenoLite::new(None);

  #[derive(Serialize)]
  struct Add {
    a: i32,
    b: i32,
  }

  define_deno_export!(add, Add);

  let module = deno.load_module_once(
    r#"
    export function add({ a, b }) {
      return a + b;
    }
    "#,
  )?;

  let result: i32 = deno.call_function(module, Add { a: 1, b: 2 })?;

  assert_eq!(result, 3);

  Ok(())
}

#[test]
fn test_async_function() -> anyhow::Result<()> {
  let mut deno = DenoLite::new(None);

  #[derive(Serialize)]
  struct Sleep {
    ms: f64,
  }

  define_deno_export!(sleep, Sleep);

  let module = deno.load_module_once(
    r#"
    export function sleep({ ms }) {
      return new Promise(resolve => setTimeout(() => resolve("done"), ms));
    }
    "#,
  )?;

  let now = std::time::Instant::now();

  let result: String = deno.call_function(module, Sleep { ms: 500_f64 })?;

  assert!(now.elapsed().as_millis() >= 500);
  assert_eq!(result, "done");

  Ok(())
}

#[test]
fn test_top_level_await() -> anyhow::Result<()> {
  let mut deno = DenoLite::new(None);

  let now = std::time::Instant::now();

  deno.load_module_once(
    r#"
    export function sleep({ ms }) {
      return new Promise(resolve => setTimeout(() => resolve("done"), ms));
    }
    await sleep({ ms: 500 });
    "#,
  )?;

  assert!(now.elapsed().as_millis() >= 500);

  Ok(())
}

#[test]
fn test_non_object_args() -> anyhow::Result<()> {
  let mut deno = DenoLite::new(None);

  #[derive(Serialize)]
  struct Add(i32, i32);

  define_deno_export!(add, Add);

  let module = deno.load_module_once(r#"export const add = ([a, b]) => a + b"#)?;

  let result: i32 = deno.call_function(module, Add(1, 2))?;

  assert_eq!(result, 3);

  Ok(())
}

#[test]
fn test_throw() -> anyhow::Result<()> {
  let mut deno = DenoLite::new(None);

  #[derive(Serialize)]
  struct Add(i32, i32);

  define_deno_export!(add, Add);

  let module = deno.load_module_once(
    r#"
    export function add([a, b]) {
      throw new Error("oops");
    }
    "#,
  )?;

  let result: anyhow::Result<i32> = deno.call_function(module, Add(1, 2));

  match result {
    Ok(_) => panic!("expected an error"),
    Err(err) => assert!(err.to_string().contains("Error: oops")),
  }

  Ok(())
}
