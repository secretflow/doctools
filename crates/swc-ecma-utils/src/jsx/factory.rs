use serde::{Deserialize, Serialize};
use swc_core::{
  atoms::Atom,
  common::{util::take::Take, Span},
  ecma::ast::{
    ArrayLit, CallExpr, Callee, Expr, ExprOrSpread, Ident, KeyValueProp, Lit, Null, ObjectLit,
    Prop, PropName, PropOrSpread, Str,
  },
};

use crate::{
  ast::{json_to_expr, PropMutator, PropMutatorResult},
  jsx_or_pass,
  span::with_span,
};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
#[serde(tag = "type", content = "name")]
pub enum JSXTagName {
  Intrinsic(Atom),
  Ident(Atom),
  Fragment,
}

impl JSXTagName {
  pub fn is_metadata(&self) -> bool {
    match self {
      JSXTagName::Intrinsic(name) => match name.as_str() {
        "base" | "link" | "meta" | "noscript" | "script" | "style" | "title" => true,
        _ => false,
      },
      _ => false,
    }
  }
}

impl From<&str> for JSXTagName {
  fn from(value: &str) -> Self {
    JSXTagName::Intrinsic(value.into())
  }
}

impl From<String> for JSXTagName {
  fn from(value: String) -> Self {
    JSXTagName::Intrinsic(value.into())
  }
}

impl From<Ident> for JSXTagName {
  fn from(value: Ident) -> Self {
    JSXTagName::Ident(value.sym)
  }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JSXFactory {
  #[serde(rename = "Fragment")]
  sym_fragment: Atom,
  #[serde(rename = "jsx")]
  sym_jsx: Atom,
  #[serde(rename = "jsxs")]
  sym_jsxs: Atom,
}

impl JSXFactory {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn with_jsx(mut self, jsx: &str) -> Self {
    self.sym_jsx = jsx.into();
    self
  }

  pub fn with_jsxs(mut self, jsxs: &str) -> Self {
    self.sym_jsxs = jsxs.into();
    self
  }

  pub fn with_fragment(mut self, fragment: &str) -> Self {
    self.sym_fragment = fragment.into();
    self
  }

  pub fn get_names(&self) -> [&str; 3] {
    [
      self.sym_fragment.as_str(),
      self.sym_jsx.as_str(),
      self.sym_jsxs.as_str(),
    ]
  }
}

impl JSXFactory {
  pub fn create<'a>(&'a self, name: &'a JSXTagName) -> JSXBuilder<'a> {
    JSXBuilder {
      factory: self,
      name,
      arg1: None,
      props: vec![],
      children: vec![],
    }
  }
}

impl JSXFactory {
  pub fn call_is_jsx(&self, call: &CallExpr) -> Option<JSXTagName> {
    match &call.callee {
      Callee::Expr(callee) => match &**callee {
        Expr::Ident(Ident { sym: caller, .. }) => {
          if caller == &self.sym_jsx || caller == &self.sym_jsxs {
            match call.args.get(0) {
              Some(ExprOrSpread { expr, .. }) => match &**expr {
                Expr::Lit(Lit::Str(Str { value, .. })) => {
                  Some(JSXTagName::Intrinsic(value.as_str().into()))
                }
                Expr::Ident(Ident { sym, .. }) => {
                  if sym.as_str() == self.sym_fragment.as_str() {
                    Some(JSXTagName::Fragment)
                  } else {
                    Some(JSXTagName::Ident(sym.as_str().into()))
                  }
                }
                _ => None,
              },
              _ => None,
            }
          } else {
            None
          }
        }
        _ => None,
      },
      _ => None,
    }
  }

  pub fn expr_is_jsx(&self, expr: &Box<Expr>) -> Option<JSXTagName> {
    match &**expr {
      Expr::Call(call) => self.call_is_jsx(call),
      _ => None,
    }
  }

  pub fn set_prop_with<F>(
    &self,
    call: &mut CallExpr,
    path: &[&str],
    mutator: &mut F,
  ) -> PropMutatorResult
  where
    F: FnMut(&mut Box<Expr>),
  {
    jsx_or_pass!(self, call, unreachable);

    let props = call
      .args
      .get_mut(1)
      .and_then(|a| Some(&mut a.expr))
      .expect("expected props in argument");

    PropMutator::mut_with(props, path, mutator, false)
  }

