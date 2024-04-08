use std::marker::PhantomData;

use serde::{
  de::{Error, IntoDeserializer},
  Deserializer,
};
use swc_core::ecma::ast::{CallExpr, Expr};

use crate::ecma::{UnpackError, UnpackExpr};

use super::{jsx, tag::JSXTagType, JSXElement, JSXRuntime};

struct UnpackJSX<'ast, R: JSXRuntime> {
  call: &'ast CallExpr,
  runtime: PhantomData<R>,
}

impl<'de, R: JSXRuntime> serde::de::Deserializer<'de> for UnpackJSX<'de, R> {
  type Error = UnpackJSXError<'de>;

  fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: serde::de::Visitor<'de>,
  {
    let Self { call, runtime } = self;
    if jsx::<R>(call).is_some() {
      visitor.visit_enum(UnpackJSXComponent { call, runtime })
    } else {
      Err(UnpackJSXError::custom("not a JSX element"))
    }
  }

  fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: serde::de::Visitor<'de>,
  {
    visitor.visit_some(self)
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

struct UnpackJSXComponent<'ast, R: JSXRuntime> {
  call: &'ast CallExpr,
  runtime: PhantomData<R>,
}

impl<'ast, R: JSXRuntime> serde::de::EnumAccess<'ast> for UnpackJSXComponent<'ast, R> {
  type Error = UnpackJSXError<'ast>;
  type Variant = UnpackJSXProps<'ast>;

  fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
  where
    V: serde::de::DeserializeSeed<'ast>,
  {
    let elem = jsx::<R>(&self.call);

    let component = elem
      .get_tag()
      .ok_or_else(|| UnpackJSXError::custom("could not get tag type"))?;

    let key: Result<V::Value, UnpackJSXError<'ast>> = match component.tag_type() {
      JSXTagType::Component(name) | JSXTagType::Intrinsic(name) => {
        seed.deserialize(name.into_deserializer())
      }
      JSXTagType::Fragment => seed.deserialize(R::FRAGMENT.into_deserializer()),
    };

    let key = key?;

    let props = {
      let arg1 = self
        .call
        .args
        .get(1)
        .ok_or_else(|| UnpackError::custom("could not get props"))?;

      if arg1.spread.is_some() {
        return Err(UnpackError::custom("spread props are not supported").into());
      }

      &arg1.expr
    };

    Ok((key, UnpackJSXProps { props }))
  }
}

struct UnpackJSXProps<'ast> {
  props: &'ast Expr,
}

impl<'ast> serde::de::VariantAccess<'ast> for UnpackJSXProps<'ast> {
  type Error = UnpackJSXError<'ast>;

  fn unit_variant(self) -> Result<(), Self::Error> {
    Ok(())
  }

  fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
  where
    T: serde::de::DeserializeSeed<'ast>,
  {
    seed
      .deserialize(UnpackExpr::new(&self.props))
      .map_err(Into::into)
  }

  fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: serde::de::Visitor<'ast>,
  {
    UnpackExpr::new(&self.props)
      .deserialize_tuple(len, visitor)
      .map_err(Into::into)
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
    UnpackExpr::new(&self.props)
      .deserialize_map(visitor)
      .map_err(Into::into)
  }
}

#[derive(Debug, thiserror::Error)]
pub enum UnpackJSXError<'ast> {
  #[error("unexpected component: {unexpected}, expected one of: {expected:?}")]
  ComponentMismatch {
    unexpected: String,
    expected: &'static [&'static str],
  },
  #[error("invalid props: {0}")]
  InvalidProps(UnpackError<'ast>),
  #[error(transparent)]
  UnpackError(UnpackError<'ast>),
}

impl serde::de::Error for UnpackJSXError<'_> {
  fn custom<T: std::fmt::Display>(msg: T) -> Self {
    UnpackJSXError::UnpackError(UnpackError::custom(msg))
  }

  fn unknown_variant(variant: &str, expected: &'static [&'static str]) -> Self {
    UnpackJSXError::ComponentMismatch {
      unexpected: variant.to_string(),
      expected,
    }
  }
}

