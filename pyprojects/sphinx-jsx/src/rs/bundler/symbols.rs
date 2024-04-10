use swc_ecma_transform_i18n::I18nSymbols;
use swc_ecma_utils2::jsx::JSXRuntime;

#[derive(Default)]
pub struct Symbols;

impl JSXRuntime for Symbols {
  const JSX: &'static str = "_jsx";
  const JSXS: &'static str = "_jsxs";
  const FRAGMENT: &'static str = "_Fragment";
}

impl I18nSymbols for Symbols {
  const GETTEXT: &'static str = "_";
  const TRANS: &'static str = "Trans";
}
