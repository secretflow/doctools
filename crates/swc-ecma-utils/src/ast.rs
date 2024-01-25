use swc_core::{
  atoms::Atom,
  ecma::{
    ast::{
      ArrayLit, Bool, CallExpr, Expr, ExprOrSpread, Ident, KeyValueProp, Lit, Null, Number,
      ObjectLit, Prop, PropName, PropOrSpread, Regex, Str,
    },
    visit::{VisitMut, VisitMutWith},
  },
};

pub fn json_to_expr(value: serde_json::Value) -> Box<Expr> {
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
            value: json_to_expr(v),
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

pub enum JSONLossy<'ast> {
  _Expr(&'ast Expr),
  _Null,
  _String(&'ast Str),
  _Number(&'ast Number),
  _Boolean(&'ast Bool),
  _Array(&'ast ArrayLit),
  _Object(&'ast ObjectLit),
}

impl<'ast> serde::ser::Serialize for JSONLossy<'ast> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::ser::Serializer,
  {
    use serde::{ser::SerializeMap as _, ser::SerializeSeq as _};
    use JSONLossy::*;

    match *self {
      _Expr(Expr::Array(array)) | _Array(array) => {
        let elems = array
          .elems
          .iter()
          .filter_map(|e| match *e {
            Some(ExprOrSpread { ref expr, .. }) => Some(_Expr(&**expr)),
            None => None,
          })
          .collect::<Vec<_>>();
        let mut seq = serializer.serialize_seq(Some(elems.len()))?;
        for elem in elems {
          seq.serialize_element(&elem)?
        }
        seq.end()
      }

      _Expr(Expr::Object(object)) | _Object(object) => {
        let props = object
          .props
          .iter()
          .filter_map(|prop| match prop {
            PropOrSpread::Prop(prop) => match &**prop {
              Prop::KeyValue(KeyValueProp { key, value, .. }) => match key {
                PropName::Ident(Ident { sym, .. }) => Some((sym, &**value)),
                PropName::Str(Str { value: key, .. }) => Some((key, &**value)),
                _ => None,
              },
              _ => None,
            },
            _ => None,
          })
          .collect::<Vec<_>>();
        let mut map = serializer.serialize_map(Some(props.len()))?;
        for (key, value) in props {
          map.serialize_entry(key, &_Expr(value))?
        }
        map.end()
      }

      _Expr(Expr::Lit(Lit::Str(Str { value, .. }))) | _String(Str { value, .. }) => {
        serializer.serialize_str(&**value)
      }

      _Expr(Expr::Lit(Lit::Num(Number { value, .. }))) | _Number(Number { value, .. }) => {
        if value.fract() == 0.0 {
          serializer.serialize_i64(value.trunc() as i64)
        } else {
          serializer.serialize_f64(*value)
        }
      }

      _Expr(Expr::Lit(Lit::Bool(Bool { value, .. }))) | _Boolean(Bool { value, .. }) => {
        serializer.serialize_bool(*value)
      }

      _Expr(Expr::Lit(Lit::Null(_))) | _Null => serializer.serialize_none(),

      _Expr(Expr::Lit(Lit::Regex(Regex { exp, .. }))) => serializer.serialize_str(&**exp),

      _Expr(Expr::Lit(Lit::BigInt(bigint))) => bigint.value.serialize(serializer),

      _ => serializer.serialize_none(),
    }
  }
}

macro_rules! into_json_lossy {
  ($ty:ty, $variant:ident) => {
    impl<'ast> Into<JSONLossy<'ast>> for &'ast $ty {
      fn into(self) -> JSONLossy<'ast> {
        JSONLossy::$variant(self)
      }
    }
  };
}

into_json_lossy!(Expr, _Expr);
into_json_lossy!(Str, _String);
into_json_lossy!(Number, _Number);
into_json_lossy!(Bool, _Boolean);
into_json_lossy!(ArrayLit, _Array);
into_json_lossy!(ObjectLit, _Object);

pub fn expr_to_json_lossy<'ast, E, T>(from: E) -> Result<T, serde_json::Error>
where
  E: Into<JSONLossy<'ast>>,
  T: serde::de::DeserializeOwned,
{
  let intermediate = serde_json::to_string(&from.into())?;
  serde_json::from_str(&intermediate)
}

pub enum SelectNode<'ast> {
  Found(&'ast Expr),
  NotFound,
}

impl<'ast> From<&'ast Expr> for SelectNode<'ast> {
  fn from(expr: &'ast Expr) -> Self {
    Self::Found(expr)
  }
}

impl<'ast> SelectNode<'ast> {
  pub fn key(self, key: &str) -> Self {
    match self {
      Self::NotFound => Self::NotFound,
      Self::Found(Expr::Object(object)) => SelectNode::from_key(object, key),
      _ => Self::NotFound,
    }
  }

  pub fn index(self, index: usize) -> Self {
    match self {
      Self::NotFound => Self::NotFound,
      Self::Found(Expr::Array(array)) => SelectNode::from_index(array, index),
      _ => Self::NotFound,
    }
  }

  pub fn get(self) -> Option<&'ast Expr> {
    match self {
      Self::NotFound => None,
      Self::Found(expr) => Some(expr),
    }
  }

  pub fn as_string(self) -> Option<&'ast str> {
    match self {
      Self::NotFound => None,
      Self::Found(Expr::Lit(Lit::Str(Str { ref value, .. }))) => Some(&**value),
      _ => None,
    }
  }

  pub fn as_number(self) -> Option<f64> {
    match self {
      Self::NotFound => None,
      Self::Found(Expr::Lit(Lit::Num(Number { value, .. }))) => Some(*value),
      _ => None,
    }
  }
}

