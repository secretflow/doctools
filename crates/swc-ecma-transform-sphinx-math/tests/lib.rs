use deno_lite::DenoLite;
use swc_core::ecma::transforms::testing::test;

use swc_ecma_transform_sphinx_math::render_math;
use swc_ecma_utils::jsx::factory::JSXRuntime;

test!(
  Default::default(),
  |_| render_math(JSXRuntime::playground(), DenoLite::default()),
  test1,
  r#"
  import { jsx as _jsx } from "react/jsx-runtime";
  _jsx(math_block, {
    "backrefs": [],
    "classes": [],
    "docname": "demo/demo",
    "dupnames": [],
    "ids": [
        "equation-this-is-a-label"
    ],
    "label": "This is a label",
    "names": [],
    "nowrap": false,
    "number": 1,
    "xml:space": "preserve",
    "children": "\\nabla^2 f =\n\\frac{1}{r^2} \\frac{\\partial}{\\partial r}\n\\left( r^2 \\frac{\\partial f}{\\partial r} \\right) +\n\\frac{1}{r^2 \\sin \\theta} \\frac{\\partial f}{\\partial \\theta}\n\\left( \\sin \\theta \\, \\frac{\\partial f}{\\partial \\theta} \\right) +\n\\frac{1}{r^2 \\sin^2\\theta} \\frac{\\partial^2 f}{\\partial \\phi^2}"
  });
  "#
);
