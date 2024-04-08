use std::path::PathBuf;

use swc_core::{common::chain, ecma::parser::parse_file_as_module, testing::fixture};

use deno_lite::DenoLite;
use swc_ecma_testing2::{parse_one, test_js_fixture};

use swc_ecma_transform_sphinx_code::{
  init_esm, render_code, render_math, render_raw, render_typograph,
};
use swc_ecma_utils2::{ecma::fixes::remove_invalid, jsx::JSXRuntimeDefault};

#[fixture("tests/fixtures/**/*.js")]
fn test_transforms(path: PathBuf) {
  let deno = DenoLite::default();
  let esm = init_esm(deno).unwrap();
  test_js_fixture(
    path,
    |src| parse_one(&src.src, None, parse_file_as_module).unwrap(),
    |_: ()| {
      chain!(
        render_code::<JSXRuntimeDefault>(&esm),
        render_math::<JSXRuntimeDefault>(&esm),
        render_raw::<JSXRuntimeDefault>(),
        render_typograph::<JSXRuntimeDefault>(),
        remove_invalid(),
      )
    },
  )
}