impl<'ast> SelectNode<'ast> {
  pub fn from_key(object: &'ast ObjectLit, key: &str) -> Self {
    let props = &object.props;
    let found = props.iter().find_map(|p| {
      let entry = p.as_prop().and_then(|p| p.as_key_value());
      if let Some(entry) = entry {
        let target = match entry.key {
          PropName::Ident(Ident { ref sym, .. }) => Some((&**sym).to_string()),
          PropName::Str(Str { ref value, .. }) => Some((&**value).to_string()),
          PropName::Num(Number { ref value, .. }) => Some(value.to_string()),
          _ => None,
        };
        match target {
          Some(target) if target == key => Some(&entry.value),
          _ => None,
        }
      } else {
        None
      }
    });
    match found {
      Some(expr) => Self::Found(&*expr),
      _ => Self::NotFound,
    }
  }

  pub fn from_index(object: &'ast ArrayLit, index: usize) -> Self {
    let elems = &object.elems;
    let found = elems.get(index).and_then(|e| e.as_ref());
    match found {
      Some(expr) => Self::Found(&*expr.expr),
      _ => Self::NotFound,
    }
  }
}

pub struct PropMutator<'callback, T> {
  path: Vec<Atom>,
  set_intermediate_path: bool,
  idx: usize,
  callback: Option<Box<dyn FnOnce(&mut Box<Expr>) -> T + 'callback>>,
  result: Option<T>,
}

