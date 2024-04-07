use serde::{ser::SerializeSeq as _, Serialize as _};
use swc_core::{
  common::util::take::Take as _,
  ecma::ast::{
    ArrayLit, BigInt, ComputedPropName, Expr, ExprOrSpread, KeyValueProp, Lit, NewExpr, Null,
    ObjectLit, Prop, PropName,
  },
};

use crate::{
  collections::{MutableMapping, MutableSequence},
  var,
};

#[derive(Debug)]
struct RepackExpr;

macro_rules! safe_number {
  ($f:ident, $type:ty) => {
    fn $f(self, v: $type) -> Result<Self::Ok, Self::Error> {
      let num: f64 = v.into();
      Ok(Expr::Lit(num.into()))
    }
  };
}

impl<'a> serde::ser::Serializer for &'a mut RepackExpr {
  type Ok = Expr;
  type Error = RepackError;

  type SerializeSeq = RepackArray;
  type SerializeTuple = RepackArray;
  type SerializeTupleStruct = RepackArray;
  type SerializeTupleVariant = RepackTaggedArray;
  type SerializeMap = RepackMap;
  type SerializeStruct = RepackObject;
  type SerializeStructVariant = RepackTaggedObject;

  fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
    Ok(Expr::Lit(v.into()))
  }

  fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
    Ok(Expr::Lit(v.into()))
  }

  safe_number!(serialize_i8, i8);
  safe_number!(serialize_i16, i16);
  safe_number!(serialize_i32, i32);
  safe_number!(serialize_u8, u8);
  safe_number!(serialize_u16, u16);
  safe_number!(serialize_u32, u32);
  safe_number!(serialize_f32, f32);

  fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
    if v <= f64::MAX as i64 {
      Ok(Expr::Lit((v as f64).into()))
    } else {
      Ok(Expr::Lit(Lit::BigInt(BigInt {
        value: Box::new(v.into()),
        span: Default::default(),
        raw: None,
      })))
    }
  }

  fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
    if v <= f64::MAX as u64 {
      Ok(Expr::Lit((v as f64).into()))
    } else {
      Ok(Expr::Lit(Lit::BigInt(BigInt {
        value: Box::new(v.into()),
        span: Default::default(),
        raw: None,
      })))
    }
  }

  fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
    Ok(Expr::Lit(v.to_string().into()))
  }

  fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
    Ok(Expr::Lit(v.into()))
  }

  fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
    let mut ser = RepackExpr;
    let mut seq = ser.serialize_seq(Some(v.len()))?;
    v.iter().try_for_each(|v| seq.serialize_element(v))?;
    let array = seq.end()?;

    let new = NewExpr {
      callee: var!(Uint8Array).into(),
      args: Some(vec![ExprOrSpread {
        expr: Box::new(array),
        spread: None,
      }]),
      type_args: None,
      span: Default::default(),
    };

    Ok(new.into())
  }

  fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
    Ok(Null::dummy().into())
  }

  fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
  where
    T: serde::Serialize,
  {
    let mut ser = RepackExpr;
    value.serialize(&mut ser)
  }

  fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
    Ok(Null::dummy().into())
  }

  fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
    self.serialize_unit()
  }

  fn serialize_unit_variant(
    self,
    _name: &'static str,
    _variant_index: u32,
    variant: &'static str,
  ) -> Result<Self::Ok, Self::Error> {
    variant.serialize(&mut RepackExpr)
  }

  fn serialize_newtype_struct<T: ?Sized>(
    self,
    _name: &'static str,
    value: &T,
  ) -> Result<Self::Ok, Self::Error>
  where
    T: serde::Serialize,
  {
    value.serialize(&mut RepackExpr)
  }

  fn serialize_newtype_variant<T: ?Sized>(
    self,
    _name: &'static str,
    _variant_index: u32,
    variant: &'static str,
    value: &T,
  ) -> Result<Self::Ok, Self::Error>
  where
    T: serde::Serialize,
  {
    let tag = variant
      .serialize(&mut RepackExpr)?
      .lit()
      .expect("should be a string literal");
    let value = value.serialize(&mut RepackExpr)?;
    let mut object = ObjectLit::dummy();
    object.set_item(tag, value);
    Ok(object.into())
  }

  fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
    Ok(RepackArray::new())
  }

  fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
    Ok(RepackArray::new())
  }

  fn serialize_tuple_struct(
    self,
    _name: &'static str,
    _len: usize,
  ) -> Result<Self::SerializeTupleStruct, Self::Error> {
    Ok(RepackArray::new())
  }

  fn serialize_tuple_variant(
    self,
    _name: &'static str,
    _variant_index: u32,
    variant: &'static str,
    _len: usize,
  ) -> Result<Self::SerializeTupleVariant, Self::Error> {
    Ok(RepackTaggedArray::new(variant))
  }

  fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
    Ok(RepackMap::new())
  }

  fn serialize_struct(
    self,
    _name: &'static str,
    _len: usize,
  ) -> Result<Self::SerializeStruct, Self::Error> {
    Ok(RepackObject::new())
  }

  fn serialize_struct_variant(
    self,
    _name: &'static str,
    _variant_index: u32,
    variant: &'static str,
    _len: usize,
  ) -> Result<Self::SerializeStructVariant, Self::Error> {
    Ok(RepackTaggedObject::new(variant))
  }
}
#[derive(Debug)]

