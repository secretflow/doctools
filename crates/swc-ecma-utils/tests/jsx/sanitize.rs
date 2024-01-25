use swc_core::{
  common::{chain, sync::Lrc},
  ecma::transforms::testing::test,
};

use swc_ecma_utils::jsx::{factory::JSXRuntime, sanitize};

test!(
  Default::default(),
  |_| chain!(
    sanitize::fold_fragments(Lrc::new(JSXRuntime::aliased("_jsx", "_jsxs", "_Fragment"))),
    sanitize::remove_invalid()
  ),
  fold_fragments,
  r#"
  import { jsx as _jsx, jsxs as _jsxs, Fragment as _Fragment } from "react/jsx-runtime";
  _jsx(_Fragment, {
      children: _jsxs(_Fragment, {
          children: [
              "42",
              _jsx(_Fragment, {
                  children: _jsx(_Fragment, {
                      children: _jsx(_Fragment, {
                          children: _jsx(_Fragment, {})
                      })
                  })
              })
          ]
      })
  });
  "#
);

test!(
  Default::default(),
  |_| chain!(
    sanitize::fold_fragments(Lrc::new(Default::default())),
    sanitize::remove_invalid()
  ),
  fold_fragments_deep,
  r#"
  jsxs(Fragment, {
    children: [
      jsx(Fragment, {}),
      jsx(Fragment, {}),
      jsx(Fragment, {}),
      jsx(Fragment, {
        children: jsxs("div", {
          children: ["Lorem", jsx("span", {
            children: jsxs(Fragment, {
              children: [jsx("span", {
                children: "ipsum",
              }), jsxs("strong", {
                children: ["dolor", jsx("span", {
                  children: "sit",
                }), "amet"],
              })],
            }),
          })],
        }),
      }),
      jsx(Fragment, {}),
      jsx(Fragment, {}),
    ],
  });
  "#
);

test!(
  Default::default(),
  |_| chain!(
    sanitize::sanitize_jsx(Lrc::new(JSXRuntime::aliased("_jsx", "_jsxs", "_Fragment"))),
    sanitize::remove_invalid()
  ),
  sanitize_jsx,
  r#"
  import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
  _jsxs("div", {
      children: _jsx("span", {
          children: [
              "The quick brown ",
              _jsxs("strong", {
                  children: "fox"
              }),
              " jumps over the lazy dog."
          ]
      })
  });
    "#
);