  pub fn set_children(&self, call: &mut CallExpr, path: &[&str], mut value: Box<Expr>) {
    jsx_or_pass!(self, call, ());

    let props = call
      .args
      .get_mut(1)
      .and_then(|a| Some(&mut a.expr))
      .expect("expected props in argument");

    let (mut children, count) = match *value {
      Expr::Array(ArrayLit { ref mut elems, .. }) => {
        if elems.len() > 1 {
          match path {
            ["children"] => {
              let count = elems.len();
              (value, count)
            }
            _ => (
              self
                .create(&JSXTagName::Fragment)
                .children(
                  elems
                    .into_iter()
                    .filter_map(|x| match x.as_mut() {
                      None => None,
                      Some(item) => Some(item.expr.take()),
                    })
                    .collect(),
                )
                .build()
                .into(),
              1,
            ),
          }
        } else if elems.len() == 1 {
          match elems.last_mut().unwrap() {
            Some(ref mut expr) => {
              if expr.spread.is_some() {
                (value, 2)
              } else {
                (expr.expr.take(), 1)
              }
            }
            None => unreachable!(),
          }
        } else {
          (
            Expr::Lit(Lit::Null(Null {
              span: Default::default(),
            }))
            .into(),
            1,
          )
        }
      }
      _ => (value, 1),
    };

    let func = call
      .callee
      .as_mut_expr()
      .and_then(|e| e.as_mut_ident())
      .expect("expected jsx or jsxs");

    match (path, count) {
      (["children"], 0 | 1) => func.sym = self.sym_jsx.as_str().into(),
      (["children"], _) => func.sym = self.sym_jsxs.as_str().into(),
      _ => {}
    }

    PropMutator::mut_with(
      props,
      path,
      &mut |expr| *expr = children.take().into(),
      true,
    )
    .unwrap();
  }

  pub fn replace_props<S>(&self, call: &mut CallExpr, value: S) -> Result<(), serde_json::Error>
  where
    S: Serialize,
  {
    let (_, props) = jsx_or_pass!(self, mut call, unreachable);

    let mut children: Option<Box<Expr>> = None;

    PropMutator::mut_with(
      props,
      &["children"],
      &mut |expr| {
        children.replace(expr.take());
      },
      false,
    )
    .ok();

    *props = *json_to_expr(serde_json::to_value(value)?);

    if let Some(children) = children {
      self.set_children(call, &["children"], children);
    }

    Ok(())
  }
}

impl Default for JSXFactory {
  fn default() -> Self {
    Self {
      sym_fragment: "Fragment".into(),
      sym_jsx: "jsx".into(),
      sym_jsxs: "jsxs".into(),
    }
  }
}

pub struct JSXBuilder<'factory> {
  factory: &'factory JSXFactory,
  name: &'factory JSXTagName,
  pub arg1: Option<Box<Expr>>,
  pub props: Vec<Box<Prop>>,
  pub children: Vec<ExprOrSpread>,
}

