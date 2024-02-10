use serde::{Deserialize, Serialize};
use swc_core::atoms::Atom;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", content = "type")]
pub enum JSXTag {
  Intrinsic(Atom),
  Component(Atom),
  Fragment,
}

impl JSXTag {
  pub fn intrinsic(value: &str) -> Self {
    JSXTag::Intrinsic(value.into())
  }

  pub fn component(value: &str) -> Self {
    JSXTag::Component(value.into())
  }

  pub fn fragment() -> Self {
    JSXTag::Fragment
  }
}

impl JSXTag {
  pub fn tuple(&self) -> (&Self, &str) {
    match self {
      JSXTag::Intrinsic(value) => (self, &**value),
      JSXTag::Component(value) => (self, &**value),
      JSXTag::Fragment => (self, ""),
    }
  }
}
