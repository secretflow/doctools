use swc_ecma_testing2::{insta, print_one_unchecked};
use swc_ecma_utils2::{
  json_expr,
  jsx::{DocumentBuilder, JSXRuntimeDefault},
};

fn build_jsx(build: impl Fn(&mut DocumentBuilder<JSXRuntimeDefault>)) -> String {
  let mut builder = DocumentBuilder::new();
  build(&mut builder);
  print_one_unchecked(&builder.to_document().to_module())
}

#[test]
fn test_fragment() {
  let document = build_jsx(|builder| {
    builder.element(None, json_expr!({}).object(), None);
  });
  insta::assert_snapshot!(document);
}

#[test]
fn test_intrinsic() {
  let document = build_jsx(|builder| {
    builder
      .element(Some("div".into()), json_expr!({}).object(), None)
      .enter(&["children"])
      .value("foo".into())
      .exit();
  });
  insta::assert_snapshot!(document);
}

#[test]
fn test_props() {
  let document = build_jsx(|builder| {
    builder
      .element(
        Some("a".into()),
        json_expr!({
          "href": "https://example.com",
          "title": "Example"
        })
        .object(),
        None,
      )
      .enter(&["children"])
      .value("Example".into())
      .exit();
  });
  insta::assert_snapshot!(document);
}

#[test]
fn test_head() {
  let document = build_jsx(|builder| {
    builder
      .element(Some("section".into()), None, None)
      .enter(&["children"])
      .element(Some("style".into()), None, None)
      .enter(&["children"])
      .value("p { background: #fff; }".into())
      .exit()
      .element(
        Some("link".into()),
        json_expr!({
          "rel": "preconnect",
          "href": "https://rsms.me/"
        })
        .object(),
        None,
      )
      .element(Some("p".into()), None, None)
      .enter(&["children"])
      .value("Lorem ipsum".into())
      .exit()
      .exit();
  });
  insta::assert_snapshot!(document);
}