impl JSXBuilder<'_> {
  pub fn prop(mut self, key: &str, value: Box<Expr>, span: Option<Span>) -> Self {
    self.props.push(with_span(span)(
      (Prop::KeyValue(KeyValueProp {
        key: PropName::Str(Str {
          value: key.into(),
          span: Default::default(),
          raw: None,
        }),
        value,
      }))
      .into(),
    ));
    self
  }

  pub fn children(mut self, mut children: Vec<Box<Expr>>) -> Self {
    self.children.append(
      &mut children
        .drain(..)
        .map(|expr| expr.into())
        .collect::<Vec<_>>(),
    );
    self
  }

  pub fn build(mut self) -> CallExpr {
    let jsx = if self.children.len() > 1 {
      &*self.factory.sym_jsxs
    } else {
      &*self.factory.sym_jsx
    };

    let props = match self.arg1 {
      Some(props) => {
        if !(self.props.is_empty() || !self.children.is_empty()) {
          unreachable!("arg1 is set but props and children are not empty");
        }
        props
      }
      None => {
        let mut props = self.props;

        if self.children.len() > 1 {
          // { "children": [jsx(...), jsxs(...), ...] }
          props.push(
            Prop::from(KeyValueProp {
              key: PropName::Str("children".into()),
              value: ArrayLit {
                elems: self
                  .children
                  .drain(..)
                  .map(|expr| Some(expr.into()))
                  .collect(),
                span: Default::default(),
              }
              .into(),
            })
            .into(),
          )
        } else if self.children.len() == 1 {
          // { "children": jsx(...) }
          // { "children": null }
          let value = self.children.pop().unwrap();
          props.push(
            Prop::from(KeyValueProp {
              key: PropName::Str("children".into()),
              value: value.expr,
            })
            .into(),
          )
        }

        Expr::from(ObjectLit {
          props: props
            .into_iter()
            .map(|prop| PropOrSpread::Prop(prop.into()))
            .collect(),
          span: Default::default(),
        })
        .into()
      }
    };

    // jsx("tag", { ...attrs, children: jsx(...) })
    // jsxs("tag", { ...attrs, children: [jsx(...), jsxs(...), ...] })
    CallExpr {
      // jsx(
      callee: Callee::from(Box::from(Ident::from(jsx))),
      args: vec![
        match self.name {
          JSXTagName::Intrinsic(tag) => Expr::from(tag.as_str()).into(),
          JSXTagName::Ident(tag) => Expr::from(Ident::from(tag.as_str())).into(),
          JSXTagName::Fragment => {
            Expr::from(Ident::from(self.factory.sym_fragment.as_str())).into()
          }
        },
        props.into(),
      ],
      span: Default::default(),
      type_args: None,
    }
  }
}

#[macro_export]
macro_rules! props {
  ( $($key:literal = $value:expr),* ) => {
    vec![
    $(  swc_core::ecma::ast::Prop::KeyValue(swc_core::ecma::ast::KeyValueProp {
            key: swc_core::ecma::ast::PropName::Str($key.into()),
            value: swc_core::ecma::ast::Expr::from($value).into(),
        })
        .into(), )*
    ]
  };
  ($obj:expr) => {
    match *$obj {
      swc_core::ecma::ast::Expr::Object(obj) => obj
        .props
        .into_iter()
        .map(|prop| match prop {
            swc_core::ecma::ast::PropOrSpread::Prop(prop) => prop,
            swc_core::ecma::ast::PropOrSpread::Spread(_) => unreachable!(),
        })
        .collect(),
      _ => unreachable!(),
    }
  };
}

#[macro_export]
macro_rules! object_lit {
  ( $($key:literal = $value:expr),* ) => {
    swc_core::ecma::ast::ObjectLit {
        props: swc_ecma_utils::props!($($key = $value),*),
        span: Default::default(),
    }
  };
}

#[macro_export]
macro_rules! jsx_or_pass {
  ($factory:expr, $call:expr, unreachable) => {
    match $factory.call_is_jsx($call) {
      Some(elem) => (elem, &*$call.args.get(1).unwrap().expr),
      None => unreachable!(),
    }
  };
  ($factory:expr, mut $call:expr, unreachable) => {
    match $factory.call_is_jsx($call) {
      Some(elem) => (elem, &mut *$call.args.get_mut(1).unwrap().expr),
      None => unreachable!(),
    }
  };
  ($factory:expr, $call:expr, $ret:expr) => {
    match $factory.call_is_jsx($call) {
      Some(elem) => (elem, &*$call.args.get(1).unwrap().expr),
      None => return $ret,
    }
  };
  ($factory:expr, mut $call:expr, $ret:expr) => {
    match $factory.call_is_jsx($call) {
      Some(elem) => (elem, &mut *$call.args.get_mut(1).unwrap().expr),
      None => return $ret,
    }
  };
  ($visitor:ident, $factory:expr, $call:expr) => {{
    use swc_core::ecma::visit::VisitWith;
    match $factory.call_is_jsx($call) {
      Some(elem) => (elem, &*$call.args.get(1).unwrap().expr),
      None => {
        $call.visit_children_with($visitor);
        return;
      }
    }
  }};
  ($visitor:ident, $factory:expr, mut $call:expr) => {{
    use swc_core::ecma::visit::VisitMutWith;
    match $factory.call_is_jsx($call) {
      Some(elem) => (elem, &mut *$call.args.get_mut(1).unwrap().expr),
      None => {
        $call.visit_mut_children_with($visitor);
        return;
      }
    }
  }};
}

