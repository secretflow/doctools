use swc_core::ecma::ast::{CallExpr, Expr, ObjectLit};

mod ast;
mod builder;
mod element;
mod runtime;

pub mod sanitize;
pub mod tag;

use self::element::JSXCall;
pub use self::{
  builder::{DocumentBuilder, JSXDocument},
  element::{JSXElement, JSXElementMut},
  runtime::{JSXRuntime, JSXRuntimeDefault},
};

#[inline(always)]
pub fn jsx<R: JSXRuntime>(
  call: &CallExpr,
) -> Option<&(impl JSXElement<R, Component = Expr, Props = ObjectLit> + '_)> {
  match <CallExpr as JSXCall<R>>::is_jsx(call) {
    Some(_) => Some(call),
    None => None,
  }
}

#[inline(always)]
pub fn jsx_mut<R: JSXRuntime>(
  call: &mut CallExpr,
) -> Option<&mut (impl JSXElementMut<R, Component = Expr, Props = ObjectLit> + '_)> {
  match <CallExpr as JSXCall<R>>::is_jsx(call) {
    Some(_) => Some(call),
    None => None,
  }
}

#[inline(always)]
pub fn create_element<R: JSXRuntime>(component: Expr) -> CallExpr {
  <CallExpr as JSXElement<R>>::create_element(component)
}

#[inline(always)]
pub fn create_fragment<R: JSXRuntime>() -> CallExpr {
  <CallExpr as JSXElement<R>>::create_fragment()
}

#[macro_export]
macro_rules! jsx_tag {
  (<>) => {
    $crate::jsx::tag::JSXTag::Fragment
  };
  ($tag:literal) => {
    $crate::jsx::tag::JSXTag::Intrinsic($tag.into())
  };
  (<$tag:ident>) => {
    $crate::jsx::tag::JSXTag::Ident(stringify!($tag).into())
  };
}

#[macro_export]
macro_rules! JSX {
  ([(), $runtime:ty], $props:expr) => {{
    use swc_ecma_utils2::collections::MutableMapping as _;
    let mut call = $crate::jsx::create_fragment::<$runtime>();
    call.set_item(2usize, $props.into());
    call
  }};
  ([($name:ident), $runtime:ty], $props:expr) => {{
    use swc_ecma_utils2::collections::MutableMapping as _;
    let name = swc_core::ecma::ast::Ident::from(stringify!($name));
    let mut call = $crate::jsx::create_element::<$runtime>(name.into());
    call.set_item(2usize, $props.into());
    call
  }};
  ([$name:expr, $runtime:ty], $props:expr) => {{
    use swc_ecma_utils2::collections::MutableMapping as _;
    let mut call = $crate::jsx::create_element::<$runtime>($name.into());
    call.set_item(2usize, $props.into());
    call
  }};
}
