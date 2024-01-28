use swc_core::ecma::{
  ast::{Expr, Ident},
  transforms::{base::pass::noop, testing::test},
};

use swc_ecma_utils::{
  jsx::factory::{JSXRuntime, JSXTagName},
  testing::print_one_unwrap,
};

test!(
  Default::default(),
  |_| noop(),
  fragment,
  print_one_unwrap(&JSXRuntime::default().create(&JSXTagName::Fragment).build()).as_str()
);

test!(
  Default::default(),
  |_| noop(),
  intrinsic,
  print_one_unwrap({
    &JSXRuntime::default()
      .create(&JSXTagName::Intrinsic("div".into()))
      .children(vec![Box::from(Expr::from(Ident::from("foo")))])
      .build()
  })
  .as_str()
);

test!(
  Default::default(),
  |_| noop(),
  component,
  print_one_unwrap(
    &JSXRuntime::default()
      .create(&JSXTagName::Ident("Foo".into()))
      .build()
  )
  .as_str()
);

test!(
  Default::default(),
  |_| noop(),
  jsxs,
  print_one_unwrap({
    &JSXRuntime::default()
      .create(&JSXTagName::Intrinsic("div".into()))
      .children(vec![
        JSXRuntime::default()
          .create(&JSXTagName::Intrinsic("span".into()))
          .build()
          .into(),
        JSXRuntime::default()
          .create(&JSXTagName::Intrinsic("span".into()))
          .build()
          .into(),
      ])
      .build()
  })
  .as_str()
);

test!(
  Default::default(),
  |_| noop(),
  props,
  print_one_unwrap({
    &JSXRuntime::default()
      .create(&JSXTagName::Intrinsic("div".into()))
      .prop("className", "foo".into(), None)
      .prop("id", "bar".into(), None)
      .build()
  })
  .as_str()
);
