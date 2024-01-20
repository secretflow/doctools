/// please don't use this crate
use serde::{Deserialize, Serialize, Serializer};
use serde_json::{value::RawValue, Number};
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct JSXElement {
  #[serde(rename = "$$jsx")]
  r#type: ElementType,
  name: String,
  props: HashMap<String, JSXValue>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ElementType {
  Intrinsic,
  Identifier,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum JSXValue {
  Element(JSXElement),
  Null,
  Bool(bool),
  Number(Number),
  String(String),
  Array(Vec<JSXValue>),
  Object(HashMap<String, JSXValue>),
}

impl Serialize for JSXElement {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let mut ecmascript = String::new();

    match self.props.get("children") {
      Some(JSXValue::Array(children)) if children.len() > 1 => {
        ecmascript += "jsxs";
      }
      _ => ecmascript += "jsx",
    };

    ecmascript += "(";

    match &self.r#type {
      ElementType::Intrinsic => {
        ecmascript += &serde_json::to_string(&self.name).unwrap();
      }
      ElementType::Identifier => {
        ecmascript += &self.name;
      }
    };

    ecmascript += ",";

    ecmascript += &serde_json::to_string(&self.props).unwrap();

    ecmascript += ")";

    let unsafe_inline = unsafe { std::mem::transmute::<&str, &RawValue>(&ecmascript) };

    unsafe_inline.serialize(serializer)
  }
}
