use std::marker::PhantomData;

use serde::{de::IntoDeserializer, Deserializer};

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum SerdeData {
  bool(bool),
  i8(i8),
  i16(i16),
  i32(i32),
  i64(i64),
  u8(u8),
  u16(u16),
  u32(u32),
  u64(u64),
  f32(f32),
  f64(f64),
  char(char),
  str(String),
  bytes(Vec<u8>),
  unit,
  unit_variant(&'static str),
  newtype_struct(Box<SerdeData>),
  newtype_variant(&'static str, Box<SerdeData>),
  seq(Vec<SerdeData>),
  tuple(Vec<SerdeData>),
  tuple_struct(Vec<SerdeData>),
  tuple_variant(&'static str, Vec<SerdeData>),
  map(Vec<(SerdeData, SerdeData)>),
  struct_(Vec<(SerdeData, SerdeData)>),
  struct_variant(&'static str, Vec<(SerdeData, SerdeData)>),
}

#[derive(Debug)]
struct PassthruSerializer {
  data: SerdeData,
}

impl Default for PassthruSerializer {
  fn default() -> Self {
    Self {
      data: SerdeData::unit,
    }
  }
}

macro_rules! primitive {
  ($f:ident, $type:ident) => {
    fn $f(self, v: $type) -> Result<Self::Ok, Self::Error> {
      self.data = SerdeData::$type(v);
      Ok(())
    }
  };
}

impl<'a> serde::ser::Serializer for &'a mut PassthruSerializer {
  type Ok = ();
  type Error = PassthruSerdeError;

  type SerializeSeq = PassthruSerializeList<'a>;
  type SerializeTuple = PassthruSerializeList<'a>;
  type SerializeTupleStruct = PassthruSerializeList<'a>;
  type SerializeTupleVariant = PassthruSerializeList<'a>;
  type SerializeMap = PassthruSerializeDict<'a>;
  type SerializeStruct = PassthruSerializeDict<'a>;
  type SerializeStructVariant = PassthruSerializeDict<'a>;

  primitive!(serialize_bool, bool);
  primitive!(serialize_i8, i8);
  primitive!(serialize_i16, i16);
  primitive!(serialize_i32, i32);
  primitive!(serialize_i64, i64);
  primitive!(serialize_u8, u8);
  primitive!(serialize_u16, u16);
  primitive!(serialize_u32, u32);
  primitive!(serialize_u64, u64);
  primitive!(serialize_f32, f32);
  primitive!(serialize_f64, f64);
  primitive!(serialize_char, char);

  fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
    self.data = SerdeData::str(v.to_string());
    Ok(())
  }

  fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
    self.data = SerdeData::bytes(v.to_vec());
    Ok(())
  }

  fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
    self.data = SerdeData::unit;
    Ok(())
  }

  fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
  where
    T: serde::Serialize,
  {
    value.serialize(self)
  }

  fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
    self.data = SerdeData::unit;
    Ok(())
  }

  fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
    self.data = SerdeData::unit;
    Ok(())
  }

  fn serialize_unit_variant(
    self,
    _name: &'static str,
    _variant_index: u32,
    variant: &'static str,
  ) -> Result<Self::Ok, Self::Error> {
    self.data = SerdeData::unit_variant(variant);
    Ok(())
  }

  fn serialize_newtype_struct<T: ?Sized>(
    self,
    _name: &'static str,
    value: &T,
  ) -> Result<Self::Ok, Self::Error>
  where
    T: serde::Serialize,
  {
    let mut ser = PassthruSerializer::default();
    value.serialize(&mut ser)?;
    self.data = SerdeData::newtype_struct(Box::new(ser.data));
    Ok(())
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
    let mut ser = PassthruSerializer::default();
    value.serialize(&mut ser)?;
    self.data = SerdeData::newtype_variant(variant, Box::new(ser.data));
    Ok(())
  }

  fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
    self.data = SerdeData::seq(Vec::with_capacity(len.unwrap_or(0)));
    Ok(PassthruSerializeList {
      data: match self.data {
        SerdeData::seq(ref mut v) => v,
        _ => unreachable!(),
      },
    })
  }

  fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
    self.data = SerdeData::tuple(Vec::with_capacity(len));
    Ok(PassthruSerializeList {
      data: match self.data {
        SerdeData::tuple(ref mut v) => v,
        _ => unreachable!(),
      },
    })
  }

  fn serialize_tuple_struct(
    self,
    _name: &'static str,
    len: usize,
  ) -> Result<Self::SerializeTupleStruct, Self::Error> {
    self.data = SerdeData::tuple_struct(Vec::with_capacity(len));
    Ok(PassthruSerializeList {
      data: match self.data {
        SerdeData::tuple_struct(ref mut v) => v,
        _ => unreachable!(),
      },
    })
  }

  fn serialize_tuple_variant(
    self,
    _name: &'static str,
    _variant_index: u32,
    variant: &'static str,
    len: usize,
  ) -> Result<Self::SerializeTupleVariant, Self::Error> {
    self.data = SerdeData::tuple_variant(variant, Vec::with_capacity(len));
    Ok(PassthruSerializeList {
      data: match self.data {
        SerdeData::tuple_variant(_, ref mut v) => v,
        _ => unreachable!(),
      },
    })
  }

  fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
    self.data = SerdeData::map(Vec::with_capacity(len.unwrap_or(0)));
    Ok(PassthruSerializeDict {
      data: match self.data {
        SerdeData::map(ref mut v) => v,
        _ => unreachable!(),
      },
    })
  }

  fn serialize_struct(
    self,
    _name: &'static str,
    len: usize,
  ) -> Result<Self::SerializeStruct, Self::Error> {
    self.data = SerdeData::struct_(Vec::with_capacity(len));
    Ok(PassthruSerializeDict {
      data: match self.data {
        SerdeData::struct_(ref mut v) => v,
        _ => unreachable!(),
      },
    })
  }

  fn serialize_struct_variant(
    self,
    _name: &'static str,
    _variant_index: u32,
    variant: &'static str,
    len: usize,
  ) -> Result<Self::SerializeStructVariant, Self::Error> {
    self.data = SerdeData::struct_variant(variant, Vec::with_capacity(len));
    Ok(PassthruSerializeDict {
      data: match self.data {
        SerdeData::struct_variant(_, ref mut v) => v,
        _ => unreachable!(),
      },
    })
  }
}

