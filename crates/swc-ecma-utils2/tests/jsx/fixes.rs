use std::path::PathBuf;

use serde_json::Value;
use swc_core::{common::chain, ecma::parser::parse_file_as_module, testing::fixture};

use swc_ecma_testing2::{parse_one, test_js_fixture};
use swc_ecma_utils2::{
  ad_hoc_tag,
  ecma::fixes::remove_invalid,
  jsx::{
    fixes::{drop_elements, fix_jsx_factories, fold_fragments},
    JSXSymbols,
  },
};

#[fixture("tests/jsx/fixtures/fixes/**/*.js")]
fn test_fixes(path: PathBuf) {
  test_js_fixture(
    path,
    |src| parse_one(&src.src, None, parse_file_as_module).unwrap(),
    |_: Value| {
      chain!(
        drop_elements()
          .delete(ad_hoc_tag!("comment"))
          .unwrap(ad_hoc_tag!("section"))
          .build::<JSXSymbols>(),
        fold_fragments::<JSXSymbols>(),
        fix_jsx_factories::<JSXSymbols>(),
        remove_invalid()
      )
    },
  )
}
