use std::borrow::{Borrow, Cow};

use serde::{
  de::{Error, IntoDeserializer},
  Deserializer,
};
use swc_core::ecma::ast::{
  ArrayLit, BigInt, Bool, ComputedPropName, Expr, Ident, JSXText, KeyValueProp, Lit, NewExpr,
  Number, ObjectLit, Prop, PropName, PropOrSpread, Str,
};

use crate::{
  collections::{Mapping, Sequence},
  serde::{to_serde_data, visit_serde_data},
};

macro_rules! deserialize_integer {
  ($getter:ident, $parser:ident, $t:ty, $de:ident, $visit:ident) => {
    fn $de<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
      V: serde::de::Visitor<'de>,
    {
      visitor.$visit(bounded_integer($parser(self.$getter)?)?)
    }
  };
}

macro_rules! deserialize_float {
  ($getter:ident, $parser:ident, $t:ty, $de:ident, $visit:ident) => {
    fn $de<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
      V: serde::de::Visitor<'de>,
    {
      visitor.$visit(bounded_float($parser(self.$getter)?)?)
    }
  };
}

macro_rules! deserialize_char {
  ($getter:ident, $parse:ident, $error:path) => {
    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
      V: serde::de::Visitor<'de>,
    {
      let value = $parse(&self.$getter)?;
      if value.len() == 0 {
        Err($error(&self.$getter, "a non-empty string"))
      } else if value.len() > 1 {
        Err($error(&self.$getter, "a single-character string"))
      } else {
        visitor.visit_char(value.chars().next().unwrap())
      }
    }
  };
}

macro_rules! forward_to_lit {
  ($de:ident, $expect:literal) => {
    fn $de<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
      V: serde::de::Visitor<'de>,
    {
      match self.expr {
        Expr::Lit(lit) => UnpackLit { lit }.$de(visitor),
        _ => Err(UnpackError::incorrect_expr_type(&self.expr, $expect)),
      }
    }
  };
}

pub struct UnpackExpr<'ast> {
  expr: &'ast Expr,
}

impl<'de> serde::de::Deserializer<'de> for UnpackExpr<'de> {
  type Error = UnpackError;

  forward_to_lit!(deserialize_bool, "boolean");
  forward_to_lit!(deserialize_i8, "number");
  forward_to_lit!(deserialize_i16, "number");
  forward_to_lit!(deserialize_i32, "number");
  forward_to_lit!(deserialize_i64, "number");
  forward_to_lit!(deserialize_u8, "number");
  forward_to_lit!(deserialize_u16, "number");
  forward_to_lit!(deserialize_u32, "number");
  forward_to_lit!(deserialize_u64, "number");

  fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: serde::de::Visitor<'de>,
  {
    match self.expr {
      Expr::Lit(lit) => UnpackLit { lit }.deserialize_f32(visitor),
      Expr::Ident(Ident { ref sym, .. }) if sym == "NaN" => visitor.visit_f32(f32::NAN),
      _ => Err(UnpackError::incorrect_expr_type(&self.expr, "float")),
    }
  }

  fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: serde::de::Visitor<'de>,
  {
    match self.expr {
      Expr::Lit(lit) => UnpackLit { lit }.deserialize_f64(visitor),
      Expr::Ident(Ident { ref sym, .. }) if sym == "NaN" => visitor.visit_f64(f64::NAN),
      _ => Err(UnpackError::incorrect_expr_type(&self.expr, "float")),
    }
  }

  fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: serde::de::Visitor<'de>,
  {
    visitor.visit_str(expr_to_str(&self.expr)?.borrow())
  }

  fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: serde::de::Visitor<'de>,
  {
    visitor.visit_string(expr_to_str(&self.expr)?.into_owned())
  }

  deserialize_char!(expr, expr_to_str, UnpackError::incorrect_expr_value);

  fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: serde::de::Visitor<'de>,
  {
    visitor.visit_byte_buf(expr_to_byte_buf(&self.expr)?)
  }

  fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: serde::de::Visitor<'de>,
  {
    visitor.visit_bytes(expr_to_byte_buf(&self.expr)?.as_slice())
  }

  fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: serde::de::Visitor<'de>,
  {
    match self.expr {
      Expr::Lit(Lit::Null(_)) => visitor.visit_none(),
      Expr::Ident(Ident { ref sym, .. }) if sym == "undefined" => visitor.visit_none(),
      expr => visitor.visit_some(UnpackExpr { expr }),
    }
  }

  fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: serde::de::Visitor<'de>,
  {
    match self.expr {
      Expr::Lit(lit) => UnpackLit { lit }.deserialize_unit(visitor),
      Expr::Ident(Ident { ref sym, .. }) if sym == "undefined" => visitor.visit_unit(),
      _ => Err(UnpackError::incorrect_expr_type(
        &self.expr,
        "null or undefined",
      )),
    }
  }

  fn deserialize_unit_struct<V>(
    self,
    _name: &'static str,
    visitor: V,
  ) -> Result<V::Value, Self::Error>
  where
    V: serde::de::Visitor<'de>,
  {
    self.deserialize_unit(visitor)
  }

  fn deserialize_newtype_struct<V>(
    self,
    _name: &'static str,
    visitor: V,
  ) -> Result<V::Value, Self::Error>
  where
    V: serde::de::Visitor<'de>,
  {
    self.deserialize_unit(visitor)
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
    match self.expr {
      Expr::Lit(Lit::Str(Str { value, .. })) => visitor.visit_enum((&*value).into_deserializer()),
      Expr::Object(object) => visitor.visit_enum(UnpackObjectEnum { object }),
      _ => Err(UnpackError::incorrect_expr_type(
        &self.expr,
        "a discriminator or an externally-tagged union",
      )),
    }
  }

  fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: serde::de::Visitor<'de>,
  {
    match self.expr {
      Expr::Lit(lit) => UnpackLit { lit }.deserialize_any(visitor),
      Expr::Tpl(_) => self.deserialize_string(visitor),
      Expr::New(_) => self.deserialize_byte_buf(visitor),
      Expr::Array(array) => visitor.visit_seq(UnpackArray { array, index: 0 }),
      Expr::Object(object) => visitor.visit_map(UnpackObject { object, index: 0 }),
      expr => deserialize_ser(visitor, expr),
    }
  }

  serde::forward_to_deserialize_any! {
    seq tuple tuple_struct
    map struct
    identifier ignored_any
  }
}

