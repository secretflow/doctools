use swc_core::{
  common::{
    chain,
    sync::{Lazy, Lrc},
  },
  ecma::transforms::testing::test,
};

use swc_ecma_transform_sphinx::drop_elements::drop_elements;
use swc_ecma_utils::{
  jsx::{factory::JSXRuntime, sanitize::sanitize_jsx},
  tag,
};

static JSX_RUNTIME: Lazy<Lrc<JSXRuntime>> =
  Lazy::new(|| Lrc::new(JSXRuntime::aliased("_jsx", "_jsxs", "_Fragment")));

test!(
  Default::default(),
  |_| chain!(
    drop_elements((*JSX_RUNTIME).clone(), |options| options
      .delete(tag!("comment"))
      .unwrap(tag!("div"))),
    sanitize_jsx((*JSX_RUNTIME).clone())
  ),
  drop_1,
  r#"
  import { jsx as _jsx, jsxs as _jsxs, Fragment as _Fragment } from "react/jsx-runtime";
  _jsxs(_Fragment, {
      children: [
          _jsxs("div", {
              children: [
                  "Lorem ipsum",
                  _jsx(_Fragment, {
                      children: _jsx("comment", {
                          children: "This is a comment"
                      })
                  }),
                  "dolor sit amet"
              ]
          }),
          _jsx("comment", {
              children: "This is another comment"
          })
      ]
  });

  "#
);
