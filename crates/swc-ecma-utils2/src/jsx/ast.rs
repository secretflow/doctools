use swc_core::{
  common::util::take::Take as _,
  ecma::ast::{CallExpr, Expr, Ident, Lit, ObjectLit, Str},
};

use crate::collections::{DefaultContainer, Mapping, MutableMapping};
use crate::ecma::itertools::is_nullish;

use super::{tag::JSXTag, JSXRuntime};

pub trait JSXCall<R: JSXRuntime>: Mapping {
  const KEY_FACTORY: Self::Key;
  const KEY_TYPE: Self::Key;
  const KEY_PROPS: Self::Key;

  fn as_factory(value: &Self::Value) -> Option<&str>;
  fn as_type(value: &Self::Value) -> Option<&Expr>;
  fn as_props(value: &Self::Value) -> Option<&ObjectLit>;
  fn as_tag(value: &Self::Value) -> Option<JSXTag>;

  fn new() -> Self
  where
    Self: MutableMapping;
  fn new_factory(factory: &str) -> Self::Value;
  fn new_fragment(fragment: &str) -> Self::Value;
  fn new_props() -> Self::Value;

  fn is_jsx(&self) -> Option<()> {
    let callee = Self::as_factory(self.get_item(Self::KEY_FACTORY)?)?;
    if callee == R::JSX || callee == R::JSXS {
      let _ = Self::as_type(self.get_item(Self::KEY_TYPE)?)?;
      let _ = Self::as_props(self.get_item(Self::KEY_PROPS)?)?;
      Some(())
    } else {
      None
    }
  }
}

pub trait JSXCallMut<R: JSXRuntime>: JSXCall<R> + MutableMapping {
  fn as_type_mut(value: &mut Self::Value) -> Option<&mut Expr>;
  fn as_props_mut(value: &mut Self::Value) -> Option<&mut ObjectLit>;
}

pub trait JSXElement<R: JSXRuntime>: JSXCall<R> {
  fn get_tag(&self) -> Option<JSXTag> {
    Self::as_tag(self.get_item(Self::KEY_TYPE)?)
  }

  fn get_type(&self) -> Option<&Expr> {
    Self::as_type(self.get_item(Self::KEY_TYPE)?)
  }

  fn get_props(&self) -> Option<&ObjectLit> {
    Self::as_props(self.get_item(Self::KEY_PROPS)?)
  }

  fn create_element(component: Self::Value) -> Self
  where
    Self: Sized + MutableMapping,
  {
    let mut new = Self::new();
    new
      .set_item(Self::KEY_FACTORY, Self::new_factory(R::JSX))
      .set_item(Self::KEY_TYPE, component)
      .set_item(Self::KEY_PROPS, Self::new_props());
    new
  }

  fn create_fragment() -> Self
  where
    Self: Sized + MutableMapping,
  {
    let mut new = Self::new();
    new
      .set_item(Self::KEY_FACTORY, Self::new_factory(R::JSX))
      .set_item(Self::KEY_TYPE, Self::new_fragment(R::FRAGMENT))
      .set_item(Self::KEY_PROPS, Self::new_props());
    new
  }
}

pub trait JSXElementMut<R: JSXRuntime>: JSXCallMut<R> {
  fn get_type_mut(&mut self) -> Option<&mut Expr> {
    Self::as_type_mut(self.get_item_mut(Self::KEY_TYPE)?)
  }

  fn get_props_mut(&mut self) -> Option<&mut ObjectLit> {
    Self::as_props_mut(self.get_item_mut(Self::KEY_PROPS)?)
  }

  fn set_type(&mut self, value: Expr) -> Option<&mut Self> {
    let current = self.get_type_mut()?;
    *current = value;
    Some(self)
  }

  fn set_factory(&mut self, count: usize) -> &mut Self {
    if count > 1 {
      self.set_item(Self::KEY_FACTORY, Self::new_factory(R::JSXS));
    } else {
      self.set_item(Self::KEY_FACTORY, Self::new_factory(R::JSX));
    }
    self
  }
}

impl<T, R: JSXRuntime> JSXCall<R> for &T
where
  T: JSXCall<R>,
{
  const KEY_FACTORY: Self::Key = T::KEY_FACTORY;
  const KEY_TYPE: Self::Key = T::KEY_TYPE;
  const KEY_PROPS: Self::Key = T::KEY_PROPS;

  fn as_factory(value: &Self::Value) -> Option<&str> {
    T::as_factory(value)
  }
  fn as_type(value: &Self::Value) -> Option<&Expr> {
    T::as_type(value)
  }
  fn as_props(value: &Self::Value) -> Option<&ObjectLit> {
    T::as_props(value)
  }
  fn as_tag(value: &Self::Value) -> Option<JSXTag> {
    T::as_tag(value)
  }

  fn new() -> Self {
    unreachable!()
  }
  fn new_factory(factory: &str) -> Self::Value {
    T::new_factory(factory)
  }
  fn new_fragment(fragment: &str) -> Self::Value {
    T::new_fragment(fragment)
  }
  fn new_props() -> Self::Value {
    T::new_props()
  }
}

