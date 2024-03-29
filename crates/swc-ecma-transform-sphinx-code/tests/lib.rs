use std::path::PathBuf;

use swc_core::{ecma::parser::parse_file_as_module, testing::fixture};

use deno_lite::DenoLite;
use swc_ecma_testing2::{parse_one, test_fixture};

use swc_ecma_transform_sphinx_code::render_code;
use swc_ecma_utils2::jsx::JSXRuntimeDefault;

#[fixture("tests/fixtures/*.js")]
fn test_math(path: PathBuf) {
  test_fixture(
    path,
    |src| parse_one(&src.src, None, parse_file_as_module).unwrap(),
    |_: ()| render_code::<JSXRuntimeDefault>(DenoLite::default()),
  )
}
