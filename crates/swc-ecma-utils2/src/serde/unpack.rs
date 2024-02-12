use std::marker::PhantomData;

use serde::de::{Error, IntoDeserializer};
use swc_core::{
  common::util::take::Take,
  ecma::ast::{ArrayLit, BigInt, Bool, Expr, Ident, JSXText, Lit, NewExpr, Number, Str},
};

use crate::{
  collections::{MutableMapping, MutableSequence},
  serde::passthru::{to_serde_data, visit_serde_data},
};

#[derive(Debug)]
struct UnpackExpr<'ast> {
  expr: Option<Expr>,
  expr_lifetime: PhantomData<&'ast Expr>,
}

impl<'ast> UnpackExpr<'ast> {
  fn from_expr(value: Expr) -> Self {
    Self {
      expr: Some(value),
      expr_lifetime: PhantomData,
    }
  }

  fn take(&mut self) -> Result<Expr, UnpackError> {
    self
      .expr
      .take()
      .ok_or(UnpackError::custom("unexpected None"))
  }

  fn number(value: Expr) -> Result<f64, UnpackError> {
    match value {
      Expr::Lit(Lit::Num(Number { value, .. })) => Ok(value),
      _ => Err(UnpackError::invalid_type(
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
      let number = self.take().and_then(UnpackExpr::number)?;
      if number < <$t>::MIN as f64 || number > <$t>::MAX as f64 {
        Err(UnpackError::invalid_value(
          serde::de::Unexpected::Float(number),
          &stringify!($t),
        ))
      } else if number.fract() != 0.0 {
        Err(UnpackError::invalid_value(
          serde::de::Unexpected::Float(number),
          &stringify!($t),
        ))
      } else {
        visitor.$visit(number as $t)
      }
    }
  };
}

impl<'de: 'ast, 'ast> serde::de::Deserializer<'de> for &'ast mut UnpackExpr<'de> {
  type Error = UnpackError;

  fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: serde::de::Visitor<'de>,
  {
    match self.take()? {
      Expr::Lit(Lit::Null(_)) => visitor.visit_unit(),
      Expr::Lit(Lit::Bool(Bool { value, .. })) => visitor.visit_bool(value),
      Expr::Lit(Lit::Num(Number { value, .. })) => visitor.visit_f64(value),
      Expr::Lit(Lit::Str(Str { value, .. })) => visitor.visit_str(&*value),

      Expr::Lit(Lit::BigInt(BigInt { value, .. })) => {
        visit_serde_data(visitor, to_serde_data(&value))
          .map_err(<UnpackError as serde::de::Error>::custom)
      }
      Expr::Lit(Lit::JSXText(JSXText { value, .. })) => visitor.visit_str(&*value),

      Expr::Array(container) => {
        let de = UnpackSequence::new(self, container);
        visitor.visit_seq(de)
      }

      Expr::Object(container) => {
        let de = UnpackMapping::new(self, container);
        visitor.visit_map(de)
      }

      value => Ok(
        visit_serde_data(visitor, to_serde_data(&value))
          .map_err(<UnpackError as serde::de::Error>::custom)?,
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
        self.expr = Some(value);
        visitor.visit_some(self)
      }
    }
  }

  serde::forward_to_deserialize_any! {
    bool
    char str string
    f64
    unit unit_struct
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
    let number = self.take().and_then(UnpackExpr::number)?;
    if number < f32::MIN as f64 || number > f32::MAX as f64 {
      Err(UnpackError::invalid_value(
        serde::de::Unexpected::Float(number),
        &"f32",
      ))
    } else {
      visitor.visit_f32(number as f32)
    }
  }

  fn deserialize_enum<V>(
    self,
    _name: &'static str,
    _variants: &'static [&'static str],
    visitor: V,
  ) -> Result<V::Value, Self::Error>
  where
    V: serde::de::Visitor<'de>,
  {
    match self.take()? {
      Expr::Lit(Lit::Str(Str { value, .. })) => visitor.visit_enum((&*value).into_deserializer()),
      expr => {
        self.expr = Some(expr);
        visitor.visit_enum(UnpackEnum::new(self))
      }
    }
  }

  fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: serde::de::Visitor<'de>,
  {
    let exprs = match self.take()? {
      Expr::New(NewExpr { callee, args, .. }) => {
        let arr = match *callee {
          Expr::Ident(Ident { sym, .. }) if sym == "Uint8Array" => match args {
            None => None,
            Some(mut args) => args.first_mut().and_then(|arg| match arg.spread {
              None => match (*arg.expr).take() {
                Expr::Array(ArrayLit { elems, .. }) => Some(elems),
                _ => None,
              },
              Some(_) => None,
            }),
          },
          _ => None,
        };
        let arr = arr.ok_or_else(|| {
          Self::Error::invalid_value(
            serde::de::Unexpected::Other("arbitrary expression"),
            &"a Uint8Array constructor",
          )
        })?;
        arr
      }
      Expr::Array(ArrayLit { elems, .. }) => elems,
      expr => {
        self.expr = Some(expr);
        return self.deserialize_any(visitor);
      }
    };

    let mut array = vec![];

    for item in exprs {
      let Some(item) = item else { continue };
      let item = match item.spread {
        None => Some(*item.expr),
        Some(_) => None,
      }
      .ok_or_else(|| {
        Self::Error::invalid_value(
          serde::de::Unexpected::Other("spread argument"),
          &"a non-spread array item",
        )
      })?;
      array.push(item);
    }

    let mut byte_array: Vec<u8> = vec![];

    for item in array {
      let mut de = UnpackExpr::from_expr(item);
      use serde::Deserialize;
      byte_array.push(u8::deserialize(&mut de)?);
    }

    visitor.visit_byte_buf(byte_array)
  }

  fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: serde::de::Visitor<'de>,
  {
    self.deserialize_bytes(visitor)
  }
}

struct UnpackSequence<'ast, 'de: 'ast, T>
where
  T: MutableSequence<Value = Expr> + 'ast,
{
  de: &'ast mut UnpackExpr<'de>,
  container: T,
  iter: Box<dyn Iterator<Item = usize> + 'ast>,
}

impl<'ast, 'de, T> UnpackSequence<'ast, 'de, T>
where
  T: MutableSequence<Value = Expr> + 'ast,
{
  fn new(de: &'ast mut UnpackExpr<'de>, container: T) -> Self {
    let iter = Box::new((0..container.len()).collect::<Vec<_>>().into_iter());
    Self {
      de,
      container,
      iter,
    }
  }
}

impl<'ast, 'de, T> serde::de::SeqAccess<'de> for UnpackSequence<'ast, 'de, T>
where
  T: MutableSequence<Value = Expr> + 'ast,
{
  type Error = UnpackError;

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

    self.de.expr = value;

    seed.deserialize(&mut *self.de).map(Some)
  }
}