impl<T> PropMutator<'_, T> {
  fn traverse<N: VisitMutWith<Self>>(&mut self, expr: &mut N) {
    self.idx += 1;
    expr.visit_mut_with(self);
  }

  fn found(&mut self, found: &mut Box<Expr>) {
    let callback = self.callback.take().expect("callback already taken");
    self.result = Some((callback)(found))
  }

  fn process_array_elems(&mut self, elems: &mut Vec<Option<ExprOrSpread>>) {
    let subscript = self.path.get(self.idx).expect("index out of bounds");

    let index = match subscript.parse::<usize>() {
      Ok(idx) => idx,
      Err(_) => {
        self.result = None;
        return;
      }
    };

    let idx = self.idx;
    let path_len = self.path.len();

    let mut set_default = |arr: &mut Vec<Option<ExprOrSpread>>| {
      if self.set_intermediate_path {
        if index >= arr.len() {
          arr.resize_with(index + 1, || None);
        }
        arr[index] = Some(ExprOrSpread {
          spread: None,
          expr: {
            let mut new = Box::from(null());
            self.found(&mut new);
            new
          },
        })
      } else {
        self.result = None;
        return;
      }
    };

    let ensure_object = || {
      Some(ExprOrSpread {
        spread: None,
        expr: Box::from(ObjectLit {
          props: vec![],
          span: Default::default(),
        }),
      })
    };

    if idx == path_len - 1 {
      match elems.get_mut(index) {
        Some(item) => match item {
          Some(ExprOrSpread { expr, .. }) => {
            self.found(expr);
          }
          None => set_default(elems),
        },
        None => set_default(elems),
      }
    } else {
      match elems.get_mut(index) {
        Some(next) => match next {
          Some(ExprOrSpread { expr, .. }) => self.traverse(expr),
          None => {
            if self.set_intermediate_path {
              *next = ensure_object();
              self.traverse(next.as_mut().unwrap().expr.as_mut());
            } else {
              self.result = None;
              return;
            }
          }
        },
        None => {
          if self.set_intermediate_path {
            elems.resize_with(index + 1, || None);
            elems[index] = ensure_object();
            self.traverse(elems[index].as_mut().unwrap().expr.as_mut());
          } else {
            self.result = None;
            return;
          }
        }
      };
    }
  }

  fn process_object_props(&mut self, props: &'_ mut Vec<PropOrSpread>) {
    let subscript = self.path.get(self.idx).expect("index out of bounds");

    let key = Atom::new(&**subscript);
    let key_eq = prop_is_named(&key);

    let existing = props.iter_mut().find_map(|p| {
      key_eq(p)
        .then(|| p.as_mut_prop().unwrap())
        .and_then(|p| p.as_mut_key_value())
    });

    if self.idx == self.path.len() - 1 {
      match existing {
        Some(KeyValueProp { value, .. }) => {
          self.found(value);
        }
        None => {
          if self.set_intermediate_path {
            props.push(
              PropOrSpread::Prop(
                Prop::from(KeyValueProp {
                  key: PropName::Str((&*key).into()),
                  value: {
                    let mut new = Box::from(null());
                    self.found(&mut new);
                    new
                  },
                })
                .into(),
              )
              .into(),
            )
          } else {
            self.result = None;
            return;
          }
        }
      }
    } else {
      match existing {
        Some(prop) => self.traverse(&mut prop.value),
        None => {
          if self.set_intermediate_path {
            props.push(
              PropOrSpread::Prop(
                Prop::from(KeyValueProp {
                  key: PropName::Str((&*key).into()),
                  value: Box::from(ObjectLit {
                    props: vec![],
                    span: Default::default(),
                  }),
                })
                .into(),
              )
              .into(),
            );
            self.traverse(
              props
                .last_mut()
                .unwrap()
                .as_mut_prop()
                .unwrap()
                .as_mut_key_value()
                .unwrap()
                .value
                .as_mut(),
            );
          } else {
            self.result = None;
            return;
          }
        }
      }
    }
  }
}

impl<T> VisitMut for PropMutator<'_, T> {
  fn visit_mut_array_lit(&mut self, array: &mut ArrayLit) {
    self.process_array_elems(&mut array.elems);
  }

  fn visit_mut_object_lit(&mut self, object: &mut ObjectLit) {
    self.process_object_props(&mut object.props)
  }

  fn visit_mut_call_expr(&mut self, call: &mut CallExpr) {
    let mut args = call.args.drain(..).map(Some).collect();
    self.process_array_elems(&mut args);
    call.args = args.drain(..).map(Option::unwrap).collect();
  }

  fn visit_mut_expr(&mut self, expr: &mut Expr) {
    match expr {
      Expr::Array(array) => self.visit_mut_array_lit(array),
      Expr::Object(object) => self.visit_mut_object_lit(object),
      Expr::Call(call) => self.visit_mut_call_expr(call),
      _ => {}
    }
  }
}

