use swc_core::{
  common::{util::take::Take as _, Spanned},
  ecma::ast::{CallExpr, Expr, Ident, ObjectLit},
};

use crate::{
  collections::{Mapping, MutableMapping},
  span::with_span,
};

use super::{JSXRuntime, JSXTagDef, JSXTagType};

pub trait JSXElement {
  fn as_arg0<R: JSXRuntime>(&self) -> Option<&Expr>;
  fn as_jsx_type<R: JSXRuntime>(&self) -> Option<JSXTagType<'_>>;
  fn as_jsx_props<R: JSXRuntime>(&self) -> Option<&Expr>;
  fn new_jsx_element<R: JSXRuntime>(tag: impl JSXTagDef) -> Self;
  fn new_jsx_fragment<R: JSXRuntime>() -> Self;

  fn is_jsx<R: JSXRuntime>(&self) -> bool {
    self.as_jsx_type::<R>().is_some()
  }
}

pub trait JSXElementMut {
  fn as_arg0_mut<R: JSXRuntime>(&mut self) -> Option<&mut Expr>;
  fn as_jsx_props_mut<R: JSXRuntime>(&mut self) -> Option<&mut Expr>;
  fn set_jsx_factory<R: JSXRuntime>(&mut self, num_children: usize);
}

impl JSXElement for CallExpr {
  fn as_arg0<R: JSXRuntime>(&self) -> Option<&Expr> {
    let callee = &self.get_item(0usize)?.as_ident()?.sym;
    if callee == R::JSX || callee == R::JSXS {
      self.get_item(1usize)
    } else {
      None
    }
  }

  fn as_jsx_type<R: JSXRuntime>(&self) -> Option<JSXTagType<'_>> {
    JSXTagType::from_expr::<R>(self.as_arg0::<R>()?)
  }

  fn as_jsx_props<R: JSXRuntime>(&self) -> Option<&Expr> {
    self.as_arg0::<R>()?;
    self.get_item(2usize)
  }

  fn new_jsx_element<R: JSXRuntime>(tag: impl JSXTagDef) -> Self {
    let mut call = CallExpr::dummy();
    call.set_item(0usize, Ident::from(R::JSX).into());
    call.set_item(1usize, tag.to_expr::<R>());
    call.set_item(2usize, ObjectLit::dummy().into());
    call
  }

  fn new_jsx_fragment<R: JSXRuntime>() -> Self {
    Self::new_jsx_element::<R>(JSXTagType::Fragment)
  }
}

impl JSXElementMut for CallExpr {
  fn as_arg0_mut<R: JSXRuntime>(&mut self) -> Option<&mut Expr> {
    let callee = &self.get_item(0usize)?.as_ident()?.sym;
    if callee == R::JSX || callee == R::JSXS {
      self.get_item_mut(1usize)
    } else {
      None
    }
  }

  fn as_jsx_props_mut<R: JSXRuntime>(&mut self) -> Option<&mut Expr> {
    self.as_arg0_mut::<R>()?;
    self.get_item_mut(2usize)
  }

  fn set_jsx_factory<R: JSXRuntime>(&mut self, num_children: usize) {
    if !self.is_jsx::<R>() {
      return;
    }
    let Some(callee) = self.get_item_mut(0usize) else {
      return;
    };
    if num_children > 1 {
      *callee = with_span(Some(callee.span()))(Ident::from(R::JSXS).into());
    } else {
      *callee = with_span(Some(callee.span()))(Ident::from(R::JSX).into());
    }
  }
}

impl<T> JSXElement for &T
where
  T: JSXElement,
{
  fn as_arg0<R: JSXRuntime>(&self) -> Option<&Expr> {
    (**self).as_arg0::<R>()
  }

  fn as_jsx_type<R: JSXRuntime>(&self) -> Option<JSXTagType<'_>> {
    (**self).as_jsx_type::<R>()
  }

  fn as_jsx_props<R: JSXRuntime>(&self) -> Option<&Expr> {
    (**self).as_jsx_props::<R>()
  }

  fn new_jsx_element<R: JSXRuntime>(_: impl JSXTagDef) -> Self {
    unimplemented!("Cannot call `new_jsx_element` on a reference type")
  }

  fn new_jsx_fragment<R: JSXRuntime>() -> Self {
    unimplemented!("Cannot call `new_jsx_fragment` on a reference type")
  }
}

impl<T> JSXElementMut for &mut T
where
  T: JSXElementMut,
{
  fn as_arg0_mut<R: JSXRuntime>(&mut self) -> Option<&mut Expr> {
    (**self).as_arg0_mut::<R>()
  }

  fn as_jsx_props_mut<R: JSXRuntime>(&mut self) -> Option<&mut Expr> {
    (**self).as_jsx_props_mut::<R>()
  }

  fn set_jsx_factory<R: JSXRuntime>(&mut self, num_children: usize) {
    (**self).set_jsx_factory::<R>(num_children)
  }
}
