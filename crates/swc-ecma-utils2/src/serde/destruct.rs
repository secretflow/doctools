use std::marker::PhantomData;

use serde::de::Error;
use swc_core::{
  common::util::take::Take,
  ecma::ast::{BigInt, Bool, Expr, Ident, JSXText, Lit, Number, Str},
};

use crate::{
  collections::{MutableMapping, MutableSequence},
  serde::passthru::{to_serde_data, visit_serde_data},
};

struct DestructExpr<'ast> {
  value: Option<Expr>,
  marker: PhantomData<&'ast Expr>,
}

impl<'ast> DestructExpr<'ast> {
  fn from_expr(value: Expr) -> Self {
    Self {
      value: Some(value),
      marker: PhantomData,
    }
  }

  fn take(&mut self) -> Result<Expr, DestructError> {
    self
      .value
      .take()
      .ok_or(DestructError::custom("unexpected None"))
  }

  fn number(value: Expr) -> Result<f64, DestructError> {
    match value {
      Expr::Lit(Lit::Num(Number { value, .. })) => Ok(value),
      _ => Err(DestructError::invalid_type(
        serde::de::Unexpected::Other("arbitrary expression"),
        &"a numeric literal",
      )),
    }
  }
}

macro_rules! integer {
  ($t:ty, $de:ident, $visit:ident) => {
    fn $de<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
      V: serde::de::Visitor<'de>,
    {
      let number = self.take().and_then(DestructExpr::number)?;
      if number < <$t>::MIN as f64 || number > <$t>::MAX as f64 {
        Err(DestructError::invalid_value(
          serde::de::Unexpected::Float(number),
          &stringify!($t),
        ))
      } else if number.fract() != 0.0 {
        Err(DestructError::invalid_value(
          serde::de::Unexpected::Float(number),
          &stringify!($t),
        ))
      } else {
        visitor.$visit(number as $t)
      }
    }
  };
}

impl<'de: 'ast, 'ast> serde::de::Deserializer<'de> for &'ast mut DestructExpr<'de> {
  type Error = DestructError;

  fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: serde::de::Visitor<'de>,
  {
    match self.take() {
      Err(e) => Err(e),

      Ok(Expr::Lit(Lit::Null(_))) => visitor.visit_unit(),
      Ok(Expr::Lit(Lit::Bool(Bool { value, .. }))) => visitor.visit_bool(value),
      Ok(Expr::Lit(Lit::Num(Number { value, .. }))) => visitor.visit_f64(value),
      Ok(Expr::Lit(Lit::Str(Str { value, .. }))) => visitor.visit_str(&*value),

      Ok(Expr::Lit(Lit::BigInt(BigInt { value, .. }))) => Ok(
        visit_serde_data(visitor, to_serde_data(&value))
          .map_err(<DestructError as serde::de::Error>::custom)?,
      ),
      Ok(Expr::Lit(Lit::JSXText(JSXText { value, .. }))) => visitor.visit_str(&*value),

      Ok(Expr::Array(container)) => {
        let de = DestructSequence::new(self, container);
        visitor.visit_seq(de)
      }

      Ok(Expr::Object(container)) => {
        let de = DestructMapping::new(self, container);
        visitor.visit_map(de)
      }

      Ok(value) => Ok(
        visit_serde_data(visitor, to_serde_data(&value))
          .map_err(<DestructError as serde::de::Error>::custom)?,
      ),
    }
  }

  fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: serde::de::Visitor<'de>,
  {
    let value = self.take()?;
    match value {
      Expr::Lit(Lit::Null(_)) => visitor.visit_none(),
      Expr::Ident(Ident { ref sym, .. }) if sym == "undefined" => visitor.visit_none(),
      value => {
        self.value = Some(value);
        visitor.visit_some(self)
      }
    }
  }

  serde::forward_to_deserialize_any! {
    bool
    byte_buf bytes char str string
    f64
    unit unit_struct
    enum
    map struct newtype_struct
    seq tuple tuple_struct
    identifier ignored_any
  }

