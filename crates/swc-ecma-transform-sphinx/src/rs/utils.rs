#[macro_export]
macro_rules! move_basic_attributes {
  ($R:ty, $from:ident, $into:ident) => {{
    let mut elem = swc_ecma_utils2::jsx::jsx_mut::<R>(&mut $into);
    let mut props = elem.get_props_mut();
    props.mut_item("ids", |f| {
      f.as_mut_array()
        .extend($from.ids.into_iter().map(|s| s.into()))
    });
    props.mut_item("classes", |f| {
      f.as_mut_array()
        .extend($from.classes.into_iter().map(|s| s.into()))
    });
    props.mut_item("names", |f| {
      f.as_mut_array()
        .extend($from.names.into_iter().map(|s| s.into()))
    });
    props.mut_item("dupnames", |f| {
      f.as_mut_array()
        .extend($from.dupnames.into_iter().map(|s| s.into()))
    });
  }};
}
