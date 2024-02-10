pub mod itertools;
pub mod sanitize;

#[macro_export]
macro_rules! var {
  ($name:ident) => {
    swc_core::ecma::ast::Ident::from(stringify!($name))
  };
  ($name:expr) => {
    swc_core::ecma::ast::Ident::from($name)
  };
}

#[macro_export]
macro_rules! null {
  () => {
    swc_core::ecma::ast::Null::dummy()
  };
}

#[macro_export]
macro_rules! undefined {
  () => {
    $crate::var!(undefined)
  };
}
