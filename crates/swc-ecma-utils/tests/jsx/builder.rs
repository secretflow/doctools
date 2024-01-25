use serde_json::json;
use swc_core::{
  common::sync::Lazy,
  ecma::transforms::{base::pass::noop, testing::test},
};

use swc_ecma_utils::{
  ast::json_to_expr,
  jsx::{
    builder::DocumentBuilder,
    factory::{JSXRuntime, JSXTagName},
  },
  testing::{document_as_module, print_one_unwrap},
};

static JSX_RUNTIME: Lazy<JSXRuntime> = Lazy::new(|| JSXRuntime::default());

fn build_jsx(build: impl Fn(&mut DocumentBuilder)) -> String {
  let mut builder = DocumentBuilder::new(JSX_RUNTIME.clone());
  build(&mut builder);
  print_one_unwrap(&document_as_module(builder.declare()))
}

test!(
  Default::default(),
  |_| noop(),
  fragment,
  build_jsx(|builder| {
    builder.element(&JSXTagName::Fragment, None, None);
  })
  .as_str()
);

test!(
  Default::default(),
  |_| noop(),
  intrinsic,
  build_jsx(|builder| {
    builder
      .element(&"div".into(), None, None)
      .enter(&["children"])
      .value("foo".into())
      .exit();
  })
  .as_str()
);

test!(
  Default::default(),
  |_| noop(),
  props,
  build_jsx(|builder| {
    builder
      .element(
        &"a".into(),
        Some(json_to_expr(
          json!({"href": "https://example.com", "title": "Example"}),
        )),
        None,
      )
      .enter(&["children"])
      .value("Example".into())
      .exit();
  })
  .as_str()
);

test!(
  Default::default(),
  |_| noop(),
  head,
  build_jsx(|builder| {
    builder
      .element(&"section".into(), None, None)
      .enter(&["children"])
      .element(&"style".into(), None, None)
      .enter(&["children"])
      .value("p { background: #fff; }".into())
      .exit()
      .element(
        &"link".into(),
        Some(json_to_expr(
          json!({"rel": "preconnect", "href": "https://rsms.me/"}),
        )),
        None,
      )
      .element(&"p".into(), None, None)
      .enter(&["children"])
      .value("Lorem ipsum".into())
      .exit()
      .exit();
  })
  .as_str()
);
