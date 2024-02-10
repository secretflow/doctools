use crate::collections::{DefaultContainer, Mapping, MutableMapping};

use super::{runtime::JSXRuntime, tag::JSXTag};

pub trait JSXCall<R: JSXRuntime>: Mapping {
  const KEY_FACTORY: Self::Key;
  const KEY_TYPE: Self::Key;
  const KEY_PROPS: Self::Key;

  type Component;
  type Props;

  fn as_factory(value: &Self::Value) -> Option<&str>;
  fn as_type(value: &Self::Value) -> Option<&Self::Component>;
  fn as_props(value: &Self::Value) -> Option<&Self::Props>;
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
  fn as_type_mut(value: &mut Self::Value) -> Option<&mut Self::Component>;
  fn as_props_mut(value: &mut Self::Value) -> Option<&mut Self::Props>;
}

pub trait JSXElement<R: JSXRuntime>: JSXCall<R> {
  fn get_tag(&self) -> Option<JSXTag> {
    Self::as_tag(self.get_item(Self::KEY_TYPE)?)
  }

  fn get_type(&self) -> Option<&Self::Component> {
    Self::as_type(self.get_item(Self::KEY_TYPE)?)
  }

  fn get_props(&self) -> Option<&Self::Props> {
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
  fn get_type_mut(&mut self) -> Option<&mut Self::Component> {
    Self::as_type_mut(self.get_item_mut(Self::KEY_TYPE)?)
  }

  fn get_props_mut(&mut self) -> Option<&mut Self::Props> {
    Self::as_props_mut(self.get_item_mut(Self::KEY_PROPS)?)
  }

  fn set_type(&mut self, value: Self::Component) -> Option<&mut Self> {
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

  type Component = T::Component;
  type Props = T::Props;

  fn as_factory(value: &Self::Value) -> Option<&str> {
    T::as_factory(value)
  }
  fn as_type(value: &Self::Value) -> Option<&Self::Component> {
    T::as_type(value)
  }
  fn as_props(value: &Self::Value) -> Option<&Self::Props> {
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

  type Component = T::Component;
  type Props = T::Props;

  fn as_factory(value: &Self::Value) -> Option<&str> {
    T::as_factory(value)
  }
  fn as_type(value: &Self::Value) -> Option<&Self::Component> {
    T::as_type(value)
  }
  fn as_props(value: &Self::Value) -> Option<&Self::Props> {
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
  fn as_type_mut(value: &mut Self::Value) -> Option<&mut Self::Component> {
    T::as_type_mut(value)
  }
  fn as_props_mut(value: &mut Self::Value) -> Option<&mut Self::Props> {
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

  type Component = T::Component;
  type Props = T::Props;

  fn as_factory(value: &Self::Value) -> Option<&str> {
    T::as_factory(value)
  }
  fn as_type(value: &Self::Value) -> Option<&Self::Component> {
    T::as_type(value)
  }
  fn as_props(value: &Self::Value) -> Option<&Self::Props> {
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
  fn as_type_mut(value: &mut Self::Value) -> Option<&mut Self::Component> {
    T::as_type_mut(value)
  }
  fn as_props_mut(value: &mut Self::Value) -> Option<&mut Self::Props> {
    T::as_props_mut(value)
  }
}

impl<T, R: JSXRuntime> JSXElement<R> for T where T: JSXCall<R> {}
impl<T, R: JSXRuntime> JSXElementMut<R> for T where T: JSXCallMut<R> {}
