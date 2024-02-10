use swc_core::ecma::ast::Ident;

use swc_ecma_testing2::{insta, print_one_unchecked};
use swc_ecma_utils2::{
  collections::MutableMapping,
  jsx::{create_element, create_fragment, jsx_mut, JSXElementMut, JSXRuntimeDefault},
};

#[test]
fn test_fragment() {
  let document = print_one_unchecked(&create_fragment::<JSXRuntimeDefault>());
  insta::assert_snapshot!(document);
}

#[test]
fn test_intrinsic() {
  let document = print_one_unchecked(&{
    let mut elem = create_element::<JSXRuntimeDefault>("div".into());
    jsx_mut::<JSXRuntimeDefault>(&mut elem)
      .get_props_mut()
      .set_item("children", "foo".into());
    elem
  });
  insta::assert_snapshot!(document);
}

#[test]
fn test_component() {
  let document = print_one_unchecked(&create_element::<JSXRuntimeDefault>(
    Ident::new("Foo".into(), Default::default()).into(),
  ));
  insta::assert_snapshot!(document);
}

#[test]
fn test_props() {
  let document = print_one_unchecked(&{
    let mut elem = create_element::<JSXRuntimeDefault>("div".into());
    jsx_mut::<JSXRuntimeDefault>(&mut elem)
      .get_props_mut()
      .set_item("className", "foo".into())
      .set_item("id", "bar".into());
    elem
  });
  insta::assert_snapshot!(document);
}