impl<'ast> UnpackExpr<'ast> {
  pub fn new(expr: &'ast Expr) -> Self {
    Self { expr }
  }
}

struct UnpackArray<'ast> {
  array: &'ast ArrayLit,
  index: usize,
}

impl<'ast> serde::de::SeqAccess<'ast> for UnpackArray<'ast> {
  type Error = UnpackError;

  fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
  where
    T: serde::de::DeserializeSeed<'ast>,
  {
    let Some(elem) = self.array.elems.get(self.index) else {
      return Ok(None);
    };

    let Some(elem) = elem else {
      return seed.deserialize(UnpackArrayHole).map(Some);
    };

    if elem.spread.is_some() {
      return Err(UnpackError::invalid_value(
        serde::de::Unexpected::Other("spread expression"),
        &"an array element",
      ));
    }

    self.index += 1;

    seed.deserialize(UnpackExpr { expr: &*elem.expr }).map(Some)
  }

  fn size_hint(&self) -> Option<usize> {
    Some(self.array.len())
  }
}

struct UnpackObject<'ast> {
  object: &'ast ObjectLit,
  index: usize,
}

impl<'ast> serde::de::MapAccess<'ast> for UnpackObject<'ast> {
  type Error = UnpackError;

  fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
  where
    K: serde::de::DeserializeSeed<'ast>,
  {
    match self.object.props.get(self.index) {
      None => Ok(None),
      Some(prop) => seed.deserialize(UnpackPropName { prop }).map(Some),
    }
  }

  fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
  where
    V: serde::de::DeserializeSeed<'ast>,
  {
    match self.object.props.get(self.index) {
      None => unreachable!(),
      Some(prop) => {
        self.index += 1;
        seed.deserialize(UnpackExpr {
          expr: &prop_to_key_value(prop)?.value,
        })
      }
    }
  }

  fn size_hint(&self) -> Option<usize> {
    Some(self.object.len())
  }
}

struct UnpackObjectEnum<'ast> {
  object: &'ast ObjectLit,
}

