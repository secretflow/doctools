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
  (()?) => {
    $crate::jsx::tag::JSXTagType::Fragment
  };
  ($tag:literal?) => {
    $crate::jsx::tag::JSXTagType::Intrinsic($tag)
  };
  ($tag:ident?) => {
    $crate::jsx::tag::JSXTagType::Component(stringify!($tag))
  };
  (<>) => {
    $crate::jsx::tag::JSXTag::fragment()
  };
  ($tag:literal) => {
    $crate::jsx::tag::JSXTag::intrinsic($tag.into())
  };
  (<$tag:ident>) => {
    $crate::jsx::tag::JSXTag::component(stringify!($tag).into())
  };
}

#[macro_export]
macro_rules! JSX {
  ([(), $runtime:ty], $props:expr) => {{
    use $crate::collections::MutableMapping as _;
    let mut call = $crate::jsx::create_fragment::<$runtime>();
    call.set_item(2usize, $props.into());
    call
  }};
  ([($name:expr), $runtime:ty], $props:expr) => {{
    use $crate::collections::MutableMapping as _;
    let mut call = $crate::jsx::create_element::<$runtime>($name.into());
    call.set_item(2usize, $props.into());
    call
  }};
  ([$name:ident, $runtime:ty], $props:expr) => {{
    use $crate::collections::MutableMapping as _;
    let name = swc_core::ecma::ast::Ident::from(stringify!($name));
    let mut call = $crate::jsx::create_element::<$runtime>(name.into());
    call.set_item(2usize, $props.into());
    call
  }};
  ([$name:literal, $runtime:ty], $props:expr) => {{
    use $crate::collections::MutableMapping as _;
    let name = swc_core::ecma::ast::Str::from($name);
    let mut call = $crate::jsx::create_element::<$runtime>(name.into());
    call.set_item(2usize, $props.into());
    call
  }};
}

#[macro_export]
macro_rules! unpack_jsx {
  (
    [ $call:ident, $rtype:ty, $runtime:ty ],
    $(
      $tag_type:pat =
      [ $result_variant:path, $deserialize:ty $( ,$binding:ident )* ],
    )* ) => {{
      use $crate::collections::MutableMapping as _;
      use $crate::jsx::JSXElement;

      fn unpack<R: $crate::jsx::JSXRuntime>(call: &mut swc_core::ecma::ast::CallExpr) -> Option<$rtype> {
        match $crate::jsx::jsx::<R>(call)?.get_tag()?.tag_type() {
          $($tag_type => {
            let props = call.del_item(2usize)?;
            $(
              let (_, $binding) = props.del_item(stringify!($binding))?;
            )*
            let rest: $deserialize = $crate::serde::destructure_expr(props).ok()?;
            Some($result_variant(rest, $($binding),*))
          })*
          _ => None,
        }
      }

      unpack::<$runtime>($call)
  }};
}