  integer!(i8, deserialize_i8, visit_i8);
  integer!(i16, deserialize_i16, visit_i16);
  integer!(i32, deserialize_i32, visit_i32);
  integer!(i64, deserialize_i64, visit_i64);
  integer!(u8, deserialize_u8, visit_u8);
  integer!(u16, deserialize_u16, visit_u16);
  integer!(u32, deserialize_u32, visit_u32);
  integer!(u64, deserialize_u64, visit_u64);

  fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: serde::de::Visitor<'de>,
  {
    let number = self.take().and_then(DestructExpr::number)?;
    if number < f32::MIN as f64 || number > f32::MAX as f64 {
      Err(DestructError::invalid_value(
        serde::de::Unexpected::Float(number),
        &"f32",
      ))
    } else {
      visitor.visit_f32(number as f32)
    }
  }
}

struct DestructSequence<'ast, 'de: 'ast, T>
where
  T: MutableSequence<Value = Expr> + 'ast,
{
  de: &'ast mut DestructExpr<'de>,
  container: T,
  iter: Box<dyn Iterator<Item = usize> + 'ast>,
}

impl<'ast, 'de, T> DestructSequence<'ast, 'de, T>
where
  T: MutableSequence<Value = Expr> + 'ast,
{
  fn new(de: &'ast mut DestructExpr<'de>, container: T) -> Self {
    let iter = Box::new((0..container.len()).collect::<Vec<_>>().into_iter());
    Self {
      de,
      container,
      iter,
    }
  }
}

impl<'ast, 'de, T> serde::de::SeqAccess<'de> for DestructSequence<'ast, 'de, T>
where
  T: MutableSequence<Value = Expr> + 'ast,
{
  type Error = DestructError;

  fn next_element_seed<U>(&mut self, seed: U) -> Result<Option<U::Value>, Self::Error>
  where
    U: serde::de::DeserializeSeed<'de>,
  {
    let idx = self.iter.next();

    let Some(idx) = idx else { return Ok(None) };

    let value = self
      .container
      .get_item_mut(idx)
      .and_then(|item| Some(item.take()));

    self.de.value = value;

    seed.deserialize(&mut *self.de).map(Some)
  }
}

struct DestructMapping<'ast, 'de: 'ast, T>
where
  T: MutableMapping<Value = Expr> + 'ast,
{
  de: &'ast mut DestructExpr<'de>,
  container: T,
  iter: Box<dyn Iterator<Item = T::Key> + 'ast>,
  key: Option<T::Key>,
}

impl<'ast, 'de, T> DestructMapping<'ast, 'de, T>
where
  T: MutableMapping<Value = Expr> + 'ast,
{
  fn new(de: &'ast mut DestructExpr<'de>, container: T) -> Self {
    let iter = Box::new(container.keys().collect::<Vec<_>>().into_iter());
    Self {
      de,
      container,
      iter,
      key: None,
    }
  }
}

impl<'ast, 'de, T> serde::de::MapAccess<'de> for DestructMapping<'ast, 'de, T>
where
  T: MutableMapping<Value = Expr> + 'ast,
  T::Key: TryInto<Expr>,
{
  type Error = DestructError;

  fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
  where
    K: serde::de::DeserializeSeed<'de>,
  {
    self.key = self.iter.next();

    let key = self.key.take();

    let Some(key) = key else { return Ok(None) };

    self.de.value = key.try_into().ok();

    seed.deserialize(&mut *self.de).map(Some)
  }

  fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
  where
    V: serde::de::DeserializeSeed<'de>,
  {
    let value = self
      .container
      .get_item_mut(self.key.take().expect("current key should be set"))
      .and_then(|item| Some(item.take()));

    self.de.value = value;

    seed.deserialize(&mut *self.de)
  }
}

#[derive(Debug, thiserror::Error)]
#[error("failed to deserialize AST: {0}")]
pub struct DestructError(String);

impl serde::de::Error for DestructError {
  fn custom<T: std::fmt::Display>(msg: T) -> Self {
    DestructError(format!("{}", msg))
  }
}

pub fn destruct_expr<'ast, T>(expr: Expr) -> Result<T, DestructError>
where
  T: serde::de::Deserialize<'ast>,
{
  let mut deserializer = DestructExpr::from_expr(expr);
  Ok(T::deserialize(&mut deserializer)?)
}