impl<'ast> serde::de::EnumAccess<'ast> for UnpackObjectEnum<'ast> {
  type Error = UnpackError;
  type Variant = UnpackObjectVariant<'ast>;

  fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
  where
    V: serde::de::DeserializeSeed<'ast>,
  {
    let Some(prop) = self.object.props.get(0) else {
      return Err(UnpackError::invalid_length(
        0,
        &"an object with at least 1 key-value property",
      ));
    };

    let key = seed.deserialize(UnpackPropName { prop })?;
    let value = &*prop_to_key_value(prop)?.value;

    Ok((key, UnpackObjectVariant { value }))
  }
}

struct UnpackObjectVariant<'ast> {
  value: &'ast Expr,
}

impl<'ast> serde::de::VariantAccess<'ast> for UnpackObjectVariant<'ast> {
  type Error = UnpackError;

  fn unit_variant(self) -> Result<(), Self::Error> {
    Ok(())
  }

  fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
  where
    T: serde::de::DeserializeSeed<'ast>,
  {
    seed.deserialize(UnpackExpr { expr: &self.value })
  }

  fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: serde::de::Visitor<'ast>,
  {
    UnpackExpr { expr: &self.value }.deserialize_tuple(len, visitor)
  }

  fn struct_variant<V>(
    self,
    fields: &'static [&'static str],
    visitor: V,
  ) -> Result<V::Value, Self::Error>
  where
    V: serde::de::Visitor<'ast>,
  {
    let _ = fields;
    UnpackExpr { expr: &self.value }.deserialize_map(visitor)
  }
}

struct UnpackLit<'ast> {
  lit: &'ast Lit,
}

impl<'de> serde::de::Deserializer<'de> for UnpackLit<'de> {
  type Error = UnpackError;

  deserialize_integer!(lit, lit_to_f64, i8, deserialize_i8, visit_i8);
  deserialize_integer!(lit, lit_to_f64, i16, deserialize_i16, visit_i16);
  deserialize_integer!(lit, lit_to_f64, i32, deserialize_i32, visit_i32);
  deserialize_integer!(lit, lit_to_f64, i64, deserialize_i64, visit_i64);
  deserialize_integer!(lit, lit_to_f64, u8, deserialize_u8, visit_u8);
  deserialize_integer!(lit, lit_to_f64, u16, deserialize_u16, visit_u16);
  deserialize_integer!(lit, lit_to_f64, u32, deserialize_u32, visit_u32);
  deserialize_integer!(lit, lit_to_f64, u64, deserialize_u64, visit_u64);

  deserialize_float!(lit, lit_to_f64, f32, deserialize_f32, visit_f32);
  deserialize_float!(lit, lit_to_f64, f64, deserialize_f64, visit_f64);

  fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: serde::de::Visitor<'de>,
  {
    match self.lit {
      Lit::Null(_) => visitor.visit_unit(),
      _ => Err(UnpackError::incorrect_lit_type(self.lit, &"null")),
    }
  }

  fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: serde::de::Visitor<'de>,
  {
    match self.lit {
      Lit::Bool(Bool { value, .. }) => visitor.visit_bool(*value),
      _ => Err(UnpackError::incorrect_lit_type(self.lit, &"boolean")),
    }
  }

  fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: serde::de::Visitor<'de>,
  {
    visitor.visit_str(lit_to_str(self.lit)?)
  }

  fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: serde::de::Visitor<'de>,
  {
    visitor.visit_string(lit_to_str(self.lit)?.to_string())
  }

  deserialize_char!(lit, lit_to_str, UnpackError::incorrect_lit_value);

  fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: serde::de::Visitor<'de>,
  {
    match self.lit {
      Lit::Null(_) => self.deserialize_unit(visitor),
      Lit::Bool(_) => self.deserialize_bool(visitor),
      Lit::Num(_) => self.deserialize_f64(visitor),
      Lit::Str(_) | Lit::JSXText(_) => self.deserialize_string(visitor),
      Lit::BigInt(BigInt { value, .. }) => deserialize_ser(visitor, value),
      Lit::Regex(value) => deserialize_ser(visitor, value),
    }
  }

  serde::forward_to_deserialize_any! {
    bytes byte_buf
    option enum unit_struct
    map struct newtype_struct
    seq tuple tuple_struct
    identifier ignored_any
  }
}

struct UnpackPropName<'ast> {
  prop: &'ast PropOrSpread,
}

