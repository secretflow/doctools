use serde::{Deserialize, Serialize};
use swc_core::{
  common::Span,
  ecma::ast::{ArrayLit, Expr, Ident},
};

use crate::span::with_span;

use super::factory::{JSXBuilder, JSXElement, JSXFactory};

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
pub struct JSXSnippet {
  pub name: Ident,
  pub tree: Box<Expr>,
  pub html_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JSXDocument {
  pub head: Box<Expr>,
  pub body: Box<Expr>,
  pub snippets: Vec<JSXSnippet>,
}

#[derive(Debug)]
pub struct DocumentBuilder {
  factory: JSXFactory,

  state: Option<LastElement>,
  context: Vec<Context>,

  head: Children,
  body: Children,
  snippets: Vec<JSXSnippet>,
}

impl DocumentBuilder {
  pub fn element(
    &mut self,
    name: &JSXElement,
    mut build: impl FnMut(JSXBuilder) -> JSXBuilder,
    span: Option<Span>,
  ) -> &mut Self {
    let builder = self.factory.create(name);
    let elem = build(builder).build().into();
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
    self.factory.set_prop(
      &mut parent.as_mut_call().unwrap(),
      &prop.as_strs()[..],
      children,
    );
    self.push(parent);
    self
  }

  pub fn id(&mut self, id: String) -> &mut Self {
    let tree = self.pop();
    let name = self.snippet_name();
    self.element(
      &Ident::from(name.as_str()).into(),
      |mut builder| {
        builder.arg1 = Some(Ident::from("props").into());
        builder
      },
      None,
    );
    self.snippets.push(JSXSnippet {
      html_id: id,
      name: Ident::from(name.as_str()),
      tree,
    });
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
    let kind = self.factory.expr_is_jsx(&value);

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

  fn snippet_name(&self) -> String {
    format!("$snippet{}", self.snippets.len())
  }

  pub fn new(jsx: JSXFactory) -> Self {
    Self {
      factory: jsx,
      state: Some(LastElement::Body),
      context: vec![],
      head: Children(vec![]),
      body: Children(vec![]),
      snippets: vec![],
    }
  }

  pub fn declare(self) -> JSXDocument {
    let wrap_tree = |elements: Vec<Box<Expr>>| {
      if elements.len() == 1 {
        elements.into_iter().next().unwrap()
      } else {
        self
          .factory
          .create(&JSXElement::Fragment)
          .children(elements)
          .build()
          .into()
      }
    };

    let head = wrap_tree(self.head.0);
    let body = wrap_tree(self.body.0);

    JSXDocument {
      head,
      body,
      snippets: self.snippets,
    }
  }
}

impl PropPath {
  fn as_strs(&self) -> Vec<&str> {
    self.0.iter().map(String::as_str).collect()
  }
}