impl<'ast> From<UnpackError<'ast>> for UnpackJSXError<'ast> {
  fn from(err: UnpackError<'ast>) -> Self {
    UnpackJSXError::InvalidProps(err)
  }
}

pub fn unpack_jsx<'ast, R, T>(call: &'ast CallExpr) -> Result<T, UnpackJSXError>
where
  R: JSXRuntime,
  T: serde::de::Deserialize<'ast>,
{
  T::deserialize(UnpackJSX {
    call,
    runtime: PhantomData::<R>,
  })
}

#[cfg(test)]

mod tests {
  use serde::Deserialize;
  use swc_core::ecma::parser::parse_file_as_expr;
  use swc_ecma_testing2::parse_one;

  use crate::jsx::{unpack::unpack_jsx, JSXRuntime};

  struct Runtime;

  impl JSXRuntime for Runtime {
    const JSX: &'static str = "jsx";
    const JSXS: &'static str = "jsxs";
    const FRAGMENT: &'static str = "Fragment";
  }

  #[test]
  fn test_unpack_jsx_1() {
    #[derive(Debug, PartialEq, Deserialize)]
    enum SectionElement {
      #[serde(rename = "article")]
      Article,
      #[serde(rename = "section")]
      Section,
    }

    let src = r#"jsx("section", {})"#;

    let call = parse_one(src, None, parse_file_as_expr)
      .unwrap()
      .expect_call();

    let elem = unpack_jsx::<Runtime, SectionElement>(&call).unwrap();

    assert_eq!(elem, SectionElement::Section {});
  }

  #[test]
  fn test_unpack_jsx_2() {
    #[derive(Debug, PartialEq, Deserialize)]
    enum MediaElement {
      #[serde(rename = "img")]
      Image {
        src: String,
        #[serde(default)]
        alt: String,
      },
      #[serde(rename = "video")]
      Video {
        #[serde(default)]
        src: Option<String>,
        #[serde(default)]
        controls: bool,
        #[serde(default)]
        width: Option<u32>,
      },
    }

    let src = r#"
    jsxs(video, {
      controls: true,
      width: 320,
      children: [
        jsx(source, {
          src: "movie.mp4",
          type: "video/mp4",
        }),
        "Your browser does not support the video tag.",
      ]
    })
    "#;

    let call = parse_one(src, None, parse_file_as_expr)
      .unwrap()
      .expect_call();

    let elem = unpack_jsx::<Runtime, MediaElement>(&call).unwrap();

    assert_eq!(
      elem,
      MediaElement::Video {
        src: None,
        controls: true,
        width: Some(320)
      }
    );
  }

  #[test]
  fn test_unpack_jsx_3() {
    #[derive(Debug, PartialEq, Deserialize)]
    enum MetaElement {
      #[serde(rename = "meta")]
      Meta(Meta),
    }

    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(untagged)]
    enum Meta {
      Document {
        name: String,
        content: String,
      },
      Pragma {
        #[serde(rename = "httpEquiv")]
        http_equiv: String,
        content: String,
      },
      Charset {
        charset: String,
      },
      Property {
        itemprop: String,
        content: String,
      },
    }

    let src = r#"
    jsx(meta, {
      httpEquiv: "Content-Security-Policy",
      content: "default-src 'self'",
    })
    "#;

    let call = parse_one(src, None, parse_file_as_expr)
      .unwrap()
      .expect_call();

    let elem = unpack_jsx::<Runtime, MetaElement>(&call).unwrap();

    assert_eq!(
      elem,
      MetaElement::Meta(Meta::Pragma {
        http_equiv: "Content-Security-Policy".to_string(),
        content: "default-src 'self'".to_string()
      })
    );
  }
}