impl<'callback, T> PropMutator<'callback, T> {
  pub fn mut_with<E, F>(
    ast: &'callback mut E,
    path: &[&str],
    callback: F,
    set_default: bool,
  ) -> Option<T>
  where
    E: VisitMutWith<PropMutator<'callback, T>>,
    F: FnOnce(&mut Box<Expr>) -> T + 'callback,
  {
    let mut visitor = PropMutator {
      path: path.iter().map(|p| (*p).into()).collect(),
      set_intermediate_path: set_default,
      idx: 0,
      result: None,
      callback: Some(Box::new(callback)),
    };
    ast.visit_mut_with(&mut visitor);
    visitor.result
  }
}

fn prop_is_named(key: &str) -> impl Fn(&PropOrSpread) -> bool + '_ {
  move |prop: &PropOrSpread| match prop.as_prop().and_then(|p| p.as_key_value()) {
    None => false,
    Some(KeyValueProp { key: prop, .. }) => match prop {
      PropName::Str(Str { value, .. }) => &*value == key,
      PropName::Ident(Ident { sym, .. }) => &*sym == key,
      PropName::Num(Number { value, .. }) => value.to_string() == key,
      _ => false,
    },
  }
}

#[cfg(test)]
mod tests {
  use serde::Deserialize;
  use serde_json::json;
  use swc_core::{
    ecma::{
      ast::{CallExpr, Lit},
      codegen::Config,
      parser::parse_file_as_expr,
    },
    testing::DebugUsingDisplay,
  };

  use crate::{
    ast::expr_to_json_lossy,
    testing::{parse_one, print_one},
  };

