use std::marker::PhantomData;

use serde::{Deserialize, Serialize};
use swc_core::{
  common::{util::take::Take, Span, Spanned as _},
  ecma::ast::{
    ArrayLit, CallExpr, Decl, ExportDecl, Expr, ExprOrSpread, Ident, Lit, Module, ModuleDecl,
    ModuleItem, ObjectLit, Str, VarDecl, VarDeclKind, VarDeclarator,
  },
};

use crate::{
  collections::{Mapping, MutableMapping, MutableSequence},
  span::with_span,
};

use super::{create_element, create_fragment, jsx_mut, runtime::JSXRuntime, JSXElementMut};

#[derive(Debug)]
struct Context {
  parent: Expr,
  prop: Vec<String>,
  children: Vec<Expr>,
}

#[derive(Debug)]
enum LastElement {
  Head,
  Body,
  Context,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JSXDocument {
  pub head: Vec<Expr>,
  pub body: Vec<Expr>,
}

impl Default for JSXDocument {
  fn default() -> Self {
    Self {
      head: vec![],
      body: vec![],
    }
  }
}

impl JSXDocument {
  pub fn to_fragment<R: JSXRuntime>(self) -> CallExpr {
    let mut elem = create_fragment::<R>();

    let mut children = ArrayLit::dummy();
    children.extend(&mut self.head.into_iter());
    children.extend(&mut self.body.into_iter());

    jsx_mut::<R>(&mut elem)
      .get_props_mut()
      .set_item("children", children.into());

    elem
  }
}

#[derive(Debug)]
pub struct DocumentBuilder<R: JSXRuntime> {
  state: LastElement,
  context: Vec<Context>,

  head: Vec<Expr>,
  body: Vec<Expr>,

  jsx: PhantomData<R>,
}

impl<R: JSXRuntime> DocumentBuilder<R> {
  pub fn element(
    &mut self,
    component: Option<Expr>,
    props: Option<ObjectLit>,
    span: Option<Span>,
  ) -> &mut Self {
    let mut elem = match component {
      Some(component) => create_element::<R>(component),
      None => create_fragment::<R>(),
    };

    let mut props = props;
    let mut props = props.drain().collect::<Vec<_>>();

    props.reverse();

    jsx_mut::<R>(&mut elem)
      .get_props_mut()
      .update_from(&mut props.drain(..));

    self.push(with_span(span)(elem.into()));
    self
  }

  pub fn enter(&mut self, path: &[&str]) -> &mut Self {
    let parent = self.pop();
    self.context.push(Context {
      parent,
      prop: path.iter().map(|s| s.to_string()).collect(),
      children: vec![],
    });
    self
  }

  pub fn value(&mut self, value: Expr) -> &mut Self {
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

    let children = {
      let mut array = ArrayLit::dummy();
      array.extend(&mut children.into_iter());
      array
    };

    jsx_mut::<R>(&mut parent.as_mut_call().unwrap())
      .get_props_mut()
      .set_item_at_path(prop.into_iter(), children.into())
      .unwrap();

    let parent = with_span(Some(parent.span()))(parent);

    self.push(parent);
    self
  }

  pub fn flush(&mut self) {
    while self.context.len() > 0 {
      self.exit();
    }
  }

  fn pop(&mut self) -> Expr {
    match self.state {
      LastElement::Head => self.head.pop(),
      LastElement::Body => self.body.pop(),
      LastElement::Context => self.context.last_mut().and_then(|ctx| ctx.children.pop()),
    }
    .expect("should have an element to enter")
  }

  fn push(&mut self, value: Expr) {
    let is_metadata = {
      if let Some(call) = value.as_call() {
        call.get_item(1usize).and_then(|expr| {
          expr.as_lit().and_then(|lit| match lit {
            Lit::Str(Str { value, .. }) => match &**value {
              "base" | "link" | "meta" | "noscript" | "script" | "style" | "title" => Some(()),
              _ => None,
            },
            _ => None,
          })
        })
      } else {
        None
      }
    }
    .is_some();

    if is_metadata {
      self.head.push(value);
      self.state = LastElement::Head;
    } else {
      match self.context.last_mut() {
        Some(Context { children, .. }) => {
          children.push(value);
          self.state = LastElement::Context;
        }
        None => {
          self.body.push(value);
          self.state = LastElement::Body;
        }
      }
    }
  }

  pub fn new() -> Self {
    Self {
      jsx: PhantomData,
      state: LastElement::Body,
      context: vec![],
      head: vec![],
      body: vec![],
    }
  }

  pub fn to_document(self) -> JSXDocument {
    JSXDocument {
      head: self.head,
      body: self.body,
    }
  }
}

impl JSXDocument {
  pub fn to_module(mut self) -> Module {
    Module {
      span: Default::default(),
      body: vec![ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
        span: Default::default(),
        decl: Decl::Var(
          VarDecl {
            span: Default::default(),
            kind: VarDeclKind::Const,
            declare: false,
            decls: vec![
              VarDeclarator {
                definite: false,
                span: Default::default(),
                name: Ident::from("head").into(),
                init: Some(
                  Expr::from(ArrayLit {
                    elems: self
                      .head
                      .drain(..)
                      .map(|expr| {
                        Some(ExprOrSpread {
                          expr: expr.into(),
                          spread: None,
                        })
                      })
                      .collect(),
                    span: Default::default(),
                  })
                  .into(),
                ),
              },
              VarDeclarator {
                definite: false,
                span: Default::default(),
                name: Ident::from("body").into(),
                init: Some(
                  Expr::from(ArrayLit {
                    elems: self
                      .body
                      .drain(..)
                      .map(|expr| {
                        Some(ExprOrSpread {
                          expr: expr.into(),
                          spread: None,
                        })
                      })
                      .collect(),
                    span: Default::default(),
                  })
                  .into(),
                ),
              },
            ],
          }
          .into(),
        ),
      }))],
      shebang: None,
    }
  }
}