struct UnpackMapping<'ast, 'de: 'ast, T>
where
  T: MutableMapping<Value = Expr> + 'ast,
{
  de: &'ast mut UnpackExpr<'de>,
  container: T,
  iter: Box<dyn Iterator<Item = T::Key> + 'ast>,
  value: Option<T::Value>,
}

impl<'ast, 'de, T> UnpackMapping<'ast, 'de, T>
where
  T: MutableMapping<Value = Expr> + 'ast,
{
  fn new(de: &'ast mut UnpackExpr<'de>, container: T) -> Self {
    let iter = Box::new(container.keys().collect::<Vec<_>>().into_iter());
    Self {
      de,
      container,
      iter,
      value: None,
    }
  }
}

impl<'ast, 'de, T> serde::de::MapAccess<'de> for UnpackMapping<'ast, 'de, T>
where
  T: MutableMapping<Value = Expr> + 'ast,
  T::Key: Clone + TryInto<Expr>,
{
  type Error = UnpackError;

  fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
  where
    K: serde::de::DeserializeSeed<'de>,
  {
    let Some(key) = self.iter.next() else {
      return Ok(None);
    };

    self.value = self.container.del_item(key.clone());

    self.de.expr = key.try_into().ok();
    seed.deserialize(&mut *self.de).map(Some)
  }

  fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
  where
    V: serde::de::DeserializeSeed<'de>,
  {
    self.de.expr = self.value.take();
    seed.deserialize(&mut *self.de)
  }
}

#[derive(Debug)]
struct UnpackEnum<'de: 'a, 'a> {
  de: &'a mut UnpackExpr<'de>,
}

