use serde::{Deserialize, Serialize};
use swc_core::{
  atoms::Atom,
  ecma::ast::{Expr, Ident},
};

use super::JSXRuntime;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum JSXTagKind {
  Intrinsic,
  Component,
  Fragment,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

  pub fn into_expr<R: JSXRuntime>(self) -> Expr {
    match self.kind {
      JSXTagKind::Intrinsic => Expr::Lit(self.name.into()),
      JSXTagKind::Component => Expr::Ident(Ident {
        sym: self.name,
        span: Default::default(),
        optional: false,
      }),
      JSXTagKind::Fragment => Expr::Ident(Ident {
        sym: R::FRAGMENT.into(),
        span: Default::default(),
        optional: false,
      }),
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

#[macro_export]
macro_rules! tag {
  (<>?) => {
    $crate::jsx::tag::JSXTagType::Fragment
  };
  ("*"?) => {
    $crate::jsx::tag::JSXTagType::Intrinsic(_)
  };
  (*?) => {
    $crate::jsx::tag::JSXTagType::Component(_)
  };
  ("*" as $name:ident?) => {
    $crate::jsx::tag::JSXTagType::Intrinsic($name)
  };
  (* as $name:ident?) => {
    $crate::jsx::tag::JSXTagType::Component($name)
  };
  ($tag:literal?) => {
    $crate::jsx::tag::JSXTagType::Intrinsic($tag)
  };
  ($tag:ident?) => {
    $crate::jsx::tag::JSXTagType::Component(stringify!($tag))
  };
  (<>) => {
    $crate::jsx::tag::JSXTag::fragment()
  };
  ($tag:literal) => {
    $crate::jsx::tag::JSXTag::intrinsic($tag.into())
  };
  ($tag:ident) => {
    $crate::jsx::tag::JSXTag::component(stringify!($tag).into())
  };
  ("" $tag:expr) => {
    $crate::jsx::tag::JSXTag::intrinsic($tag)
  };
  (<> $tag:expr) => {
    $crate::jsx::tag::JSXTag::component($tag)
  };
}
