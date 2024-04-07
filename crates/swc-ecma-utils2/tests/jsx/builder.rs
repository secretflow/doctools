use swc_ecma_testing2::{insta, print_one_unchecked};
use swc_ecma_utils2::{
  json_expr,
  jsx::{create_element, create_fragment, DocumentBuilder, JSXRuntimeDefault},
  tag,
};

#[test]
fn test_fragment() {
  let document = print_one_unchecked(&create_fragment::<JSXRuntimeDefault>().guarantee());
  insta::assert_snapshot!(document);
}

#[test]
fn test_intrinsic() {
  let document = print_one_unchecked(&{
    create_element::<JSXRuntimeDefault>(tag!("div"))
      .child("foo".into())
      .guarantee()
  });
  insta::assert_snapshot!(document);
}

#[test]
fn test_component() {
  let document = print_one_unchecked(&create_element::<JSXRuntimeDefault>(tag!(Foo)).guarantee());
  insta::assert_snapshot!(document);
}

#[test]
fn test_props() {
  let document = print_one_unchecked(&{
    create_element::<JSXRuntimeDefault>(tag!("div"))
      .prop("className", &"foo")
      .prop("id", &"bar")
      .guarantee()
  });
  insta::assert_snapshot!(document);
}

fn document_builder(build: impl Fn(&mut DocumentBuilder<JSXRuntimeDefault>)) -> String {
  let mut builder = DocumentBuilder::new();
  build(&mut builder);
  print_one_unchecked(&builder.to_document().to_module())
}

#[test]
fn test_document_fragment() {
  let document = document_builder(|builder| {
    builder.element(tag!(<>), json_expr!({}).object(), None);
  });
  insta::assert_snapshot!(document);
}

#[test]
fn test_document_intrinsic() {
  let document = document_builder(|builder| {
    builder
      .element(tag!("div"), json_expr!({}).object(), None)
      .enter(&["children"])
      .value("foo".into())
      .exit();
  });
  insta::assert_snapshot!(document);
}

#[test]
fn test_document_props() {
  let document = document_builder(|builder| {
    builder
      .element(
        tag!("a"),
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
fn test_document_head() {
  let document = document_builder(|builder| {
    builder
      .element(tag!("section"), None, None)
      .enter(&["children"])
      .element(tag!("style"), None, None)
      .enter(&["children"])
      .value("p { background: #fff; }".into())
      .exit()
      .element(
        tag!("link"),
        json_expr!({
          "rel": "preconnect",
          "href": "https://rsms.me/"
        })
        .object(),
        None,
      )
      .element(tag!("p"), None, None)
      .enter(&["children"])
      .value("Lorem ipsum".into())
      .exit()
      .exit();
  });
  insta::assert_snapshot!(document);
}