impl<'de, 'a> UnpackEnum<'de, 'a> {
  fn new(de: &'a mut UnpackExpr<'de>) -> Self {
    Self { de }
  }
}

impl<'de, 'a> serde::de::EnumAccess<'de> for UnpackEnum<'de, 'a> {
  type Error = UnpackError;
  type Variant = Self;

  fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
  where
    V: serde::de::DeserializeSeed<'de>,
  {
    let (key, value) = self.de.take()?.pop_item().ok_or_else(|| {
      Self::Error::invalid_value(
        serde::de::Unexpected::Other("empty object"),
        &"an object with at least one item",
      )
    })?;
    self.de.expr = Some(key.into());
    let key = seed.deserialize(&mut *self.de)?;
    self.de.expr = Some(value);
    Ok((key, self))
  }
}

impl<'de, 'a> serde::de::VariantAccess<'de> for UnpackEnum<'de, 'a> {
  type Error = UnpackError;

  fn unit_variant(self) -> Result<(), Self::Error> {
    Err(Self::Error::invalid_type(
      serde::de::Unexpected::Unit,
      &"unit variant",
    ))
  }

  fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
  where
    T: serde::de::DeserializeSeed<'de>,
  {
    seed.deserialize(&mut *self.de)
  }

  fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: serde::de::Visitor<'de>,
  {
    serde::de::Deserializer::deserialize_seq(&mut *self.de, visitor)
  }

  fn struct_variant<V>(
    self,
    _fields: &'static [&'static str],
    visitor: V,
  ) -> Result<V::Value, Self::Error>
  where
    V: serde::de::Visitor<'de>,
  {
    serde::de::Deserializer::deserialize_map(&mut *self.de, visitor)
  }
}

#[derive(Debug, thiserror::Error)]
#[error("failed to deserialize AST: {0}")]
pub struct UnpackError(String);

impl serde::de::Error for UnpackError {
  fn custom<T: std::fmt::Display>(msg: T) -> Self {
    UnpackError(format!("{}", msg))
  }
}

pub fn unpack_expr<'ast, T>(expr: Expr) -> Result<T, UnpackError>
where
  T: serde::de::Deserialize<'ast>,
{
  let mut deserializer = UnpackExpr::from_expr(expr);
  Ok(T::deserialize(&mut deserializer)?)
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;

  use serde::{Deserialize, Serialize};
  use swc_core::ecma::parser::parse_file_as_expr;
  use swc_ecma_testing2::parse_one;

  use super::unpack_expr;

  #[derive(Debug, Serialize, Deserialize)]
  enum HTTPMethod {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
    HEAD,
    OPTIONS,
    TRACE,
  }

  #[derive(Debug, Serialize, Deserialize)]
  enum RequestMode {
    #[serde(rename = "cors")]
    CORS,
    #[serde(rename = "no-cors")]
    NoCORS,
    #[serde(rename = "same-origin")]
    SameOrigin,
  }

  impl Default for RequestMode {
    fn default() -> Self {
      Self::CORS
    }
  }

  #[derive(Debug, Serialize, Deserialize)]
  struct Request {
    method: HTTPMethod,
    url: String,
    #[serde(default)]
    mode: Option<RequestMode>,
    headers: HashMap<String, String>,
    #[serde(with = "serde_bytes")]
    body: Option<Vec<u8>>,
  }

  #[test]
  fn test_unpack() {
    let src = r#"
    {
      method: "POST",
      url: "https://example.org/api",
      mode: "same-origin",
      headers: {
        "accept-charset": "utf-8",
        "cache-control": "no-store",
        "content-type": "text/plain; charset=utf-8",
      },
      body: new Uint8Array([
        78, 101, 118, 101, 114, 32, 103, 111, 110, 110, 97, 32, 103, 105, 118, 101,
        32, 121, 111, 117, 32, 117, 112,
      ]),
    }
    "#;
    let expr = parse_one(src, None, parse_file_as_expr).unwrap();

    let request: Request = unpack_expr(*expr).unwrap();

    let body = request.body.unwrap();
    let body = String::from_utf8_lossy(&body);

    assert_eq!(body, "Never gonna give you up");
  }
}
