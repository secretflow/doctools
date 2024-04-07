use swc_core::{
  common::util::take::Take as _,
  ecma::ast::{self, ArrayLit, Expr, Ident, KeyValueProp, ObjectLit, Prop, PropName},
};

pub fn json_to_expr(value: serde_json::Value) -> Expr {
  use serde_json::Value::*;
  match value {
    Null => ast::Null::dummy().into(),
    Bool(value) => value.into(),
    String(value) => value.into(),
    Number(number) => number
      .as_f64()
      .and_then(|f| Some(Expr::from(f)))
      .unwrap_or_else(|| Expr::from(Ident::from("NaN")))
      .into(),
    Array(elems) => ArrayLit {
      elems: elems
        .into_iter()
        .map(|v| Some(json_to_expr(v).into()))
        .collect(),
      span: Default::default(),
    }
    .into(),
    Object(props) => ObjectLit {
      props: props
        .into_iter()
        .map(|(k, v)| {
          Prop::from(KeyValueProp {
            key: PropName::Str(k.into()),
            value: json_to_expr(v).into(),
          })
          .into()
        })
        .collect(),
      span: Default::default(),
    }
    .into(),
  }
}

#[macro_export]
macro_rules! json_expr {
  ($($tokens:tt)+) => {
    $crate::ecma::json::json_to_expr(serde_json::json!($($tokens)+))
  };
}
