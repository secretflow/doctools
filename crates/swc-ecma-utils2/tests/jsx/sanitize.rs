use std::path::PathBuf;

use serde_json::Value;
use swc_core::{common::chain, ecma::parser::parse_file_as_module, testing::fixture};

use swc_ecma_testing2::{parse_one, test_fixture};
use swc_ecma_utils2::{
  ecma::sanitize::remove_invalid,
  jsx::{
    sanitize::{fix_jsx_factories, fold_fragments},
    JSXRuntimeDefault,
  },
};

#[fixture("tests/jsx/fixtures/sanitize/fold-fragments/*.js")]
fn test_fold_fragments(path: PathBuf) {
  test_fixture(
    path,
    |src| parse_one(&src.src, None, parse_file_as_module).unwrap(),
    |_: Value| chain!(fold_fragments::<JSXRuntimeDefault>(), remove_invalid()),
  )
}

#[fixture("tests/jsx/fixtures/sanitize/fix-jsx-factories/*.js")]
fn test_fix_jsx_factories(path: PathBuf) {
  test_fixture(
    path,
    |src| parse_one(&src.src, None, parse_file_as_module).unwrap(),
    |_: Value| chain!(fix_jsx_factories::<JSXRuntimeDefault>(), remove_invalid()),
  )
}