impl<'de> serde::de::Deserializer<'de> for UnpackPropName<'de> {
  type Error = UnpackError;

  deserialize_integer!(prop, prop_name_to_f64, i8, deserialize_i8, visit_i8);
  deserialize_integer!(prop, prop_name_to_f64, i16, deserialize_i16, visit_i16);
  deserialize_integer!(prop, prop_name_to_f64, i32, deserialize_i32, visit_i32);
  deserialize_integer!(prop, prop_name_to_f64, i64, deserialize_i64, visit_i64);
  deserialize_integer!(prop, prop_name_to_f64, u8, deserialize_u8, visit_u8);
  deserialize_integer!(prop, prop_name_to_f64, u16, deserialize_u16, visit_u16);
  deserialize_integer!(prop, prop_name_to_f64, u32, deserialize_u32, visit_u32);
  deserialize_integer!(prop, prop_name_to_f64, u64, deserialize_u64, visit_u64);

  deserialize_float!(prop, prop_name_to_f64, f32, deserialize_f32, visit_f32);
  deserialize_float!(prop, prop_name_to_f64, f64, deserialize_f64, visit_f64);

  fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: serde::de::Visitor<'de>,
  {
    match prop_to_key_value(self.prop)?.key {
      PropName::Computed(ComputedPropName { ref expr, .. }) => {
        UnpackExpr { expr: &*expr }.deserialize_bool(visitor)
      }
      _ => Err(UnpackError::incorrect_prop_type(self.prop, "a boolean key")),
    }
  }

  fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: serde::de::Visitor<'de>,
  {
    visitor.visit_str(prop_name_to_str(self.prop)?)
  }

  deserialize_char!(prop, prop_name_to_str, UnpackError::incorrect_prop_value);

  fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: serde::de::Visitor<'de>,
  {
    match prop_to_key_value(self.prop)?.key {
      PropName::Ident(_) | PropName::Str(_) => self.deserialize_str(visitor),
      PropName::Num(_) => self.deserialize_f64(visitor),
      PropName::BigInt(BigInt { ref value, .. }) => deserialize_ser(visitor, value),
      PropName::Computed(ComputedPropName { ref expr, .. }) => {
        UnpackExpr { expr }.deserialize_any(visitor)
      }
    }
  }

  serde::forward_to_deserialize_any! {
    bytes byte_buf string
    option enum unit unit_struct
    map struct newtype_struct
    seq tuple tuple_struct
    identifier ignored_any
  }
}

struct UnpackArrayHole;

impl<'de> serde::de::Deserializer<'de> for UnpackArrayHole {
  type Error = UnpackError;

  fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: serde::de::Visitor<'de>,
  {
    visitor.visit_unit()
  }

  fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: serde::de::Visitor<'de>,
  {
    visitor.visit_none()
  }

  serde::forward_to_deserialize_any! {
    bytes byte_buf string str char
    bool
    i8 i16 i32 i64
    u8 u16 u32 u64
    f32 f64
    enum unit unit_struct
    map struct newtype_struct
    seq tuple tuple_struct
    identifier ignored_any
  }
}

fn lit_to_f64(lit: &Lit) -> Result<f64, UnpackError> {
  match lit {
    Lit::Num(Number { value, .. }) => Ok(*value),
    _ => Err(UnpackError::incorrect_lit_type(lit, &"a numeric literal")),
  }
}

fn lit_to_str(lit: &Lit) -> Result<&str, UnpackError> {
  match lit {
    Lit::Str(Str { value, .. }) | Lit::JSXText(JSXText { value, .. }) => Ok(&*value),
    _ => Err(UnpackError::incorrect_lit_type(lit, "a string literal")),
  }
}

fn expr_to_str(expr: &Expr) -> Result<Cow<'_, str>, UnpackError> {
  match expr {
    Expr::Lit(lit) => Ok(Cow::from(lit_to_str(lit)?)),
    Expr::Tpl(tpl) => {
      if tpl.exprs.len() > 0 {
        Err(UnpackError::invalid_value(
          serde::de::Unexpected::Other("template literal with expressions"),
          &"a template literal without expressions",
        ))
      } else {
        Ok(Cow::from(
          tpl
            .quasis
            .iter()
            .map(|tpl| tpl.raw.to_string())
            .collect::<Vec<_>>()
            .join(""),
        ))
      }
    }
    _ => Err(UnpackError::incorrect_expr_type(expr, "a string literal")),
  }
}