#[macro_export]
macro_rules! children_or_pass {
  ($prop:expr) => {{
    let prop = match $prop.key {
      swc_core::ecma::ast::PropName::Ident(swc_core::ecma::ast::Ident { ref sym, .. })
        if sym.as_str() == "children" =>
      {
        $prop
      }
      swc_core::ecma::ast::PropName::Str(swc_core::ecma::ast::Str { ref value, .. })
        if value.as_str() == "children" =>
      {
        $prop
      }
      _ => return,
    };
    &*prop.value
  }};
  (take $prop:expr) => {{
    let prop = match $prop.key {
      swc_core::ecma::ast::PropName::Ident(swc_core::ecma::ast::Ident { ref sym, .. })
        if sym.as_str() == "children" =>
      {
        $prop
      }
      swc_core::ecma::ast::PropName::Str(swc_core::ecma::ast::Str { ref value, .. })
        if value.as_str() == "children" =>
      {
        $prop
      }
      _ => return,
    };
    *prop.value.take()
  }};
}

#[cfg(test)]
mod tests {
  use swc_core::{
    ecma::{
      ast::{Expr, Ident},
      codegen,
    },
    testing::DebugUsingDisplay,
  };

  use crate::testing::print_one;

  use super::{JSXFactory, JSXTagName};

  #[test]
  fn test_fragment() {
    let jsx = JSXFactory::default();
    let elem = jsx.create(&JSXTagName::Fragment).build();
    let code = print_one(&elem, None, None).unwrap();
    assert_eq!(code, "jsx(Fragment, {})");
  }

  #[test]
  fn test_intrinsic() {
    let jsx = JSXFactory::default();
    let elem = jsx
      .create(&JSXTagName::Intrinsic("div".into()))
      .children(vec![Box::from(Expr::from(Ident::from("foo")))])
      .build();
    let code = print_one(
      &elem,
      None,
      Some(codegen::Config::default().with_minify(true)),
    );
    assert_eq!(
      DebugUsingDisplay(code.unwrap().as_str()),
      DebugUsingDisplay(r#"jsx("div",{"children":foo})"#)
    );
  }

  #[test]
  fn test_component() {
    let jsx = JSXFactory::default();
    let elem = jsx.create(&JSXTagName::Ident("Foo".into())).build();
    let code = print_one(
      &elem,
      None,
      Some(codegen::Config::default().with_minify(true)),
    );
    assert_eq!(
      DebugUsingDisplay(code.unwrap().as_str()),
      DebugUsingDisplay(r#"jsx(Foo,{})"#)
    );
  }

  #[test]
  fn test_jsxs() {
    let jsx = JSXFactory::default();
    let elem = jsx
      .create(&JSXTagName::Intrinsic("div".into()))
      .children(vec![
        jsx
          .create(&JSXTagName::Intrinsic("span".into()))
          .build()
          .into(),
        jsx
          .create(&JSXTagName::Intrinsic("span".into()))
          .build()
          .into(),
      ])
      .build();
    let code = print_one(
      &elem,
      None,
      Some(codegen::Config::default().with_minify(true)),
    );
    assert_eq!(
      DebugUsingDisplay(code.unwrap().as_str()),
      DebugUsingDisplay(r#"jsxs("div",{"children":[jsx("span",{}),jsx("span",{})]})"#)
    );
  }

  #[test]
  fn test_props() {
    let jsx = JSXFactory::default();
    let elem = jsx
      .create(&JSXTagName::Intrinsic("div".into()))
      .prop("className", "foo".into(), None)
      .prop("id", "bar".into(), None)
      .build();
    let code = print_one(
      &elem,
      None,
      Some(codegen::Config::default().with_minify(true)),
    );
    assert_eq!(
      DebugUsingDisplay(code.unwrap().as_str()),
      DebugUsingDisplay(r#"jsx("div",{"className":"foo","id":"bar"})"#)
    );
  }
}
