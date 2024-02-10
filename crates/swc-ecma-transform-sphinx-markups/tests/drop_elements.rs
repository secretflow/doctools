use std::path::PathBuf;

use swc_core::{
  common::chain,
  ecma::{parser::parse_file_as_module, transforms::testing::test},
  testing::fixture,
};

use swc_ecma_testing2::{parse_one, test_fixture};
use swc_ecma_transform_sphinx_markups::drop_elements::drop_elements;
use swc_ecma_utils2::{
  ecma::sanitize::remove_invalid,
  jsx::{
    sanitize::{fix_jsx_factories, fold_fragments},
    JSXRuntimeDefault,
  },
  jsx_tag,
};

#[fixture("tests/fixtures/drop_elements/*.js")]
fn test(path: PathBuf) {
  test_fixture(
    path,
    |src| parse_one(&src.src, None, parse_file_as_module).unwrap(),
    |_: ()| {
      chain!(
        drop_elements::<JSXRuntimeDefault>(|options| options
          .delete(jsx_tag!("comment"))
          .unwrap(jsx_tag!("div"))),
        fold_fragments::<JSXRuntimeDefault>(),
        fix_jsx_factories::<JSXRuntimeDefault>(),
        remove_invalid(),
      )
    },
  )
}
