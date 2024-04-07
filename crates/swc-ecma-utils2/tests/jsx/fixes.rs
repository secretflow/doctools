use std::path::PathBuf;

use serde_json::Value;
use swc_core::{common::chain, ecma::parser::parse_file_as_module, testing::fixture};

use swc_ecma_testing2::{parse_one, test_js_fixture};
use swc_ecma_utils2::{
  ecma::sanitize::remove_invalid,
  jsx::{
    fixes::{drop_elements, fix_jsx_factories, fold_fragments},
    JSXRuntimeDefault,
  },
  jsx_tag,
};

#[fixture("tests/jsx/fixtures/fixes/**/*.js")]
fn test_fixes(path: PathBuf) {
  test_js_fixture(
    path,
    |src| parse_one(&src.src, None, parse_file_as_module).unwrap(),
    |_: Value| {
      chain!(
        drop_elements()
          .delete(jsx_tag!("comment"))
          .unwrap(jsx_tag!("section"))
          .build::<JSXRuntimeDefault>(),
        fold_fragments::<JSXRuntimeDefault>(),
        fix_jsx_factories::<JSXRuntimeDefault>(),
        remove_invalid()
      )
    },
  )
}
