use std::path::PathBuf;

use swc_core::{common::chain, ecma::parser::parse_file_as_module, testing::fixture};

use swc_ecma_testing2::{parse_one, test_fixture};
use swc_ecma_transform_i18n::{i18n, I18nSymbols};
use swc_ecma_utils2::jsx::{
  sanitize::{fix_jsx_factories, fold_fragments},
  JSXRuntime,
};

struct Runtime;

impl JSXRuntime for Runtime {
  const JSX: &'static str = "jsx";
  const JSXS: &'static str = "jsxs";
  const FRAGMENT: &'static str = "Fragment";
}

impl I18nSymbols for Runtime {
  const GETTEXT: &'static str = "i18n";
  const TRANS: &'static str = "Trans";
}

#[fixture("tests/fixtures/*.in.js")]
fn test_i18n(source_path: PathBuf) {
  let mut messages = vec![];
  test_fixture(
    source_path,
    |source| parse_one(&source.src, None, parse_file_as_module).unwrap(),
    |config| {
      chain!(
        i18n::<'_, Runtime, Runtime>(config, &mut messages),
        fold_fragments::<Runtime>(),
        fix_jsx_factories::<Runtime>()
      )
    },
  );
}
