use serde::{Deserialize, Serialize};
use swc_core::atoms::Atom;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum JSXTagKind {
  Intrinsic,
  Component,
  Fragment,
}

pub enum JSXTagType<'a> {
  Intrinsic(&'a str),
  Component(&'a str),
  Fragment,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JSXTag {
  pub kind: JSXTagKind,
  #[serde(rename = "type")]
  pub name: Atom,
}

impl JSXTag {
  pub fn intrinsic(value: &str) -> Self {
    Self {
      kind: JSXTagKind::Intrinsic,
      name: value.into(),
    }
  }

  pub fn component(value: &str) -> Self {
    Self {
      kind: JSXTagKind::Component,
      name: value.into(),
    }
  }

  pub fn fragment() -> Self {
    Self {
      kind: JSXTagKind::Fragment,
      name: "".into(),
    }
  }
}

impl JSXTag {
  pub fn tag_type(&self) -> JSXTagType<'_> {
    match self.kind {
      JSXTagKind::Intrinsic => JSXTagType::Intrinsic(&self.name),
      JSXTagKind::Component => JSXTagType::Component(&self.name),
      JSXTagKind::Fragment => JSXTagType::Fragment,
    }
  }
}

pub trait JSXTagMatch {
  fn tag_type(&self) -> Option<JSXTagType<'_>>;
}

impl JSXTagMatch for Option<JSXTag> {
  fn tag_type(&self) -> Option<JSXTagType<'_>> {
    match self {
      Some(tag) => Some(tag.tag_type()),
      None => None,
    }
  }
}
