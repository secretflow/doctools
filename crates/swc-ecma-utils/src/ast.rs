use swc_core::{
  atoms::Atom,
  ecma::{
    ast::{
      ArrayLit, CallExpr, Expr, ExprOrSpread, Ident, KeyValueProp, Number, ObjectLit, Prop,
      PropName, PropOrSpread, Str,
    },
    visit::{VisitMut, VisitMutWith},
  },
};

use crate::json::null;

#[derive(Debug)]
#[must_use]
pub enum PropLocatorError {
  NotFound,
  InvalidPath(String),
}

pub type PropLocatorResult = Result<(), PropLocatorError>;

pub struct PropLocator<'callback> {
  path: Vec<Atom>,
  set_intermediate_path: bool,
  idx: usize,
  callback: &'callback mut dyn FnMut(&mut Box<Expr>),
  result: PropLocatorResult,
}

macro_rules! item_getter_error {
  ($this:ident, $error:path, $msg:literal $(, $arg:expr)*) => {{
    $this.result = Err($error(format!($msg, $($arg),*).to_string()));
    return;
  }};
  ($this:ident, $error:path) => {{
    $this.result = Err($error);
    return;
  }};
}

impl PropLocator<'_> {
  fn traverse<N: VisitMutWith<Self>>(&mut self, expr: &mut N) {
    self.idx += 1;
    expr.visit_mut_with(self);
  }

  fn found(&mut self, found: &mut Box<Expr>) {
    (self.callback)(found);
    self.result = Ok(());
  }

  fn process_array_elems(&mut self, elems: &mut Vec<Option<ExprOrSpread>>) {
    let subscript = self.path.get(self.idx).expect("index out of bounds");

    let index = match subscript.parse::<usize>() {
      Ok(idx) => idx,
      Err(_) => item_getter_error!(
        self,
        PropLocatorError::InvalidPath,
        "invalid array index {}",
        subscript
      ),
    };

    let mut set_default = |arr: &mut Vec<Option<ExprOrSpread>>| {
      if self.set_intermediate_path {
        if index >= arr.len() {
          arr.resize_with(index + 1, || None);
        }
        arr[index] = Some(ExprOrSpread {
          spread: None,
          expr: {
            let mut new = Box::from(null());
            (self.callback)(&mut new);
            new
          },
        })
      } else {
        item_getter_error!(self, PropLocatorError::NotFound)
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

    if self.idx == self.path.len() - 1 {
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
              item_getter_error!(self, PropLocatorError::NotFound)
            }
          }
        },
        None => {
          if self.set_intermediate_path {
            elems.resize_with(index + 1, || None);
            elems[index] = ensure_object();
            self.traverse(elems[index].as_mut().unwrap().expr.as_mut());
          } else {
            item_getter_error!(self, PropLocatorError::NotFound)
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
                    (self.callback)(&mut new);
                    new
                  },
                })
                .into(),
              )
              .into(),
            )
          } else {
            item_getter_error!(self, PropLocatorError::NotFound)
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
            item_getter_error!(self, PropLocatorError::NotFound)
          }
        }
      }
    }
  }
}

impl VisitMut for PropLocator<'_> {
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

impl PropLocator<'_> {
  pub fn mut_with<E, F>(
    ast: &mut E,
    path: &[&str],
    callback: &mut F,
    set_default: bool,
  ) -> PropLocatorResult
  where
    E: for<'callback> VisitMutWith<PropLocator<'callback>>,
    F: FnMut(&mut Box<Expr>),
  {
    let mut visitor = PropLocator {
      path: path.iter().map(|p| (*p).into()).collect(),
      set_intermediate_path: set_default,
      idx: 0,
      result: Err(PropLocatorError::NotFound),
      callback,
    };
    ast.visit_mut_with(&mut visitor);
    match visitor.result {
      Err(PropLocatorError::NotFound) => {
        if set_default {
          Ok(())
        } else {
          visitor.result
        }
      }
      _ => visitor.result,
    }
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
  use serde_json::json;
  use swc_core::{
    ecma::{
      ast::{CallExpr, Lit},
      codegen::Config,
    },
    testing::DebugUsingDisplay,
  };

  use super::PropLocator;
  use crate::{json::json_expr, testing::print_one};

  #[test]
  fn test_set_object() {
    let mut expr = json_expr(json!({}));

    PropLocator::mut_with(
      &mut expr,
      &["children"],
      &mut |expr| *expr = json_expr(json!([])),
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
    let mut expr = json_expr(json!({
        "lorem": {
            "ipsum": {
                "dolor": [
                    "sit",
                    "amet"
                ]
            }
        }
    }));

    PropLocator::mut_with(
      &mut expr,
      &["lorem", "ipsum", "dolor", "2"],
      &mut |expr| *expr = json_expr(json!("consectetur adipiscing elit")),
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
      PropLocator::mut_with(
        &mut call,
        &["1", "props", "href", "2",],
        &mut |expr| *expr =
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
