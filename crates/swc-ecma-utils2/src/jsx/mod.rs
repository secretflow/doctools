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
macro_rules! unpack_jsx {
  (
    [ $rtype:ident, $runtime:ty, $call:ident ],
    $(
      $tag_type:pat =
      [ $variant:ident, $props:ident as $typed:ty $( ,$binding:ident )* ],
    )* ) => {{
      use $crate::collections::MutableMapping as _;
      use $crate::jsx::JSXElement;

      fn unpack<R: $crate::jsx::JSXRuntime>(call: &mut swc_core::ecma::ast::CallExpr) -> Option<$rtype> {
        match $crate::jsx::jsx::<R>(call)?.get_tag()?.tag_type() {
          $($tag_type => {
            let mut props = call.del_item(2usize)?;
            $(
              let $binding = props.del_item(stringify!($binding))?;
            )*
            let $props: $typed = $crate::serde::unpack_expr(props).ok()?;
            {
              Some($rtype::$variant { $props $(, $binding)* })
            }
          })*
          _ => None,
        }
      }

      unpack::<$runtime>($call)
  }};
}

#[macro_export]
macro_rules! JSX {
  ([ $($create:tt)+ ] $(, [ $($assign:tt)+ ])*) => {{
    use swc_core::ecma::ast::ObjectLit;
    use $crate::collections::MutableMapping as _;
    let mut call = $crate::_jsx_create!($($create)+);
    let mut __props__ = ObjectLit::dummy();
    $(
      $crate::object_assign!(__props__, $($assign)+);
    )*
    call.set_item(2usize, __props__.into());
    call
  }};
  ([ $($create:tt)+ ], $props:expr, $([ $($assign:tt)+ ]),*) => {{
    use swc_core::ecma::ast::CallExpr;
    use $crate::collections::MutableMapping as _;

    let repack = || -> Result<CallExpr, $crate::serde::RepackError> {
      let mut call = $crate::_jsx_create!($($create)+);
      let mut __props__ = $crate::serde::repack_expr(&$props)?;
      $(
        $crate::object_assign!(__props__, $($assign)+);
      )*
      call.set_item(2usize, __props__.into());
      Ok(call.into())
    };

    repack()
  }};
}

#[macro_export]
macro_rules! _jsx_create {
  (Fragment, $runtime:ty, $span:expr) => {{
    let mut elem = $crate::jsx::create_fragment::<$runtime>();
    elem.span = $span;
    elem
  }};
  (($name:expr), $runtime:ty, $span:expr) => {{
    let mut elem = $crate::jsx::create_element::<$runtime>($name.into());
    elem.span = $span;
    elem
  }};
  ($name:ident, $runtime:ty, $span:expr) => {{
    let name = swc_core::ecma::ast::Ident::from(stringify!($name));
    let mut elem = $crate::jsx::create_element::<$runtime>(name.into());
    elem.span = $span;
    elem
  }};
  ($name:literal, $runtime:ty, $span:expr) => {{
    let name = swc_core::ecma::ast::Str::from($name);
    let mut elem = $crate::jsx::create_element::<$runtime>(name.into());
    elem.span = $span;
    elem
  }};
}
