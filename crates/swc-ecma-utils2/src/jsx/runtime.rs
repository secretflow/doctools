pub trait JSXRuntime {
  const JSX: &'static str;
  const JSXS: &'static str;
  const FRAGMENT: &'static str;
}

#[derive(Default)]
pub struct JSXSymbols;

impl JSXRuntime for JSXSymbols {
  const JSX: &'static str = "_jsx";
  const JSXS: &'static str = "_jsxs";
  const FRAGMENT: &'static str = "_Fragment";
}