struct RepackArray {
  data: ArrayLit,
}

impl RepackArray {
  fn new() -> Self {
    Self {
      data: ArrayLit::dummy(),
    }
  }
}

impl serde::ser::SerializeSeq for RepackArray {
  type Ok = Expr;
  type Error = RepackError;

  fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
  where
    T: serde::Serialize,
  {
    self.data.append(value.serialize(&mut RepackExpr)?);
    Ok(())
  }

  fn end(self) -> Result<Self::Ok, Self::Error> {
    Ok(self.data.into())
  }
}

impl serde::ser::SerializeTuple for RepackArray {
  type Ok = Expr;
  type Error = RepackError;

  fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
  where
    T: serde::Serialize,
  {
    self.data.append(value.serialize(&mut RepackExpr)?);
    Ok(())
  }

  fn end(self) -> Result<Self::Ok, Self::Error> {
    Ok(self.data.into())
  }
}

impl serde::ser::SerializeTupleStruct for RepackArray {
  type Ok = Expr;
  type Error = RepackError;

  fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
  where
    T: serde::Serialize,
  {
    self.data.append(value.serialize(&mut RepackExpr)?);
    Ok(())
  }

  fn end(self) -> Result<Self::Ok, Self::Error> {
    Ok(self.data.into())
  }
}
#[derive(Debug)]

struct RepackTaggedArray {
  tag: &'static str,
  data: ArrayLit,
}

impl RepackTaggedArray {
  fn new(tag: &'static str) -> Self {
    Self {
      tag,
      data: ArrayLit::dummy(),
    }
  }
}

impl serde::ser::SerializeTupleVariant for RepackTaggedArray {
  type Ok = Expr;
  type Error = RepackError;

  fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
  where
    T: serde::Serialize,
  {
    self.data.append(value.serialize(&mut RepackExpr)?);
    Ok(())
  }

  fn end(self) -> Result<Self::Ok, Self::Error> {
    let mut object = ObjectLit::dummy();
    object.set_item(self.tag, self.data.into());
    Ok(object.into())
  }
}

#[derive(Debug)]
struct RepackMap {
  data: ObjectLit,
  key: Expr,
}

impl RepackMap {
  fn new() -> Self {
    Self {
      data: ObjectLit::dummy(),
      key: Expr::dummy(),
    }
  }
}

impl serde::ser::SerializeMap for RepackMap {
  type Ok = Expr;
  type Error = RepackError;

  fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
  where
    T: serde::Serialize,
  {
    self.key = key.serialize(&mut RepackExpr)?;
    Ok(())
  }

  fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
  where
    T: serde::Serialize,
  {
    let value = value.serialize(&mut RepackExpr)?;

    let key = match self.key.take() {
      Expr::Lit(Lit::Str(string)) => PropName::Str(string),
      Expr::Lit(Lit::Num(number)) => PropName::Num(number),
      Expr::Lit(Lit::BigInt(bigint)) => PropName::BigInt(bigint),
      expr => PropName::Computed(ComputedPropName {
        expr: Box::new(expr),
        span: Default::default(),
      }),
    };

    self.data.props.push(
      Prop::KeyValue(KeyValueProp {
        key,
        value: Box::new(value),
      })
      .into(),
    );

    Ok(())
  }

  fn end(self) -> Result<Self::Ok, Self::Error> {
    Ok(self.data.into())
  }
}

#[derive(Debug)]
struct RepackObject {
  data: ObjectLit,
}

impl RepackObject {
  fn new() -> Self {
    Self {
      data: ObjectLit::dummy(),
    }
  }
}

impl serde::ser::SerializeStruct for RepackObject {
  type Ok = Expr;
  type Error = RepackError;

  fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
  where
    T: serde::Serialize,
  {
    self.data.set_item(key, value.serialize(&mut RepackExpr)?);
    Ok(())
  }

  fn end(self) -> Result<Self::Ok, Self::Error> {
    Ok(self.data.into())
  }
}

#[derive(Debug)]
struct RepackTaggedObject {
  tag: &'static str,
  data: ObjectLit,
}

impl RepackTaggedObject {
  fn new(tag: &'static str) -> Self {
    Self {
      tag,
      data: ObjectLit::dummy(),
    }
  }
}

impl serde::ser::SerializeStructVariant for RepackTaggedObject {
  type Ok = Expr;
  type Error = RepackError;

  fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
  where
    T: serde::Serialize,
  {
    self.data.set_item(key, value.serialize(&mut RepackExpr)?);
    Ok(())
  }

  fn end(self) -> Result<Self::Ok, Self::Error> {
    let mut object = ObjectLit::dummy();
    object.set_item(self.tag, self.data.into());
    Ok(object.into())
  }
}

#[derive(Debug, thiserror::Error)]
#[error("failed to pack data as Expr: {0}")]
pub struct RepackError(String);

impl serde::ser::Error for RepackError {
  fn custom<T: std::fmt::Display>(msg: T) -> Self {
    Self(format!("{}", msg))
  }
}

pub fn repack_expr<T: serde::Serialize>(value: &T) -> Result<Expr, RepackError> {
  value.serialize(&mut RepackExpr)
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;

  use serde::{Deserialize, Serialize};
  use swc_ecma_testing2::{insta::assert_snapshot, print_one};

  use super::repack_expr;

  #[derive(Debug, Serialize, Deserialize)]
  enum HTTPStatusCode {
    OK,
    PartialContent { range: String },
    PermanentRedirect { location: String },
    BadRequest { reason: String },
    NotFound,
    Teapot,
  }

  #[derive(Debug, Serialize, Deserialize)]
  struct Response {
    status: HTTPStatusCode,
    headers: HashMap<String, String>,
    #[serde(with = "serde_bytes")]
    body: Vec<u8>,
  }

  #[test]
  fn test_repack() {
    let response = Response {
      status: HTTPStatusCode::PartialContent {
        range: "bytes 0-1023/2048".into(),
      },
      headers: {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".into(), "text/plain".into());
        headers
      },
      body: b"I'm a teapot".to_vec(),
    };

    let expr = repack_expr(&response).unwrap();

    let result = print_one(&expr, None, None).unwrap();
    let result = format!("``````js\n{}\n``````", result);

    assert_snapshot!(result);
  }
}
