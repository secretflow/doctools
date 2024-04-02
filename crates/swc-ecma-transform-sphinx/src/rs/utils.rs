#[macro_export]
macro_rules! move_basic_attributes {
  ($R:ty, Expr($from:ident), $into:ident) => {{
    let mut from_elem = swc_ecma_utils2::jsx::jsx_mut::<R>(&mut $from);
    let mut from_props = from_elem.get_props_mut();
    let mut into_elem = swc_ecma_utils2::jsx::jsx_mut::<R>(&mut $into);
    let mut into_props = into_elem.get_props_mut();
    into_props.mut_item("ids", |f| {
      f.as_mut_array()
        .extend(from_props.get_item_mut("ids").drain().map(|(_, v)| v))
    });
    into_props.mut_item("classes", |f| {
      f.as_mut_array()
        .extend(from_props.get_item_mut("classes").drain().map(|(_, v)| v))
    });
    into_props.mut_item("names", |f| {
      f.as_mut_array()
        .extend(from_props.get_item_mut("names").drain().map(|(_, v)| v))
    });
    into_props.mut_item("dupnames", |f| {
      f.as_mut_array()
        .extend(from_props.get_item_mut("dupnames").drain().map(|(_, v)| v))
    });
  }};
}