#[derive(Debug)]
struct PassthruSerializeList<'a> {
  data: &'a mut Vec<SerdeData>,
}

impl PassthruSerializeList<'_> {
  fn push<T>(&mut self, value: &T) -> PassthruResult<()>
  where
    T: ?Sized + serde::Serialize,
  {
    let mut ser = PassthruSerializer::default();
    value.serialize(&mut ser)?;
    self.data.push(ser.data);
    Ok(())
  }
}

impl<'a> serde::ser::SerializeSeq for PassthruSerializeList<'a> {
  type Ok = ();
  type Error = PassthruSerdeError;

  fn serialize_element<T: ?Sized>(&mut self, value: &T) -> PassthruResult<()>
  where
    T: serde::Serialize,
  {
    self.push(value)
  }

  fn end(self) -> Result<Self::Ok, Self::Error> {
    Ok(())
  }
}

impl<'a> serde::ser::SerializeTuple for PassthruSerializeList<'a> {
  type Ok = ();
  type Error = PassthruSerdeError;

  fn serialize_element<T: ?Sized>(&mut self, value: &T) -> PassthruResult<()>
  where
    T: serde::Serialize,
  {
    self.push(value)
  }

  fn end(self) -> Result<Self::Ok, Self::Error> {
    Ok(())
  }
}

