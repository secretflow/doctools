use std::path::PathBuf;

use swc_core::{ecma::parser::parse_file_as_module, testing::fixture};

use swc_ecma_utils::testing::{parse_one, test_fixture};

use swc_ecma_transforms_i18n::i18n;

#[fixture("tests/fixtures/*.in.js")]
fn test_i18n(source_path: PathBuf) {
  let mut messages = vec![];
  test_fixture(
    source_path,
    |source| parse_one(&source.src, None, parse_file_as_module).unwrap(),
    |runtime, config| i18n(runtime, config, &mut messages),
  );
}
