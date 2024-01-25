use serde::{Deserialize, Serialize};
use swc_core::{
  atoms::Atom,
  common::{util::take::Take, Span},
  ecma::ast::{
    ArrayLit, CallExpr, Callee, Expr, ExprOrSpread, Ident, KeyValueProp, Lit, ObjectLit, Prop,
    PropName, PropOrSpread, Str,
  },
};

use crate::{
  ast::{PropMutator, SelectNode},
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
pub struct JSXRuntime {
  #[serde(rename = "Fragment")]
  sym_fragment: Atom,
  #[serde(rename = "jsx")]
  sym_jsx: Atom,
  #[serde(rename = "jsxs")]
  sym_jsxs: Atom,
}

impl JSXRuntime {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn aliased(jsx: &str, jsxs: &str, fragment: &str) -> Self {
    Self {
      sym_fragment: fragment.into(),
      sym_jsx: jsx.into(),
      sym_jsxs: jsxs.into(),
    }
  }

  pub fn jsx(&self) -> Callee {
    Callee::Expr(Expr::Ident(Ident::from(&*self.sym_jsx)).into())
  }

  pub fn jsxs(&self) -> Callee {
    Callee::Expr(Expr::Ident(Ident::from(&*self.sym_jsxs)).into())
  }

  pub fn get_names(&self) -> [&str; 3] {
    [
      self.sym_fragment.as_str(),
      self.sym_jsx.as_str(),
      self.sym_jsxs.as_str(),
    ]
  }
}

impl JSXRuntime {
  pub fn create<'a>(&'a self, name: &'a JSXTagName) -> JSXBuilder<'a> {
    JSXBuilder {
      runtime: self,
      name,
      arg1: None,
      props: vec![],
      children: vec![],
    }
  }
}

impl JSXRuntime {
  fn as_jsx_tag(&self, call: &CallExpr) -> Option<JSXTagName> {
    match &call.callee {
      Callee::Expr(callee) => match &**callee {
        Expr::Ident(Ident { sym: caller, .. }) => {
          if caller == &self.sym_jsx || caller == &self.sym_jsxs {
            match call.args.get(0) {
              Some(ExprOrSpread { expr, .. }) => match &**expr {
                Expr::Lit(Lit::Str(Str { value, .. })) => {
                  Some(JSXTagName::Intrinsic((&**value).into()))
                }
                Expr::Ident(Ident { sym, .. }) => {
                  if &**sym == &*self.sym_fragment {
                    Some(JSXTagName::Fragment)
                  } else {
                    Some(JSXTagName::Ident((&**sym).into()))
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

  pub fn as_jsx<'ast>(&self, call: &'ast CallExpr) -> Option<(JSXTagName, &'ast ObjectLit)> {
    let tag_name = self.as_jsx_tag(call);

    let props = call.args.get(1).and_then(|a| a.expr.as_object());

    match (tag_name, props) {
      (Some(tag_name), Some(props)) => Some((tag_name, props)),
      _ => None,
    }
  }

  pub fn as_mut_jsx_tag<'ast>(&self, call: &'ast mut CallExpr) -> Option<&'ast mut Expr> {
    call.args.get_mut(0).and_then(|a| Some(&mut *a.expr))
  }

  pub fn as_mut_jsx_props<'ast>(&self, call: &'ast mut CallExpr) -> Option<&'ast mut ObjectLit> {
    call.args.get_mut(1).and_then(|a| a.expr.as_mut_object())
  }

  pub fn get_prop<'ast>(&self, props: &'ast ObjectLit, keys: &[&str]) -> SelectNode<'ast> {
    let mut keys = keys.iter();
    let key = match keys.next() {
      None => unreachable!(),
      Some(key) => *key,
    };
    let mut selector = SelectNode::from_key(props, key);
    loop {
      match keys.next() {
        None => return selector,
        Some(key) => selector = selector.key(key),
      }
    }
  }

  pub fn mut_prop<'ast, F, T>(
    &self,
    props: &'ast mut ObjectLit,
    path: &[&str],
    mutator: F,
  ) -> Option<T>
  where
    F: FnOnce(&mut Box<Expr>) -> T + 'ast,
  {
    PropMutator::mut_with(props, path, mutator, false)
  }

  pub fn mut_or_set_prop<'ast, F, T>(
    &self,
    props: &'ast mut ObjectLit,
    path: &[&str],
    mutator: F,
  ) -> Option<T>
  where
    F: FnOnce(&mut Box<Expr>) -> T + 'ast,
  {
    PropMutator::mut_with(props, path, mutator, true)
  }

  pub fn take_prop(&self, props: &mut ObjectLit, path: &[&str]) -> Option<Expr> {
    PropMutator::mut_with(props, path, |expr| *expr.take(), false)
  }
}

impl Default for JSXRuntime {
  fn default() -> Self {
    Self {
      sym_fragment: "Fragment".into(),
      sym_jsx: "jsx".into(),
      sym_jsxs: "jsxs".into(),
    }
  }
}

pub struct JSXBuilder<'factory> {
  runtime: &'factory JSXRuntime,
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

  pub fn arg1(mut self, arg1: Box<Expr>) -> Self {
    self.arg1 = Some(arg1);
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
      &*self.runtime.sym_jsxs
    } else {
      &*self.runtime.sym_jsx
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
            Expr::from(Ident::from(self.runtime.sym_fragment.as_str())).into()
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
macro_rules! jsx_or_return {
  ($factory:expr, $call:expr) => {{
    match $factory.as_jsx($call) {
      Some((elem, props)) => (elem, props),
      None => {
        return;
      }
    }
  }};
}

#[macro_export]
macro_rules! jsx_or_continue_visit {
  ($visitor:ident, $factory:expr, $call:expr) => {{
    use swc_core::ecma::visit::VisitWith as _;
    match $factory.as_jsx($call) {
      Some((elem, props)) => (elem, props),
      None => {
        $call.visit_children_with($visitor);
        return;
      }
    }
  }};
  ($visitor:ident, $factory:expr, mut $call:expr) => {{
    use swc_core::ecma::visit::VisitMutWith as _;
    match $factory.as_jsx($call) {
      Some((elem, props)) => (elem, props),
      None => {
        $call.visit_mut_children_with($visitor);
        return;
      }
    }
  }};
}

#[macro_export]
macro_rules! continue_visit {
  ($visitor:ident, $call:expr) => {{
    use swc_core::ecma::visit::VisitWith as _;
    $call.visit_children_with($visitor);
    return;
  }};
  ($visitor:ident, mut $call:expr) => {{
    use swc_core::ecma::visit::VisitMutWith as _;
    $call.visit_mut_children_with($visitor);
    return;
  }};
}