impl<'a> serde::ser::SerializeTupleStruct for PassthruSerializeList<'a> {
  type Ok = ();
  type Error = PassthruSerdeError;

  fn serialize_field<T: ?Sized>(&mut self, value: &T) -> PassthruResult<()>
  where
    T: serde::Serialize,
  {
    self.push(value)
  }

  fn end(self) -> Result<Self::Ok, Self::Error> {
    Ok(())
  }
}

impl<'a> serde::ser::SerializeTupleVariant for PassthruSerializeList<'a> {
  type Ok = ();
  type Error = PassthruSerdeError;

  fn serialize_field<T: ?Sized>(&mut self, value: &T) -> PassthruResult<()>
  where
    T: serde::Serialize,
  {
    self.push(value)
  }

  fn end(self) -> Result<Self::Ok, Self::Error> {
    Ok(())
  }
}

#[derive(Debug)]
struct PassthruSerializeDict<'a> {
  data: &'a mut Vec<(SerdeData, SerdeData)>,
}

impl PassthruSerializeDict<'_> {
  fn key<T>(&mut self, value: &T) -> PassthruResult<()>
  where
    T: ?Sized + serde::Serialize,
  {
    let mut ser = PassthruSerializer::default();
    value.serialize(&mut ser)?;
    self.data.push((ser.data, SerdeData::unit));
    Ok(())
  }

  fn value<T>(&mut self, value: &T) -> PassthruResult<()>
  where
    T: ?Sized + serde::Serialize,
  {
    let mut ser = PassthruSerializer::default();
    value.serialize(&mut ser)?;
    self.data.last_mut().unwrap().1 = ser.data;
    Ok(())
  }
}

impl<'a> serde::ser::SerializeMap for PassthruSerializeDict<'a> {
  type Ok = ();
  type Error = PassthruSerdeError;

  fn serialize_key<T: ?Sized>(&mut self, key: &T) -> PassthruResult<()>
  where
    T: serde::Serialize,
  {
    self.key(key)
  }

  fn serialize_value<T: ?Sized>(&mut self, value: &T) -> PassthruResult<()>
  where
    T: serde::Serialize,
  {
    self.value(value)
  }

  fn end(self) -> Result<Self::Ok, Self::Error> {
    Ok(())
  }
}

impl<'a> serde::ser::SerializeStruct for PassthruSerializeDict<'a> {
  type Ok = ();
  type Error = PassthruSerdeError;

  fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> PassthruResult<()>
  where
    T: serde::Serialize,
  {
    self.key(key)?;
    self.value(value)
  }

  fn end(self) -> Result<Self::Ok, Self::Error> {
    Ok(())
  }
}

impl<'a> serde::ser::SerializeStructVariant for PassthruSerializeDict<'a> {
  type Ok = ();
  type Error = PassthruSerdeError;

  fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> PassthruResult<()>
  where
    T: serde::Serialize,
  {
    self.key(key)?;
    self.value(value)
  }

  fn end(self) -> Result<Self::Ok, Self::Error> {
    Ok(())
  }
}

#[derive(Debug)]
struct PassthruDeserializer<'de> {
  data: SerdeData,
  visitor: PhantomData<&'de ()>,
}

impl<'de> PassthruDeserializer<'de> {
  fn new(data: SerdeData) -> Self {
    Self {
      data,
      visitor: PhantomData,
    }
  }
}

impl<'de: 'a, 'a> serde::de::Deserializer<'de> for &'a mut PassthruDeserializer<'de> {
  type Error = PassthruSerdeError;

  fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: serde::de::Visitor<'de>,
  {
    match self.data {
      SerdeData::bool(v) => visitor.visit_bool(v),
      SerdeData::i8(v) => visitor.visit_i8(v),
      SerdeData::i16(v) => visitor.visit_i16(v),
      SerdeData::i32(v) => visitor.visit_i32(v),
      SerdeData::i64(v) => visitor.visit_i64(v),
      SerdeData::u8(v) => visitor.visit_u8(v),
      SerdeData::u16(v) => visitor.visit_u16(v),
      SerdeData::u32(v) => visitor.visit_u32(v),
      SerdeData::u64(v) => visitor.visit_u64(v),
      SerdeData::f32(v) => visitor.visit_f32(v),
      SerdeData::f64(v) => visitor.visit_f64(v),
      SerdeData::char(v) => visitor.visit_char(v),
      SerdeData::str(ref v) => visitor.visit_str(v),
      SerdeData::bytes(ref v) => visitor.visit_bytes(v),

      SerdeData::unit => visitor.visit_unit(),
      SerdeData::unit_variant(v) => visitor.visit_enum(v.into_deserializer()),

      SerdeData::newtype_struct(ref mut data) => {
        self.data = std::mem::replace(&mut *data, SerdeData::unit);
        visitor.visit_newtype_struct(self)
      }

      SerdeData::seq(ref mut items)
      | SerdeData::tuple(ref mut items)
      | SerdeData::tuple_struct(ref mut items) => {
        let items = std::mem::replace(items, Vec::new());
        let de = PassthruDeserializeList::new(self, items);
        visitor.visit_seq(de)
      }

      SerdeData::map(ref mut items) | SerdeData::struct_(ref mut items) => {
        let items = std::mem::replace(items, Vec::new());
        let de = PassthruDeserializeDict::new(self, items);
        visitor.visit_map(de)
      }

      SerdeData::newtype_variant(variant, ref mut data) => {
        self.data = std::mem::replace(&mut *data, SerdeData::unit);
        let de = PassthruDeserializeEnum::new(self, variant);
        visitor.visit_enum(de)
      }
      SerdeData::tuple_variant(variant, ref mut items) => {
        let items = std::mem::replace(items, Vec::new());
        self.data = SerdeData::tuple(items);
        let de = PassthruDeserializeEnum::new(self, variant);
        visitor.visit_enum(de)
      }
      SerdeData::struct_variant(variant, ref mut items) => {
        let items = std::mem::replace(items, Vec::new());
        self.data = SerdeData::struct_(items);
        let de = PassthruDeserializeEnum::new(self, variant);
        visitor.visit_enum(de)
      }
    }
  }

  fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: serde::de::Visitor<'de>,
  {
    match self.data {
      SerdeData::unit => visitor.visit_none(),
      _ => visitor.visit_some(self),
    }
  }

  serde::forward_to_deserialize_any! {
    bool
    byte_buf bytes char str string
    u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 f32 f64
    unit unit_struct
    enum
    map struct newtype_struct
    seq tuple tuple_struct
    identifier ignored_any
  }
}

#[derive(Debug)]
struct PassthruDeserializeList<'de: 'a, 'a> {
  de: &'a mut PassthruDeserializer<'de>,
  list: Vec<SerdeData>,
  index: usize,
}

impl<'de, 'a> PassthruDeserializeList<'de, 'a> {
  fn new(de: &'a mut PassthruDeserializer<'de>, list: Vec<SerdeData>) -> Self {
    Self { de, list, index: 0 }
  }
}

impl<'de, 'a> serde::de::SeqAccess<'de> for PassthruDeserializeList<'de, 'a> {
  type Error = PassthruSerdeError;

  fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
  where
    T: serde::de::DeserializeSeed<'de>,
  {
    if self.index >= self.list.len() {
      return Ok(None);
    } else {
      let item = std::mem::replace(&mut self.list[self.index], SerdeData::unit);
      self.index += 1;
      self.de.data = item;
      seed.deserialize(&mut *self.de).map(Some)
    }
  }
}

#[derive(Debug)]
struct PassthruDeserializeDict<'de: 'a, 'a> {
  de: &'a mut PassthruDeserializer<'de>,
  dict: Vec<(SerdeData, SerdeData)>,
  index: usize,
}

