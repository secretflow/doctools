use swc_core::ecma::ast::{ArrayLit, Expr, Ident, KeyValueProp, Null, ObjectLit, Prop, PropName};

pub fn json_expr(value: serde_json::Value) -> Box<Expr> {
  use serde_json::Value::*;
  match value {
    Null => null().into(),
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
        .map(|v| Some(json_expr(v).into()))
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
            value: json_expr(v),
          })
          .into()
        })
        .collect(),
      span: Default::default(),
    }
    .into(),
  }
}

pub(crate) fn null() -> Null {
  Null {
    span: Default::default(),
  }
}

#[cfg(test)]
mod tests {
  use serde_json::json;
  use swc_core::{ecma::codegen::Config, testing::DebugUsingDisplay};

  use crate::testing::print_one;

  use super::json_expr;

  #[test]
  fn test_json_expr() {
    let value = json!({
        "null": null,
        "bool": true,
        "number": 1,
        "string": "string",
        "array": [42, [{"object": true}]],
    });
    let code = print_one(
      &json_expr(value),
      None,
      Some(Config::default().with_minify(true)),
    );
    assert_eq!(
      DebugUsingDisplay(code.as_str()),
      DebugUsingDisplay(
        r#"{"null":null,"bool":true,"number":1,"string":"string","array":[42,[{"object":true}]]}"#
      )
    );
  }
}
