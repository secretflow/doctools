use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use swc_core::{
  atoms::Atom,
  common::Span,
  ecma::ast::{Expr, Ident, Lit, Str},
};

use super::JSXRuntime;

pub trait JSXTagDef {
  fn to_expr<R: JSXRuntime>(&self, span: Span) -> Expr;
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", content = "type")]
pub enum JSXTagType<'a> {
  Intrinsic(&'a str),
  Component(&'a str),
  Fragment,
}

impl Debug for JSXTagType<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      JSXTagType::Intrinsic(name) => write!(f, "<{:?}>", name),
      JSXTagType::Component(name) => write!(f, "<{}>", name),
      JSXTagType::Fragment => write!(f, "<>"),
    }
  }
}

impl JSXTagDef for JSXTagType<'_> {
  fn to_expr<R: JSXRuntime>(&self, span: Span) -> Expr {
    match self {
      JSXTagType::Intrinsic(name) => Expr::Lit(Lit::Str(Str {
        span,
        value: (*name).into(),
        raw: None,
      })),
      JSXTagType::Component(name) => Expr::Ident(Ident {
        sym: (*name).into(),
        span,
        optional: false,
      }),
      JSXTagType::Fragment => Expr::Ident(Ident {
        sym: R::FRAGMENT.into(),
        span,
        optional: false,
      }),
    }
  }
}

impl<'a> JSXTagType<'a> {
  pub fn from_expr<R: JSXRuntime>(expr: &'a Expr) -> Option<Self> {
    match expr {
      Expr::Lit(Lit::Str(Str { value, .. })) => Some(JSXTagType::Intrinsic(value)),
      Expr::Ident(Ident { sym, .. }) => {
        if sym == R::FRAGMENT {
          Some(JSXTagType::Fragment)
        } else {
          Some(JSXTagType::Component(sym))
        }
      }
      _ => None,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", content = "type")]
pub enum JSXTagTypeOwned {
  Intrinsic(Atom),
  Component(Atom),
  Fragment,
}

impl JSXTagTypeOwned {
  pub fn to_borrowed(&self) -> JSXTagType {
    match self {
      JSXTagTypeOwned::Intrinsic(name) => JSXTagType::Intrinsic(name.as_str()),
      JSXTagTypeOwned::Component(name) => JSXTagType::Component(name.as_str()),
      JSXTagTypeOwned::Fragment => JSXTagType::Fragment,
    }
  }
}

impl From<JSXTagType<'_>> for JSXTagTypeOwned {
  fn from(tag: JSXTagType) -> Self {
    match tag {
      JSXTagType::Intrinsic(name) => JSXTagTypeOwned::Intrinsic(name.into()),
      JSXTagType::Component(name) => JSXTagTypeOwned::Component(name.into()),
      JSXTagType::Fragment => JSXTagTypeOwned::Fragment,
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
macro_rules! matches_tag {
  ($arg:expr, $($tt:tt)+) => {
    matches!($arg, $crate::tag_test!($($tt)+))
  };
}

#[macro_export]
macro_rules! ad_hoc_tag {
  (<>) => {
    $crate::jsx::JSXTagType::Fragment
  };
  ($tag:literal) => {
    $crate::jsx::JSXTagType::Intrinsic($tag.into())
  };
  ($tag:ident) => {
    $crate::jsx::JSXTagType::Component(stringify!($tag).into())
  };
  ("" $tag:expr) => {
    $crate::jsx::JSXTagType::Intrinsic($tag)
  };
  (<> $tag:expr) => {
    $crate::jsx::JSXTagType::Component($tag)
  };
}

#[macro_export]
macro_rules! tag_whitelist {
  ( $vis:vis enum $name:ident { $($tags:ident,)* } ) => {
    $vis enum $name { $($tags),* }

    impl $crate::jsx::JSXTagDef for $name {
      fn to_expr<R: $crate::jsx::JSXRuntime>(&self, span: swc_core::common::Span) -> swc_core::ecma::ast::Expr {
        match self {
          $(
            $name::$tags => $crate::jsx::JSXTagType::Component(stringify!($tags)).to_expr::<R>(span)
          ),*
        }
      }
    }
  };
}