impl<'de, 'a> PassthruDeserializeDict<'de, 'a> {
  fn new(de: &'a mut PassthruDeserializer<'de>, dict: Vec<(SerdeData, SerdeData)>) -> Self {
    Self { de, dict, index: 0 }
  }
}

impl<'de, 'a> serde::de::MapAccess<'de> for PassthruDeserializeDict<'de, 'a> {
  type Error = PassthruSerdeError;

  fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
  where
    K: serde::de::DeserializeSeed<'de>,
  {
    if self.index >= self.dict.len() {
      return Ok(None);
    } else {
      let key = std::mem::replace(&mut self.dict[self.index].0, SerdeData::unit);
      self.de.data = key;
      seed.deserialize(&mut *self.de).map(Some)
    }
  }

  fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
  where
    V: serde::de::DeserializeSeed<'de>,
  {
    let value = std::mem::replace(&mut self.dict[self.index].1, SerdeData::unit);
    self.de.data = value;
    self.index += 1;
    seed.deserialize(&mut *self.de)
  }
}

#[derive(Debug)]
struct PassthruDeserializeEnum<'de: 'a, 'a> {
  de: &'a mut PassthruDeserializer<'de>,
  variant: &'static str,
}

impl<'de, 'a> PassthruDeserializeEnum<'de, 'a> {
  fn new(de: &'a mut PassthruDeserializer<'de>, variant: &'static str) -> Self {
    Self { de, variant }
  }
}

impl<'de, 'a> serde::de::EnumAccess<'de> for PassthruDeserializeEnum<'de, 'a> {
  type Error = PassthruSerdeError;
  type Variant = Self;

  fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
  where
    V: serde::de::DeserializeSeed<'de>,
  {
    let value = seed.deserialize(self.variant.into_deserializer())?;
    Ok((value, self))
  }
}

impl<'de, 'a> serde::de::VariantAccess<'de> for PassthruDeserializeEnum<'de, 'a> {
  type Error = PassthruSerdeError;

  fn unit_variant(self) -> Result<(), Self::Error> {
    use serde::de::Error;
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

  fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: serde::de::Visitor<'de>,
  {
    serde::de::Deserializer::deserialize_tuple(self.de, len, visitor)
  }

  fn struct_variant<V>(
    self,
    fields: &'static [&'static str],
    visitor: V,
  ) -> Result<V::Value, Self::Error>
  where
    V: serde::de::Visitor<'de>,
  {
    serde::de::Deserializer::deserialize_struct(self.de, self.variant, fields, visitor)
  }
}

#[derive(Debug, thiserror::Error)]
#[error("failed to de/serialize: {0}")]
pub struct PassthruSerdeError(String);

impl serde::ser::Error for PassthruSerdeError {
  fn custom<T>(msg: T) -> Self
  where
    T: std::fmt::Display,
  {
    Self(format!("{}", msg))
  }
}

impl serde::de::Error for PassthruSerdeError {
  fn custom<T>(msg: T) -> Self
  where
    T: std::fmt::Display,
  {
    Self(format!("{}", msg))
  }
}

type PassthruResult<T> = Result<T, PassthruSerdeError>;

pub fn to_serde_data<T>(value: &T) -> SerdeData
where
  T: serde::Serialize,
{
  let mut ser = PassthruSerializer::default();
  value.serialize(&mut ser).expect("passthru should not fail");
  ser.data
}

pub fn from_serde_data<'de, T>(data: SerdeData) -> PassthruResult<T>
where
  T: serde::Deserialize<'de>,
{
  let mut de = PassthruDeserializer::new(data);
  T::deserialize(&mut de)
}

pub fn visit_serde_data<'de, V>(visitor: V, data: SerdeData) -> PassthruResult<V::Value>
where
  V: serde::de::Visitor<'de>,
{
  let mut de = PassthruDeserializer::new(data);
  de.deserialize_any(visitor)
}
