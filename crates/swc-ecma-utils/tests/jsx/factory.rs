use swc_core::{
  common::sync::Lazy,
  ecma::{
    ast::{Expr, Ident},
    transforms::{base::pass::noop, testing::test},
  },
};

use swc_ecma_utils::{
  jsx::factory::{JSXRuntime, JSXTagName},
  testing::print_one_unwrap,
};

static JSX_RUNTIME: Lazy<JSXRuntime> = Lazy::new(|| JSXRuntime::default());

test!(
  Default::default(),
  |_| noop(),
  fragment,
  print_one_unwrap(&JSX_RUNTIME.create(&JSXTagName::Fragment).build()).as_str()
);

test!(
  Default::default(),
  |_| noop(),
  intrinsic,
  print_one_unwrap({
    &JSX_RUNTIME
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
  print_one_unwrap(&JSX_RUNTIME.create(&JSXTagName::Ident("Foo".into())).build()).as_str()
);

test!(
  Default::default(),
  |_| noop(),
  jsxs,
  print_one_unwrap({
    &JSX_RUNTIME
      .create(&JSXTagName::Intrinsic("div".into()))
      .children(vec![
        JSX_RUNTIME
          .create(&JSXTagName::Intrinsic("span".into()))
          .build()
          .into(),
        JSX_RUNTIME
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
    &JSX_RUNTIME
      .create(&JSXTagName::Intrinsic("div".into()))
      .prop("className", "foo".into(), None)
      .prop("id", "bar".into(), None)
      .build()
  })
  .as_str()
);
