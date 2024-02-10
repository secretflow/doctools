use swc_core::{
  common::util::take::Take as _,
  ecma::ast::{CallExpr, Expr, Ident, Lit, ObjectLit, Str},
};

use crate::ecma::itertools::is_nullish;

use super::{
  element::{JSXCall, JSXCallMut},
  tag::JSXTag,
  JSXRuntime,
};

impl<R: JSXRuntime> JSXCall<R> for CallExpr {
  const KEY_FACTORY: Self::Key = 0;
  const KEY_TYPE: Self::Key = 1;
  const KEY_PROPS: Self::Key = 2;

  type Component = Expr;
  type Props = ObjectLit;

  fn as_factory(value: &Self::Value) -> Option<&str> {
    let ident = value.as_ident()?;
    Some(&*ident.sym)
  }

  fn as_type(value: &Self::Value) -> Option<&Self::Component> {
    match value {
      Expr::Ident(_) => Some(value),
      Expr::Lit(Lit::Str(_)) => Some(value),
      _ => None,
    }
  }

  fn as_props(value: &Self::Value) -> Option<&Self::Props> {
    value.as_object()
  }

  fn as_tag(value: &Self::Value) -> Option<JSXTag> {
    match value {
      Expr::Lit(Lit::Str(Str { value, .. })) => Some(JSXTag::Intrinsic((&**value).into())),
      Expr::Ident(Ident { sym, .. }) => {
        if is_nullish(value) {
          None
        } else if sym == R::FRAGMENT {
          Some(JSXTag::Fragment)
        } else {
          Some(JSXTag::Component((&**sym).into()))
        }
      }
      _ => None,
    }
  }

  fn new() -> Self {
    CallExpr::dummy()
  }

  fn new_factory(factory: &str) -> Self::Value {
    Ident::from(factory).into()
  }

  fn new_fragment(fragment: &str) -> Self::Value {
    Ident::from(fragment).into()
  }

  fn new_props() -> Self::Value {
    ObjectLit::dummy().into()
  }
}

impl<R: JSXRuntime> JSXCallMut<R> for CallExpr {
  fn as_type_mut(value: &mut Self::Value) -> Option<&mut Self::Component> {
    match value {
      Expr::Ident(_) => Some(value),
      Expr::Lit(Lit::Str(_)) => Some(value),
      _ => None,
    }
  }

  fn as_props_mut(value: &mut Self::Value) -> Option<&mut Self::Props> {
    value.as_mut_object()
  }
}
