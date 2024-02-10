pub trait I18nSymbols {
  const GETTEXT: &'static str;
  const TRANS: &'static str;
}

pub struct LinguiSymbols;

impl I18nSymbols for LinguiSymbols {
  const GETTEXT: &'static str = "t";
  const TRANS: &'static str = "Trans";
}
