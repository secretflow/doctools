use swc_core::{
  common::util::take::Take,
  ecma::{
    ast::{
      ArrayLit, CallExpr, Expr, ExprOrSpread, Ident, KeyValueProp, Lit, Number, ObjectLit, Prop,
      PropName, PropOrSpread, Str,
    },
    visit::{VisitMut, VisitMutWith},
  },
};

use crate::json::null;

#[derive(Debug)]
pub enum PropVisitorError {
  InvalidPath(String),
  NotFound,
}

struct PropVisitor<'path, T> {
  path: &'path [Lit],
  ensure: bool,
  idx: usize,
  callback: &'path mut dyn FnMut(&mut Box<Expr>) -> T,
  result: Result<T, PropVisitorError>,
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

impl<T> PropVisitor<'_, T> {
  fn traverse<N: VisitMutWith<Self>>(&mut self, expr: &mut N) {
    self.idx += 1;
    expr.visit_mut_with(self);
  }

  fn found(&mut self, found: &mut Box<Expr>) {
    self.result = Ok((self.callback)(found));
  }

  fn process_array_elems(&mut self, elems: &mut Vec<Option<ExprOrSpread>>) {
    let subscript = self.path.get(self.idx).expect("index out of bounds");

    let index = match subscript {
      Lit::Num(num) => match f64_to_usize(num.value) {
        Some(idx) => idx,
        None => item_getter_error!(
          self,
          PropVisitorError::InvalidPath,
          "invalid array index {}",
          num.value
        ),
      },
      Lit::Str(str) => match str.value.parse::<usize>() {
        Ok(idx) => idx,
        Err(_) => item_getter_error!(
          self,
          PropVisitorError::InvalidPath,
          "invalid array index {}",
          str.value
        ),
      },
      _ => item_getter_error!(
        self,
        PropVisitorError::InvalidPath,
        "array index should be Lit::Num, found {:?}",
        subscript
      ),
    };

    let mut set_default = |arr: &mut Vec<Option<ExprOrSpread>>| {
      if self.ensure {
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
        item_getter_error!(self, PropVisitorError::NotFound)
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
            if self.ensure {
              *next = ensure_object();
              self.traverse(next.as_mut().unwrap().expr.as_mut());
            } else {
              item_getter_error!(self, PropVisitorError::NotFound)
            }
          }
        },
        None => {
          if self.ensure {
            elems.resize_with(index + 1, || None);
            elems[index] = ensure_object();
            self.traverse(elems[index].as_mut().unwrap().expr.as_mut());
          } else {
            item_getter_error!(self, PropVisitorError::NotFound)
          }
        }
      };
    }
  }

  fn process_object_props(&mut self, props: &'_ mut Vec<PropOrSpread>) {
    let subscript = self.path.get(self.idx).expect("index out of bounds");

    let key = match subscript {
      Lit::Str(key) => key.value.to_string(),
      Lit::Num(num) => num.value.to_string(),
      _ => item_getter_error!(
        self,
        PropVisitorError::InvalidPath,
        "object keys should be strings or numbers, found {:?}",
        subscript
      ),
    };

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
          if self.ensure {
            props.push(
              PropOrSpread::Prop(
                Prop::from(KeyValueProp {
                  key: PropName::Str(key.as_str().into()),
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
            item_getter_error!(self, PropVisitorError::NotFound)
          }
        }
      }
    } else {
      match existing {
        Some(prop) => self.traverse(&mut prop.value),
        None => {
          if self.ensure {
            props.push(
              PropOrSpread::Prop(
                Prop::from(KeyValueProp {
                  key: PropName::Str(key.as_str().into()),
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
            item_getter_error!(self, PropVisitorError::NotFound)
          }
        }
      }
    }
  }
}

impl<T> VisitMut for PropVisitor<'_, T> {
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
}

pub fn mut_call_by_path<T>(
  call: &mut CallExpr,
  path: &[Lit],
  mut callback: impl FnMut(&mut Box<Expr>) -> T,
) -> Result<T, PropVisitorError> {
  let mut setter = PropVisitor {
    path,
    idx: 0,
    ensure: false,
    // This cannot be boxed because of the lifetime.
    //
    // With Box<dyn FnMut>, rustc asks for callback to be 'static
    // which limits what kind of closures can be passed
    // (essentially cannot borrow anything whose lifetime is shorter than 'static).
    //
    // I still don't know why Box requires 'static though.
    callback: &mut callback,
    result: Err(PropVisitorError::NotFound),
  };
  call.visit_mut_with(&mut setter);
  setter.result
}

pub fn mut_ast_by_path<T>(
  ast: &mut Expr,
  path: &[Lit],
  mut callback: impl FnMut(&mut Box<Expr>) -> T,
) -> Result<T, PropVisitorError> {
  let mut setter = PropVisitor {
    path,
    idx: 0,
    ensure: false,
    callback: &mut callback,
    result: Err(PropVisitorError::NotFound),
  };
  if let Some(object) = ast.as_mut_object() {
    object.visit_mut_with(&mut setter)
  } else if let Some(array) = ast.as_mut_array() {
    array.visit_mut_with(&mut setter)
  } else if let Some(call) = ast.as_mut_call() {
    call.visit_mut_with(&mut setter)
  } else {
    unreachable!("unsupported expression {:?}", ast)
  };
  setter.result
}

pub fn set_ast_by_path(
  ast: &mut Expr,
  path: &[Lit],
  mut value: Box<Expr>,
) -> Result<(), PropVisitorError> {
  let mut setter = PropVisitor {
    path,
    idx: 0,
    ensure: true,
    callback: &mut move |expr: &mut Box<Expr>| *expr = value.take().into(),
    result: Ok(()),
  };
  if let Some(object) = ast.as_mut_object() {
    object.visit_mut_with(&mut setter)
  } else if let Some(array) = ast.as_mut_array() {
    array.visit_mut_with(&mut setter)
  } else if let Some(call) = ast.as_mut_call() {
    call.visit_mut_with(&mut setter)
  } else {
    unreachable!("unsupported expression {:?}", ast)
  };
  setter.result
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

fn f64_to_usize(f: f64) -> Option<usize> {
  if f.is_finite() && f.fract() == 0.0 && f >= 0.0 && f <= usize::MAX as f64 {
    Some(f as usize)
  } else {
    None
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

  use super::{mut_ast_by_path, set_ast_by_path};
  use crate::{json::json_expr, testing::print_one};

  #[test]
  fn test_set_object() {
    let mut expr = json_expr(json!({}));

    set_ast_by_path(
      &mut expr,
      &[Lit::from("children")],
      Box::from(json_expr(json!([]))),
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

    set_ast_by_path(
      &mut expr,
      &[
        Lit::from("lorem"),
        Lit::from("ipsum"),
        Lit::from("dolor"),
        Lit::from(2),
      ],
      Box::from(json_expr(json!("consectetur adipiscing elit"))),
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
    let call = serde_json::from_str::<CallExpr>(
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

    let mut expr = Box::from(call);

    assert_eq!(
      mut_ast_by_path(
        &mut expr,
        &[
          Lit::from(1),
          Lit::from("props"),
          Lit::from("href"),
          Lit::from("2"),
        ],
        |expr| {
          *expr =
            Lit::from("https://fr.wikipedia.org/wiki/Portez_ce_vieux_whisky_au_juge_blond_qui_fume")
              .into()
        },
      )
      .unwrap(),
      ()
    );

    let code = print_one(&expr, None, Some(Config::default().with_minify(true)));

    assert_eq!(
      DebugUsingDisplay(code.unwrap().as_str()),
      DebugUsingDisplay(
        r#"jsx("a",{"props":{"href":_url("external",null,"https://fr.wikipedia.org/wiki/Portez_ce_vieux_whisky_au_juge_blond_qui_fume")}})"#
      )
    );
  }
}
