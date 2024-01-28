use std::path::PathBuf;

use serde_json::Value;
use swc_core::{ecma::transforms::base::pass::noop, testing::fixture};

use swc_ecma_utils::testing::{document_as_module, test_fixture};

use html5jsx::html_to_jsx;

#[fixture("tests/fixtures/*.html")]
fn test_conversion(source_path: PathBuf) {
  test_fixture(
    source_path,
    |source| document_as_module(html_to_jsx(&source, None).unwrap()),
    |_, _: Value| noop(),
  );
}

#[cfg(test)]
mod test_rejections {
  use html5jsx::html_to_jsx;
  use swc_core::common::{sync::Lrc, FileName, SourceFile, SourceMap};
  use swc_ecma_utils::jsx::factory::JSXRuntime;

  fn make_source(text: &str) -> Lrc<SourceFile> {
    let sourcemap: Lrc<SourceMap> = Default::default();
    let file = sourcemap.new_source_file(FileName::Anon, text.into());
    file
  }

  #[test]
  #[should_panic = "refuse to parse script tags"]
  fn no_unsafe_inline() {
    html_to_jsx(&make_source("<script>alert('Hi!');</script>"), None).unwrap();
  }

  #[test]
  #[should_panic = "refuse to parse script tags"]
  fn no_remote_script() {
    html_to_jsx(
      &make_source(
        r#"<script src="https://cdn.jsdelivr.net/npm/lodash@4.17.21/lodash.min.js"></script>"#,
      ),
      None,
    )
    .unwrap();
  }

  #[test]
  #[should_panic = "refuse to parse base tags"]
  fn no_base() {
    html_to_jsx(&make_source("<base href='https://example.com/' />"), None).unwrap();
  }

  #[test]
  #[should_panic = "refuse to convert event handlers"]
  fn no_on_click() {
    html_to_jsx(
      &make_source("<div onclick='alert(\"Hi!\")'>Hi!</div>"),
      None,
    )
    .unwrap();
  }

  #[test]
  #[should_panic = "refuse to convert event handlers"]
  fn no_arbitrary_event_handlers() {
    html_to_jsx(&make_source("<div onfoo='alert(\"Hi!\")'>Hi!</div>"), None).unwrap();
  }

  #[test]
  #[should_panic = "refuse to convert dangerouslySetInnerHTML"]
  fn no_dangerously_set_inner_html() {
    html_to_jsx(
      &make_source(
        "<div dangerouslySetInnerHTML={{__html: '<script>alert(\"Hi!\")</script>'}}></div>",
      ),
      None,
    )
    .unwrap();
  }

  #[test]
  #[should_panic = "refuse to convert `javascript:` URLs"]
  fn no_javascript_url() {
    html_to_jsx(
      &make_source("<a href='javascript:alert(\"Hi!\")'>Hi!</a>"),
      None,
    )
    .unwrap();
  }

  #[test]
  #[should_panic = "JSX factories cannot contain 'eval' or 'Function' in name"]
  fn no_malicious_jsx() {
    html_to_jsx(
      &make_source("<div>"),
      Some(JSXRuntime::aliased(
        "eval".into(),
        "jsxs".into(),
        "Fragment".into(),
      )),
    )
    .unwrap();
  }

  #[test]
  #[should_panic = "JSX factories cannot contain 'eval' or 'Function' in name"]
  fn no_malicious_jsx_2() {
    html_to_jsx(
      &make_source("<div>"),
      Some(JSXRuntime::aliased(
        "evaluate".into(),
        "jsxs".into(),
        "Fragment".into(),
      )),
    )
    .unwrap();
  }

  #[test]
  #[should_panic = "JSX factories cannot contain 'eval' or 'Function' in name"]
  fn no_malicious_jsxs() {
    html_to_jsx(
      &make_source("<div>"),
      Some(JSXRuntime::aliased(
        "jsxDEV".into(),
        "globalThis.eval".into(),
        "Fragment".into(),
      )),
    )
    .unwrap();
  }

  #[test]
  #[should_panic = "JSX factories cannot contain 'eval' or 'Function' in name"]
  fn no_malicious_fragment() {
    html_to_jsx(
      &make_source("<div>"),
      Some(JSXRuntime::aliased(
        "jsx".into(),
        "jsxs".into(),
        "window.Function".into(),
      )),
    )
    .unwrap();
  }
}