  use super::{json_to_expr, PropMutator};

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
      &json_to_expr(value),
      None,
      Some(Config::default().with_minify(true)),
    );
    assert_eq!(
      DebugUsingDisplay(code.unwrap().as_str()),
      DebugUsingDisplay(
        r#"{"null":null,"bool":true,"number":1,"string":"string","array":[42,[{"object":true}]]}"#
      )
    );
  }

  #[test]
  fn test_expr_json() {
    #[derive(Deserialize, Debug, PartialEq, Eq)]
    struct Test {
      string: String,
      number: i64,
      boolean: bool,
      #[serde(rename = "tuple")]
      array: (i64, i64, i64, i64),
      #[serde(default)]
      optional: Option<String>,
    }

    let expr = parse_one(
      r#"
      {
          string: "Lorem ipsum",
          number: 128,
          boolean: true,
          tuple: [1, 2, 4, 8],
      }
      "#,
      None,
      parse_file_as_expr,
    )
    .unwrap();

    let test: Test = expr_to_json_lossy(&*expr).unwrap();

    assert_eq!(
      test,
      Test {
        string: "Lorem ipsum".to_string(),
        number: 128,
        boolean: true,
        array: (1, 2, 4, 8),
        optional: None,
      }
    );
  }

  #[test]
  fn test_set_object() {
    let mut expr = json_to_expr(json!({}));

    PropMutator::mut_with(
      &mut expr,
      &["children"],
      |expr| *expr = json_to_expr(json!([])),
      true,
    )
    .unwrap();

    let code = print_one(&expr, None, Some(Config::default().with_minify(true)));
    assert_eq!(
      DebugUsingDisplay(code.unwrap().as_str()),
      DebugUsingDisplay(r#"{"children":[]}"#)
    );
  }

  #[test]
  fn test_set_object_deeply() {
    let mut expr = json_to_expr(json!({
        "lorem": {
            "ipsum": {
                "dolor": [
                    "sit",
                    "amet"
                ]
            }
        }
    }));

    PropMutator::mut_with(
      &mut expr,
      &["lorem", "ipsum", "dolor", "2"],
      |expr| *expr = json_to_expr(json!("consectetur adipiscing elit")),
      true,
    )
    .unwrap();

    let code = print_one(&expr, None, Some(Config::default().with_minify(true)));

    assert_eq!(
      DebugUsingDisplay(code.unwrap().as_str()),
      DebugUsingDisplay(
        r#"{"lorem":{"ipsum":{"dolor":["sit","amet","consectetur adipiscing elit"]}}}"#
      )
    );
  }

  #[test]
  fn test_mut_function() {
    let mut call = serde_json::from_str::<CallExpr>(
            r#"
            {
                "type": "CallExpression",
                "span": {
                  "start": 0,
                  "end": 130,
                  "ctxt": 0
                },
                "callee": {
                  "type": "Identifier",
                  "span": {
                    "start": 0,
                    "end": 3,
                    "ctxt": 1
                  },
                  "value": "jsx",
                  "optional": false
                },
                "arguments": [
                  {
                    "spread": null,
                    "expression": {
                      "type": "StringLiteral",
                      "span": {
                        "start": 4,
                        "end": 7,
                        "ctxt": 0
                      },
                      "value": "a",
                      "raw": "\"a\""
                    }
                  },
                  {
                    "spread": null,
                    "expression": {
                      "type": "ObjectExpression",
                      "span": {
                        "start": 9,
                        "end": 129,
                        "ctxt": 0
                      },
                      "properties": [
                        {
                          "type": "KeyValueProperty",
                          "key": {
                            "type": "StringLiteral",
                            "span": {
                              "start": 10,
                              "end": 17,
                              "ctxt": 0
                            },
                            "value": "props",
                            "raw": "\"props\""
                          },
                          "value": {
                            "type": "ObjectExpression",
                            "span": {
                              "start": 19,
                              "end": 128,
                              "ctxt": 0
                            },
                            "properties": [
                              {
                                "type": "KeyValueProperty",
                                "key": {
                                  "type": "StringLiteral",
                                  "span": {
                                    "start": 20,
                                    "end": 26,
                                    "ctxt": 0
                                  },
                                  "value": "href",
                                  "raw": "\"href\""
                                },
                                "value": {
                                  "type": "CallExpression",
                                  "span": {
                                    "start": 28,
                                    "end": 127,
                                    "ctxt": 0
                                  },
                                  "callee": {
                                    "type": "Identifier",
                                    "span": {
                                      "start": 28,
                                      "end": 32,
                                      "ctxt": 1
                                    },
                                    "value": "_url",
                                    "optional": false
                                  },
                                  "arguments": [
                                    {
                                      "spread": null,
                                      "expression": {
                                        "type": "StringLiteral",
                                        "span": {
                                          "start": 33,
                                          "end": 43,
                                          "ctxt": 0
                                        },
                                        "value": "external",
                                        "raw": "\"external\""
                                      }
                                    },
                                    {
                                      "spread": null,
                                      "expression": {
                                        "type": "NullLiteral",
                                        "span": {
                                          "start": 45,
                                          "end": 49,
                                          "ctxt": 0
                                        }
                                      }
                                    },
                                    {
                                      "spread": null,
                                      "expression": {
                                        "type": "StringLiteral",
                                        "span": {
                                          "start": 51,
                                          "end": 126,
                                          "ctxt": 0
                                        },
                                        "value": "https://en.wikipedia.org/wiki/The_quick_brown_fox_jumps_over_the_lazy_dog",
                                        "raw": "\"https://en.wikipedia.org/wiki/The_quick_brown_fox_jumps_over_the_lazy_dog\""
                                      }
                                    }
                                  ],
                                  "typeArguments": null
                                }
                              }
                            ]
                          }
                        }
                      ]
                    }
                  }
                ],
                "typeArguments": null
              }
            "#,
        ).unwrap();

    assert_eq!(
      PropMutator::mut_with(
        &mut call,
        &["1", "props", "href", "2",],
        |expr| *expr =
          Lit::from("https://fr.wikipedia.org/wiki/Portez_ce_vieux_whisky_au_juge_blond_qui_fume")
            .into(),
        false
      )
      .unwrap(),
      ()
    );

    let code = print_one(&call, None, Some(Config::default().with_minify(true)));

    assert_eq!(
      DebugUsingDisplay(code.unwrap().as_str()),
      DebugUsingDisplay(
        r#"jsx("a",{"props":{"href":_url("external",null,"https://fr.wikipedia.org/wiki/Portez_ce_vieux_whisky_au_juge_blond_qui_fume")}})"#
      )
    );
  }
}