impl<T, R: JSXRuntime> JSXCall<R> for &mut T
where
  T: JSXCall<R> + MutableMapping,
{
  const KEY_FACTORY: Self::Key = T::KEY_FACTORY;
  const KEY_TYPE: Self::Key = T::KEY_TYPE;
  const KEY_PROPS: Self::Key = T::KEY_PROPS;

  fn as_factory(value: &Self::Value) -> Option<&str> {
    T::as_factory(value)
  }
  fn as_type(value: &Self::Value) -> Option<&Expr> {
    T::as_type(value)
  }
  fn as_props(value: &Self::Value) -> Option<&ObjectLit> {
    T::as_props(value)
  }
  fn as_tag(value: &Self::Value) -> Option<JSXTag> {
    T::as_tag(value)
  }

  fn new() -> Self {
    unreachable!()
  }
  fn new_factory(factory: &str) -> Self::Value {
    T::new_factory(factory)
  }
  fn new_fragment(fragment: &str) -> Self::Value {
    T::new_fragment(fragment)
  }
  fn new_props() -> Self::Value {
    T::new_props()
  }
}

impl<T, R: JSXRuntime> JSXCallMut<R> for &mut T
where
  T: JSXCallMut<R>,
{
  fn as_type_mut(value: &mut Self::Value) -> Option<&mut Expr> {
    T::as_type_mut(value)
  }
  fn as_props_mut(value: &mut Self::Value) -> Option<&mut ObjectLit> {
    T::as_props_mut(value)
  }
}

impl<T, R: JSXRuntime> JSXCall<R> for Option<T>
where
  T: JSXCall<R>,
{
  const KEY_FACTORY: Self::Key = T::KEY_FACTORY;
  const KEY_TYPE: Self::Key = T::KEY_TYPE;
  const KEY_PROPS: Self::Key = T::KEY_PROPS;

  fn as_factory(value: &Self::Value) -> Option<&str> {
    T::as_factory(value)
  }
  fn as_type(value: &Self::Value) -> Option<&Expr> {
    T::as_type(value)
  }
  fn as_props(value: &Self::Value) -> Option<&ObjectLit> {
    T::as_props(value)
  }
  fn as_tag(value: &Self::Value) -> Option<JSXTag> {
    T::as_tag(value)
  }

  fn new() -> Self {
    unreachable!()
  }
  fn new_factory(factory: &str) -> Self::Value {
    T::new_factory(factory)
  }
  fn new_fragment(fragment: &str) -> Self::Value {
    T::new_fragment(fragment)
  }
  fn new_props() -> Self::Value {
    T::new_props()
  }
}

impl<T, R: JSXRuntime> JSXCallMut<R> for Option<T>
where
  T: JSXCallMut<R> + DefaultContainer,
{
  fn as_type_mut(value: &mut Self::Value) -> Option<&mut Expr> {
    T::as_type_mut(value)
  }
  fn as_props_mut(value: &mut Self::Value) -> Option<&mut ObjectLit> {
    T::as_props_mut(value)
  }
}

impl<T, R: JSXRuntime> JSXElement<R> for T where T: JSXCall<R> {}
impl<T, R: JSXRuntime> JSXElementMut<R> for T where T: JSXCallMut<R> {}

impl<R: JSXRuntime> JSXCall<R> for CallExpr {
  const KEY_FACTORY: Self::Key = 0;
  const KEY_TYPE: Self::Key = 1;
  const KEY_PROPS: Self::Key = 2;

  fn as_factory(value: &Self::Value) -> Option<&str> {
    let ident = value.as_ident()?;
    Some(&*ident.sym)
  }

  fn as_type(value: &Self::Value) -> Option<&Expr> {
    match value {
      Expr::Ident(_) => Some(value),
      Expr::Lit(Lit::Str(_)) => Some(value),
      _ => None,
    }
  }

  fn as_props(value: &Self::Value) -> Option<&ObjectLit> {
    value.as_object()
  }

  fn as_tag(value: &Self::Value) -> Option<JSXTag> {
    match value {
      Expr::Lit(Lit::Str(Str { value, .. })) => Some(JSXTag::intrinsic((&**value).into())),
      Expr::Ident(Ident { sym, .. }) => {
        if is_nullish(value) {
          None
        } else if sym == R::FRAGMENT {
          Some(JSXTag::fragment())
        } else {
          Some(JSXTag::component((&**sym).into()))
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
  fn as_type_mut(value: &mut Self::Value) -> Option<&mut Expr> {
    match value {
      Expr::Ident(_) => Some(value),
      Expr::Lit(Lit::Str(_)) => Some(value),
      _ => None,
    }
  }

  fn as_props_mut(value: &mut Self::Value) -> Option<&mut ObjectLit> {
    value.as_mut_object()
  }
}
