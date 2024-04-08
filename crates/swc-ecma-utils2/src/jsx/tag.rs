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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum JSXTagType<'a> {
  Intrinsic(&'a str),
  Component(&'a str),
  Fragment,
}

pub trait JSXTagDef {
  fn tag_type(&self) -> JSXTagType<'_>;

  fn into_expr<R: JSXRuntime>(&self) -> Expr {
    match self.tag_type() {
      JSXTagType::Intrinsic(tag) => Expr::Lit(tag.into()),
      JSXTagType::Component(tag) => Expr::Ident(Ident {
        sym: tag.into(),
        span: Default::default(),
        optional: false,
      }),
      JSXTagType::Fragment => Expr::Ident(Ident {
        sym: R::FRAGMENT.into(),
        span: Default::default(),
        optional: false,
      }),
    }
  }
}

impl JSXTagDef for JSXTag {
  fn tag_type(&self) -> JSXTagType<'_> {
    match self.kind {
      JSXTagKind::Intrinsic => JSXTagType::Intrinsic(&self.name),
      JSXTagKind::Component => JSXTagType::Component(&self.name),
      JSXTagKind::Fragment => JSXTagType::Fragment,
    }
  }

  fn into_expr<R: JSXRuntime>(&self) -> Expr {
    match self.kind {
      JSXTagKind::Intrinsic => Expr::Lit((*self.name).into()),
      JSXTagKind::Component => Expr::Ident(Ident {
        sym: (&*self.name).into(),
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
macro_rules! tag_test {
  (<>?) => {
    $crate::jsx::JSXTagType::Fragment
  };
  ("*"?) => {
    $crate::jsx::JSXTagType::Intrinsic(_)
  };
  (*?) => {
    $crate::jsx::JSXTagType::Component(_)
  };
  ("*" as $name:ident?) => {
    $crate::jsx::JSXTagType::Intrinsic($name)
  };
  (* as $name:ident?) => {
    $crate::jsx::JSXTagType::Component($name)
  };
  ($tag:literal?) => {
    $crate::jsx::JSXTagType::Intrinsic($tag)
  };
  ($tag:ident?) => {
    $crate::jsx::JSXTagType::Component(stringify!($tag))
  };
}

#[macro_export]
macro_rules! ad_hoc_tag {
  (<>) => {
    $crate::jsx::JSXTag::fragment()
  };
  ($tag:literal) => {
    $crate::jsx::JSXTag::intrinsic($tag.into())
  };
  ($tag:ident) => {
    $crate::jsx::JSXTag::component(stringify!($tag).into())
  };
  ("" $tag:expr) => {
    $crate::jsx::JSXTag::intrinsic($tag)
  };
  (<> $tag:expr) => {
    $crate::jsx::JSXTag::component($tag)
  };
}

#[macro_export]
macro_rules! tag_whitelist {
  ( $vis:vis enum $name:ident { $($tags:ident,)* } ) => {
    $vis enum $name { $($tags),* }

    impl $crate::jsx::JSXTagDef for $name {
      fn tag_type(&self) -> $crate::jsx::JSXTagType<'_> {
        match self {
          $(Self::$tags => $crate::jsx::JSXTagType::Component(stringify!($tags)),)*
        }
      }
    }
  };
}
