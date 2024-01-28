use serde::{Deserialize, Serialize};
use swc_core::{
  common::Span,
  ecma::ast::{ArrayLit, Expr},
};

use crate::span::with_span;

use super::factory::{JSXBuilder, JSXRuntime, JSXTagName};

#[derive(Debug)]
struct PropPath(Vec<String>);

#[derive(Debug)]
struct Children(Vec<Box<Expr>>);

#[derive(Debug)]
struct Context {
  parent: Box<Expr>,
  prop: PropPath,
  children: Children,
}

#[derive(Debug)]
enum LastElement {
  Head,
  Body,
  Context,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JSXDocument {
  pub head: Vec<Box<Expr>>,
  pub body: Vec<Box<Expr>>,
}

impl Default for JSXDocument {
  fn default() -> Self {
    Self {
      head: vec![],
      body: vec![],
    }
  }
}

#[derive(Debug)]
pub struct DocumentBuilder {
  runtime: JSXRuntime,

  state: Option<LastElement>,
  context: Vec<Context>,

  head: Children,
  body: Children,
}

impl DocumentBuilder {
  pub fn element(
    &mut self,
    name: &JSXTagName,
    props: Option<Box<Expr>>,
    span: Option<Span>,
  ) -> &mut Self {
    let mut builder = self.runtime.create(name);
    if let Some(arg1) = props {
      builder.arg1 = Some(arg1)
    }
    let elem = builder.build().into();
    let elem = with_span(span)(elem);
    self.push(elem);
    self
  }

  pub fn enter(&mut self, path: &[&str]) -> &mut Self {
    let parent = self.pop();
    self.context.push(Context {
      parent,
      prop: PropPath(path.iter().map(|s| s.to_string()).collect()),
      children: Children(vec![]),
    });
    self
  }

  pub fn value(&mut self, value: Box<Expr>) -> &mut Self {
    self.push(value);
    self
  }

  pub fn exit(&mut self) -> &mut Self {
    let Context {
      mut parent,
      prop,
      children,
    } = match self.context.pop() {
      Some(v) => v,
      None => return self,
    };

    let children = Box::from(Expr::from(ArrayLit {
      elems: children.0.into_iter().map(|x| Some(x.into())).collect(),
      span: Default::default(),
    }));

    let props = self
      .runtime
      .as_mut_jsx_props(parent.as_mut_call().unwrap())
      .unwrap();

    self
      .runtime
      .mut_or_set_prop(props, &prop.as_strs()[..], |expr| *expr = children);

    self.push(parent);
    self
  }

  pub fn flush(&mut self) {
    while self.context.len() > 0 {
      self.exit();
    }
  }

  fn pop(&mut self) -> Box<Expr> {
    match self.state {
      Some(LastElement::Head) => self.head.0.pop(),
      Some(LastElement::Body) => self.body.0.pop(),
      Some(LastElement::Context) => self.context.last_mut().and_then(|ctx| ctx.children.0.pop()),
      None => None,
    }
    .expect("no element to enter")
  }

  fn push(&mut self, value: Box<Expr>) {
    let kind = match *value {
      Expr::Call(ref call) => self.runtime.as_jsx(call).and_then(|t| Some(t.0)),
      _ => None,
    };

    match kind {
      Some(ref elem) if elem.is_metadata() => {
        self.head.0.push(value);
        self.state = Some(LastElement::Head);
      }
      _ => match self.context.last_mut() {
        Some(Context { children, .. }) => {
          children.0.push(value);
          self.state = if kind.is_some() {
            Some(LastElement::Context)
          } else {
            None
          };
        }
        None => {
          self.body.0.push(value);
          self.state = if kind.is_some() {
            Some(LastElement::Body)
          } else {
            None
          };
        }
      },
    }
  }

  pub fn new(runtime: JSXRuntime) -> Self {
    Self {
      runtime,
      state: Some(LastElement::Body),
      context: vec![],
      head: Children(vec![]),
      body: Children(vec![]),
    }
  }

  pub fn declare(self) -> JSXDocument {
    JSXDocument {
      head: self.head.0,
      body: self.body.0,
    }
  }

  pub fn noop(builder: JSXBuilder) -> JSXBuilder {
    builder
  }
}

impl PropPath {
  fn as_strs(&self) -> Vec<&str> {
    self.0.iter().map(String::as_str).collect()
  }
}
