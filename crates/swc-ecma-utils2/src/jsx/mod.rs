use swc_core::{
  common::util::take::Take as _,
  ecma::ast::{CallExpr, Expr, ObjectLit},
};

mod ast;
mod builder;
mod element;
mod runtime;

pub mod fixes;
pub mod tag;

use crate::collections::{MutableMapping as _, MutableSequence as _, Sequence as _};

pub use self::{
  builder::{DocumentBuilder, JSXDocument},
  element::{JSXElement, JSXElementMut},
  runtime::{JSXRuntime, JSXRuntimeDefault},
};
use self::{element::JSXCall, tag::JSXTagType};

#[inline(always)]
pub fn jsx<'a, R: JSXRuntime>(
  call: &'a CallExpr,
) -> Option<&'a (impl JSXElement<R, Component = Expr, Props = ObjectLit> + 'a)> {
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

#[inline(always)]
pub fn del_first_of_type<R: JSXRuntime>(call: &mut CallExpr, test: JSXTagType) -> Option<CallExpr> {
  let props = jsx_mut::<R>(call)?.get_props_mut()?;
  let children = props.get_item_mut("children")?;
  match children {
    Expr::Call(child) => {
      let tag = jsx_mut::<R>(child)?.get_tag()?;
      if tag.tag_type() == test {
        Some(child.take())
      } else {
        None
      }
    }
    Expr::Array(children) => {
      let found = children.iter().position(|child| {
        let Some(child) = child.as_call() else {
          return false;
        };
        let Some(child) = jsx::<R>(child) else {
          return false;
        };
        let Some(tag) = child.get_tag() else {
          return false;
        };
        tag.tag_type() == test
      });
      let Some(idx) = found else {
        return None;
      };
      children
        .get_item_mut(idx)
        .and_then(|child| child.take().call())
    }
    _ => None,
  }
}

#[macro_export]
macro_rules! jsx_tag {
  (<>?) => {
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
  ($tag:ident) => {
    $crate::jsx::tag::JSXTag::component(stringify!($tag).into())
  };
}

#[macro_export]
macro_rules! unpack_jsx {
  (
    [ $rtype:ident, $runtime:ty, $call:ident ],
    $(
      [ $variant:ident $($tag_unpack:tt)+ ] =
      [ $($tag_test:tt)+ ],
    )*
  ) => {{
      use $crate::collections::MutableMapping as _;
      use $crate::jsx::JSXElement;

      fn unpack<R: $crate::jsx::JSXRuntime>(call: &mut swc_core::ecma::ast::CallExpr) -> Option<$rtype> {
        match $crate::jsx::jsx::<R>(call)?.get_tag()?.tag_type() {
          $(
            $crate::unpack_jsx_pat!($($tag_test)+) => {
              $crate::unpack_jsx_test!(call, R, $($tag_test)+);
              $crate::unpack_jsx_eval!(call, $rtype, $variant, $($tag_unpack)+)
            },
          )*
          _ => None,
        }
      }

      unpack::<$runtime>($call)
  }};
}

#[macro_export]
macro_rules! unpack_jsx_pat {
  ($tag_type:pat) => {
    $tag_type
  };
  ($tag_type:pat, $($rest:tt)+) => {
    $tag_type
  };
}

#[macro_export]
macro_rules! unpack_jsx_test {
  ( $call:ident, $runtime:ty, $tag_type:pat ) => {};

  ( $call:ident, $runtime:ty, $tag_type:pat, $prop:ident = $value:literal $($rest:tt)* ) => {{
    use swc_core::common::EqIgnoreSpan;

    let prop = $crate::jsx::jsx::<$runtime>($call)?
      .get_props()?
      .get_item(stringify!($prop))?
      .as_lit()?;

    let test: swc_core::ecma::ast::Lit = $value.into();

    if !prop.eq_ignore_span(&test) {
      return None;
    }

    $crate::unpack_jsx_test!($call, $runtime, $tag_type $($rest)*);
  }};

  ( $call:ident, $runtime:ty, $tag_type:pat, has($child:pat), $($rest:tt)* ) => {{
    use $crate::collections::Mapping;

    let children = $crate::jsx::jsx::<$runtime>($call)?
      .get_props()?
      .get_item("children")?;

    if !children.values().any(|child| {
      child.as_call().is_some_and(|child| {
        matches!($crate::jsx::jsx::<$runtime>(child).get_tag().tag_type(), Some($child))
      })
    }) {
      return None;
    }

    $crate::unpack_jsx_test!($call, $runtime, $tag_type $($rest)*);
  }};
}

#[macro_export]
macro_rules! unpack_jsx_eval {
  ( $call:ident, $rtype:ident, $variant:ident, , $props:ident as $typed:ty $( ,$binding:ident )* ) => {{
    let mut props = $call.del_item(2usize)?;
    $(
      let $binding = props.del_item(stringify!($binding))?;
    )*
    let $props: $typed = $crate::serde::unpack_expr(props).ok()?;
    Some($rtype::$variant { $props $(, $binding)* })
  }};
  ( $call:ident, $rtype:ident, $variant:ident, as $binding:ident ) => {{
    Some($rtype::$variant { $binding: $call.take() })
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
