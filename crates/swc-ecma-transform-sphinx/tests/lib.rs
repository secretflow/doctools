use std::path::PathBuf;

use swc_core::{
  common::{chain, sync::Lrc, SourceMap},
  ecma::parser::parse_file_as_module,
  testing::fixture,
};

use deno_lite::DenoLite;
use swc_ecma_testing2::{parse_one, test_js_fixture};

use swc_ecma_transform_sphinx::{
  init_esm, render_code, render_math, render_raw, render_typography,
};
use swc_ecma_utils2::{ecma::fixes::remove_invalid, jsx::JSXSymbols};

#[fixture("tests/fixtures/**/*.js")]
fn test_transforms(path: PathBuf) {
  let deno = DenoLite::default();
  let esm = init_esm(deno).unwrap();
  test_js_fixture(
    path,
    |src| parse_one(&src.src, None, parse_file_as_module).unwrap(),
    |_: ()| {
      let sourcemap = <Lrc<SourceMap>>::default();
      chain!(
        render_code::<JSXSymbols>(sourcemap.clone(), &esm),
        render_math::<JSXSymbols>(sourcemap.clone(), &esm),
        render_raw::<JSXSymbols>(sourcemap,),
        render_typography::<JSXSymbols>(),
        remove_invalid(),
      )
    },
  )
}
