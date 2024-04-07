use std::path::PathBuf;

use serde_json::Value;
use swc_core::{common::chain, testing::fixture};

use swc_ecma_testing2::test_js_fixture;
use swc_ecma_utils2::jsx::{
  fixes::{fix_jsx_factories, fold_fragments},
  JSXRuntime,
};

use html5jsx::html_to_jsx;

struct Runtime;

impl JSXRuntime for Runtime {
  const JSX: &'static str = "jsx";
  const JSXS: &'static str = "jsxs";
  const FRAGMENT: &'static str = "Fragment";
}

#[fixture("tests/fixtures/*.html")]
fn test_conversion(source_path: PathBuf) {
  test_js_fixture(
    source_path,
    |source| html_to_jsx::<Runtime>(&source).unwrap().to_module(),
    |_: Value| chain!(fold_fragments::<Runtime>(), fix_jsx_factories::<Runtime>()),
  );
}

#[cfg(test)]
mod test_rejections {
  use html5jsx::html_to_jsx;
  use swc_core::common::{sync::Lrc, FileName, SourceFile, SourceMap};

  use crate::Runtime;

  fn make_source(text: &str) -> Lrc<SourceFile> {
    let sourcemap: Lrc<SourceMap> = Default::default();
    let file = sourcemap.new_source_file(FileName::Anon, text.into());
    file
  }

  #[test]
  #[should_panic = "refuse to parse script tags"]
  fn no_unsafe_inline() {
    html_to_jsx::<Runtime>(&make_source("<script>alert('Hi!');</script>")).unwrap();
  }

  #[test]
  #[should_panic = "refuse to parse script tags"]
  fn no_remote_script() {
    html_to_jsx::<Runtime>(&make_source(
      r#"<script src="https://cdn.jsdelivr.net/npm/lodash@4.17.21/lodash.min.js"></script>"#,
    ))
    .unwrap();
  }

  #[test]
  #[should_panic = "refuse to parse base tags"]
  fn no_base() {
    html_to_jsx::<Runtime>(&make_source("<base href='https://example.com/' />")).unwrap();
  }

  #[test]
  #[should_panic = "refuse to convert event handlers"]
  fn no_on_click() {
    html_to_jsx::<Runtime>(&make_source("<div onclick='alert(\"Hi!\")'>Hi!</div>")).unwrap();
  }

  #[test]
  #[should_panic = "refuse to convert event handlers"]
  fn no_arbitrary_event_handlers() {
    html_to_jsx::<Runtime>(&make_source("<div onfoo='alert(\"Hi!\")'>Hi!</div>")).unwrap();
  }

  #[test]
  #[should_panic = "refuse to convert dangerouslySetInnerHTML"]
  fn no_dangerously_set_inner_html() {
    html_to_jsx::<Runtime>(&make_source(
      "<div dangerouslySetInnerHTML={{__html: '<script>alert(\"Hi!\")</script>'}}></div>",
    ))
    .unwrap();
  }

  #[test]
  #[should_panic = "refuse to convert `javascript:` URLs"]
  fn no_javascript_url() {
    html_to_jsx::<Runtime>(&make_source("<a href='javascript:alert(\"Hi!\")'>Hi!</a>")).unwrap();
  }
}