fn expr_to_byte_buf(expr: &Expr) -> Result<Vec<u8>, UnpackError> {
  let exprs = match expr {
    Expr::New(NewExpr { callee, args, .. }) => {
      let arr = match &**callee {
        Expr::Ident(Ident { sym, .. }) if sym == "Uint8Array" => match args {
          None => None,
          Some(args) => args.first().and_then(|arg| match arg.spread {
            None => match *arg.expr {
              Expr::Array(ArrayLit { ref elems, .. }) => Some(elems),
              _ => None,
            },
            Some(_) => None,
          }),
        },
        _ => None,
      };
      let Some(arr) = arr else {
        return Err(UnpackError::incorrect_expr_value(
          &expr,
          &"a Uint8Array constructor",
        ));
      };
      Ok(arr)
    }
    Expr::Array(ArrayLit { elems, .. }) => Ok(elems),
    _ => Err(UnpackError::incorrect_expr_type(&expr, "a Uint8Array")),
  }?;

  let mut array = vec![];

  for item in exprs {
    let Some(item) = item else { continue };
    let item = match item.spread {
      None => Some(&*item.expr),
      Some(_) => None,
    }
    .ok_or_else(|| UnpackError::incorrect_expr_value(expr, &"a non-spread array item"))?;
    array.push(item);
  }

  let mut byte_array: Vec<u8> = vec![];

  for item in array {
    use serde::Deserialize;
    let de = UnpackExpr { expr: &item };
    byte_array.push(u8::deserialize(de)?);
  }

  Ok(byte_array)
}

fn prop_to_key_value(prop: &PropOrSpread) -> Result<&KeyValueProp, UnpackError> {
  let PropOrSpread::Prop(p) = prop else {
    return Err(UnpackError::incorrect_prop_type(prop, "an object property"));
  };
  let Prop::KeyValue(kv) = &**p else {
    return Err(UnpackError::incorrect_prop_type(
      prop,
      "a key-value property",
    ));
  };
  Ok(&kv)
}

fn prop_name_to_f64(prop: &PropOrSpread) -> Result<f64, UnpackError> {
  match prop_to_key_value(prop)?.key {
    PropName::Num(Number { value, .. }) => Ok(value),
    PropName::Computed(ComputedPropName { ref expr, .. }) => match &**expr {
      Expr::Lit(lit) => lit_to_f64(lit),
      _ => Err(UnpackError::incorrect_prop_type(prop, "a numeric key")),
    },
    _ => Err(UnpackError::incorrect_prop_type(prop, "a numeric key")),
  }
}

fn prop_name_to_str(prop: &PropOrSpread) -> Result<&str, UnpackError> {
  match prop_to_key_value(prop)?.key {
    PropName::Ident(Ident { ref sym, .. }) => Ok(&sym),
    PropName::Str(Str { ref value, .. }) => Ok(&value),
    PropName::Computed(ComputedPropName { ref expr, .. }) => match &**expr {
      Expr::Lit(lit) => lit_to_str(lit),
      _ => Err(UnpackError::incorrect_prop_type(prop, "a string key")),
    },
    _ => Err(UnpackError::incorrect_prop_type(prop, "a string key")),
  }
}

fn deserialize_ser<'a, T, V>(visitor: V, data: &T) -> Result<V::Value, UnpackError>
where
  T: serde::Serialize,
  V: serde::de::Visitor<'a>,
{
  visit_serde_data(visitor, to_serde_data(data)).map_err(<UnpackError as serde::de::Error>::custom)
}

trait BoundedNumber {
  const MIN: f64;
  const MAX: f64;
  const NAME: &'static str;
  fn from_f64_unchecked(number: f64) -> Self;
}

macro_rules! bounded_number {
  ($num:ty) => {
    impl BoundedNumber for $num {
      const MAX: f64 = <$num>::MAX as f64;
      const MIN: f64 = <$num>::MIN as f64;
      const NAME: &'static str = stringify!($num);
      fn from_f64_unchecked(number: f64) -> Self {
        number as $num
      }
    }
  };
}

bounded_number!(i8);
bounded_number!(i16);
bounded_number!(i32);
bounded_number!(i64);
bounded_number!(u8);
bounded_number!(u16);
bounded_number!(u32);
bounded_number!(u64);
bounded_number!(f32);
bounded_number!(f64);

fn bounded_float<T: BoundedNumber>(number: f64) -> Result<T, UnpackError> {
  if number < T::MIN || number > T::MAX {
    Err(UnpackError::invalid_value(
      serde::de::Unexpected::Float(number),
      &T::NAME,
    ))
  } else {
    Ok(T::from_f64_unchecked(number))
  }
}

fn bounded_integer<T: BoundedNumber>(number: f64) -> Result<T, UnpackError> {
  if number < T::MIN || number > T::MAX {
    Err(UnpackError::invalid_value(
      serde::de::Unexpected::Float(number),
      &T::NAME,
    ))
  } else if number.fract() != 0.0 {
    Err(UnpackError::invalid_value(
      serde::de::Unexpected::Float(number),
      &T::NAME,
    ))
  } else {
    Ok(T::from_f64_unchecked(number))
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

impl UnpackError {
  fn expr_type(expr: &Expr) -> serde::de::Unexpected {
    match expr {
      Expr::Lit(lit) => Self::lit_type(lit),
      Expr::Array(_) => serde::de::Unexpected::Seq,
      Expr::Object(_) => serde::de::Unexpected::Map,
      _ => serde::de::Unexpected::Other("arbitrary expression"),
    }
  }

  fn lit_type(lit: &Lit) -> serde::de::Unexpected {
    match lit {
      Lit::Null(_) => serde::de::Unexpected::Unit,
      Lit::Bool(Bool { value, .. }) => serde::de::Unexpected::Bool(*value),
      Lit::Num(Number { value, .. }) => serde::de::Unexpected::Float(*value),
      Lit::Str(Str { value, .. }) => serde::de::Unexpected::Str(&*value),
      Lit::JSXText(JSXText { value, .. }) => serde::de::Unexpected::Str(&*value),
      Lit::BigInt(_) => serde::de::Unexpected::Other("bigint"),
      Lit::Regex(_) => serde::de::Unexpected::Other("regex"),
    }
  }

  fn prop_type(prop: &PropOrSpread) -> serde::de::Unexpected {
    match prop {
      PropOrSpread::Spread(_) => serde::de::Unexpected::Other("spread expression"),
      PropOrSpread::Prop(prop) => match **prop {
        Prop::Assign(_) => serde::de::Unexpected::Other("assignment expression"),
        Prop::Getter(_) => serde::de::Unexpected::Other("getter"),
        Prop::Setter(_) => serde::de::Unexpected::Other("setter"),
        Prop::Method(_) => serde::de::Unexpected::Other("method"),
        Prop::Shorthand(_) => serde::de::Unexpected::Other("object shorthand"),
        Prop::KeyValue(ref prop) => match prop.key {
          PropName::Ident(Ident { ref sym, .. }) => serde::de::Unexpected::Str(&sym),
          PropName::Str(Str { ref value, .. }) => serde::de::Unexpected::Str(&value),
          PropName::Num(Number { value, .. }) => serde::de::Unexpected::Float(value),
          PropName::BigInt(_) => serde::de::Unexpected::Other("bigint"),
          PropName::Computed(_) => serde::de::Unexpected::Other("computed property"),
        },
      },
    }
  }

  fn incorrect_lit_type(lit: &Lit, expected: &str) -> UnpackError {
    Self::invalid_type(Self::lit_type(lit), &expected)
  }

  fn incorrect_lit_value(lit: &Lit, expected: &str) -> UnpackError {
    Self::invalid_value(Self::lit_type(lit), &expected)
  }

  fn incorrect_prop_type(prop: &PropOrSpread, expected: &str) -> UnpackError {
    Self::invalid_type(Self::prop_type(prop), &expected)
  }

  fn incorrect_prop_value(prop: &PropOrSpread, expected: &str) -> UnpackError {
    Self::invalid_value(Self::prop_type(prop), &expected)
  }

  fn incorrect_expr_type(expr: &Expr, expected: &str) -> UnpackError {
    Self::invalid_type(Self::expr_type(expr), &expected)
  }

  fn incorrect_expr_value(expr: &Expr, expected: &str) -> UnpackError {
    Self::invalid_value(Self::expr_type(expr), &expected)
  }
}

pub fn unpack_expr<'ast, T>(expr: &'ast Expr) -> Result<T, UnpackError>
where
  T: serde::de::Deserialize<'ast>,
{
  T::deserialize(UnpackExpr { expr })
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

    let request: Request = unpack_expr(&*expr).unwrap();

    let body = request.body.clone().unwrap();
    let body = String::from_utf8_lossy(&body);

    assert_eq!(body, "Never gonna give you up");
  }
}
