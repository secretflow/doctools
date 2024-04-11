use std::{fmt::Debug, marker::PhantomData};

use serde::{Deserialize, Serialize};
use swc_core::{
  common::{util::take::Take, Span, Spanned},
  ecma::ast::{
    ArrayLit, CallExpr, Decl, ExportDecl, Expr, ExprOrSpread, Ident, Lit, Module, ModuleDecl,
    ModuleItem, ObjectLit, Str, VarDecl, VarDeclKind, VarDeclarator,
  },
};

use crate::{
  collections::{Mapping, MutableMapping, MutableSequence},
  ecma::{repack_expr, RepackError},
  Array,
};

use super::{JSXElement, JSXElementMut, JSXRuntime, JSXTagDef};

#[derive(Debug, thiserror::Error)]
pub enum JSXBuilderError {
  #[error("cannot serialize value as AST: {0}")]
  RepackError(#[source] RepackError),
  #[error("invalid tag")]
  InvalidTag,
  #[error("invalid props")]
  InvalidProps,
  #[error("invalid children")]
  InvalidChildren,
}

pub struct JSXBuilder<R: JSXRuntime> {
  call: CallExpr,
  err: Option<JSXBuilderError>,
  jsx: PhantomData<R>,
}

impl<R: JSXRuntime> JSXBuilder<R> {
  pub fn tag(mut self, tag: impl JSXTagDef) -> Self {
    match self.err {
      Some(_) => {}
      None => match self.call.as_mut_arg0::<R>() {
        None => {
          self.err = Some(JSXBuilderError::InvalidTag);
        }
        Some(arg0) => {
          *arg0 = tag.to_expr::<R>(arg0.span());
        }
      },
    }
    self
  }

  pub fn prop<T: Serialize>(mut self, span: Span, name: &str, value: &T) -> Self {
    match self.err {
      Some(_) => {}
      None => match repack_expr(span, value) {
        Err(err) => {
          self.err = Some(JSXBuilderError::RepackError(err));
        }
        Ok(value) => match self.call.as_mut_jsx_props::<R>() {
          None => {
            self.err = Some(JSXBuilderError::InvalidProps);
          }
          Some(props) => {
            props.set_item(name, value);
          }
        },
      },
    }
    self
  }

  pub fn prop2(mut self, name: &str, value: Expr) -> Self {
    match self.err {
      Some(_) => {}
      None => match self.call.as_mut_jsx_props::<R>() {
        None => {
          self.err = Some(JSXBuilderError::InvalidProps);
        }
        Some(props) => {
          props.set_item(name, value);
        }
      },
    }
    self
  }

  pub fn child(mut self, child: Expr) -> Self {
    match self.err {
      Some(_) => {}
      None => match self.call.as_mut_jsx_props::<R>() {
        None => {
          self.err = Some(JSXBuilderError::InvalidChildren);
        }
        Some(props) => match props.get_item_mut("children") {
          None => {
            props.set_item("children", child);
          }
          Some(children) => match children {
            Expr::Array(children) => {
              children.append(child);
            }
            children => {
              *children = Array!(children.take(), child).into();
            }
          },
        },
      },
    }
    self
  }

  pub fn props<T: Serialize>(mut self, span: Span, props: &T) -> Self {
    match self.err {
      Some(_) => {}
      None => match repack_expr(span, props) {
        Err(err) => {
          self.err = Some(JSXBuilderError::RepackError(err));
        }
        Ok(Expr::Object(props)) => match self.call.as_mut_jsx_props::<R>() {
          None => {
            self.err = Some(JSXBuilderError::InvalidProps);
          }
          Some(old) => {
            *old = props;
          }
        },
        Ok(_) => self.err = Some(JSXBuilderError::InvalidProps),
      },
    }
    self
  }

  pub fn arg1(mut self, props: ObjectLit) -> Self {
    match self.err {
      Some(_) => self,
      None => {
        match self.call.as_mut_jsx_props::<R>() {
          None => {
            self.err = Some(JSXBuilderError::InvalidProps);
          }
          Some(old) => {
            *old = props;
          }
        };
        self
      }
    }
  }

  pub fn build(self) -> Result<CallExpr, JSXBuilderError> {
    match self.err {
      Some(err) => Err(err),
      None => Ok(self.call),
    }
  }

  pub fn guarantee(self) -> CallExpr {
    match self.build() {
      Ok(call) => call,
      Err(err) => unreachable!("{:?}", err),
    }
  }
}

pub fn jsx_builder2<R: JSXRuntime>(call: CallExpr) -> JSXBuilder<R> {
  JSXBuilder {
    call,
    err: None,
    jsx: PhantomData::<R>,
  }
}

pub fn create_element<R: JSXRuntime>(span: Span, tag: impl JSXTagDef) -> JSXBuilder<R> {
  jsx_builder2(<CallExpr as JSXElement>::new_element::<R>(span, tag))
}

pub fn create_fragment<R: JSXRuntime>(span: Span) -> JSXBuilder<R> {
  jsx_builder2(<CallExpr as JSXElement>::new_fragment::<R>(span))
}

pub fn replace_element<R: JSXRuntime, T: Serialize>(
  existing: &impl JSXElement,
  tag: impl JSXTagDef,
  props: &T,
) -> JSXBuilder<R> {
  create_element::<R>(existing.as_arg0_span::<R>(), tag).props(existing.as_arg1_span::<R>(), props)
}

struct Context {
  parent: Expr,
  prop: Vec<String>,
  children: Vec<Expr>,
}

#[derive(Debug, Default)]
enum LastElement {
  Head,
  #[default]
  Body,
  Context,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct JSXDocument {
  pub head: Vec<Expr>,
  pub body: Vec<Expr>,
}

impl JSXDocument {
  pub fn to_fragment<R: JSXRuntime>(self) -> CallExpr {
    let mut elem = create_fragment::<R>(Default::default()).guarantee();

    let mut children = ArrayLit::dummy();
    children.extend(&mut self.head.into_iter());
    children.extend(&mut self.body.into_iter());

    elem
      .as_mut_jsx_props::<R>()
      .set_item("children", children.into());

    elem
  }
}

#[derive(Default)]
struct Stack(Vec<Context>);

#[derive(Default)]
pub struct DocumentBuilder<R: JSXRuntime> {
  state: LastElement,
  context: Stack,

  head: Vec<Expr>,
  body: Vec<Expr>,

  jsx: PhantomData<R>,
}

impl<R: JSXRuntime> DocumentBuilder<R> {
  pub fn element(&mut self, span: Span, tag: impl JSXTagDef, props: ObjectLit) -> &mut Self {
    self.push(
      create_element::<R>(span, tag)
        .arg1(props)
        .guarantee()
        .into(),
    );
    self
  }

  pub fn enter(&mut self, path: &[&str]) -> &mut Self {
    let parent = self.pop();
    self.context.0.push(Context {
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
    } = match self.context.0.pop() {
      Some(v) => v,
      None => return self,
    };

    let children = {
      let mut array = ArrayLit::dummy();
      array.extend(&mut children.into_iter());
      array
    };

    parent
      .as_mut_call()
      .as_mut_jsx_props::<R>()
      .set_item_at_path(prop, children.into())
      .expect("should have props");

    self.push(parent);
    self
  }

  pub fn flush(&mut self) {
    while !self.context.0.is_empty() {
      self.exit();
    }
  }

  fn pop(&mut self) -> Expr {
    match self.state {
      LastElement::Head => self.head.pop(),
      LastElement::Body => self.body.pop(),
      LastElement::Context => self.context.0.last_mut().and_then(|ctx| ctx.children.pop()),
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
      match self.context.0.last_mut() {
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

  pub fn into_document(self) -> JSXDocument {
    JSXDocument {
      head: self.head,
      body: self.body,
    }
  }
}

impl JSXDocument {
  pub fn into_module(mut self) -> Module {
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

impl Debug for Context {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    struct DisplayAsDebug<'a>(&'a str);

    impl Debug for DisplayAsDebug<'_> {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
      }
    }

    fn expr_to_name(e: &Expr) -> DisplayAsDebug<'_> {
      DisplayAsDebug(
        e.as_call()
          .and_then(|c| c.callee.as_expr())
          .and_then(|e| e.as_ident())
          .map(|ident| ident.sym.as_str())
          .unwrap_or("Expr?"),
      )
    }

    let mut t = f.debug_tuple(format!("{:?}", expr_to_name(&self.parent)).as_str());

    self.children.iter().for_each(|e| {
      t.field(&expr_to_name(e));
    });

    t.finish()
  }
}

impl Debug for Stack {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    struct DisplayAsDebug<'a>(&'a str);

    impl Debug for DisplayAsDebug<'_> {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
      }
    }

    struct ContextChain<'a>(&'a Vec<Context>, usize);

    impl Debug for ContextChain<'_> {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Some(ctx) = self.0.get(self.1) else {
          return Ok(());
        };

        let mut tuple = f.debug_tuple(format!("{:?}", expr_to_name(&ctx.parent)).as_str());

        ctx.children.iter().for_each(|e| {
          tuple.field(&expr_to_name(e));
        });

        if self.1 + 1 < self.0.len() {
          tuple.field(&ContextChain(self.0, self.1 + 1));
        }

        tuple.finish()
      }
    }

    fn expr_to_name(e: &Expr) -> DisplayAsDebug<'_> {
      DisplayAsDebug(
        e.as_call()
          .and_then(|c| c.args.first())
          .map(|e| &e.expr)
          .and_then(|e| e.as_ident())
          .map(|ident| ident.sym.as_str())
          .or_else(|| match e.as_lit() {
            Some(Lit::Str(Str { value, .. })) => Some(&**value),
            _ => None,
          })
          .unwrap_or("Expr?"),
      )
    }

    Debug::fmt(&ContextChain(&self.0, 0), f)
  }
}

impl<R: JSXRuntime> Debug for DocumentBuilder<R> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("DocumentBuilder")
      .field("state", &self.state)
      .field("context", &self.context)
      .finish()
  }
}

#[cfg(test)]
mod tests {
  use swc_ecma_testing2::{insta, print_one_unchecked};

  use crate::{ad_hoc_tag, json_expr, jsx::JSXSymbols, Object};

  use super::{create_element, create_fragment, DocumentBuilder};

  #[test]
  fn test_fragment() {
    let document =
      print_one_unchecked(&create_fragment::<JSXSymbols>(Default::default()).guarantee());
    insta::assert_snapshot!(document);
  }

  #[test]
  fn test_intrinsic() {
    let document = print_one_unchecked(&{
      create_element::<JSXSymbols>(Default::default(), ad_hoc_tag!("div"))
        .child("foo".into())
        .guarantee()
    });
    insta::assert_snapshot!(document);
  }

  #[test]
  fn test_component() {
    let document = print_one_unchecked(
      &create_element::<JSXSymbols>(Default::default(), ad_hoc_tag!(Foo)).guarantee(),
    );
    insta::assert_snapshot!(document);
  }

  #[test]
  fn test_props() {
    let document = print_one_unchecked(&{
      create_element::<JSXSymbols>(Default::default(), ad_hoc_tag!("div"))
        .prop(Default::default(), "className", &"foo")
        .prop(Default::default(), "id", &"bar")
        .guarantee()
    });
    insta::assert_snapshot!(document);
  }

  fn document_builder(build: impl Fn(&mut DocumentBuilder<JSXSymbols>)) -> String {
    let mut builder = Default::default();
    build(&mut builder);
    print_one_unchecked(&builder.into_document().into_module())
  }

  #[test]
  fn test_document_fragment() {
    let document = document_builder(|builder| {
      builder.element(
        Default::default(),
        ad_hoc_tag!(<>),
        json_expr!({}).object().expect("object"),
      );
    });
    insta::assert_snapshot!(document);
  }

  #[test]
  fn test_document_intrinsic() {
    let document = document_builder(|builder| {
      builder
        .element(
          Default::default(),
          ad_hoc_tag!("div"),
          json_expr!({}).object().expect("object"),
        )
        .enter(&["children"])
        .value("foo".into())
        .exit();
    });
    insta::assert_snapshot!(document);
  }

  #[test]
  fn test_document_props() {
    let document = document_builder(|builder| {
      builder
        .element(
          Default::default(),
          ad_hoc_tag!("a"),
          json_expr!({
            "href": "https://example.com",
            "title": "Example"
          })
          .object()
          .expect("object"),
        )
        .enter(&["children"])
        .value("Example".into())
        .exit();
    });
    insta::assert_snapshot!(document);
  }

  #[test]
  fn test_document_head() {
    let document = document_builder(|builder| {
      builder
        .element(Default::default(), ad_hoc_tag!("section"), Object!())
        .enter(&["children"])
        .element(Default::default(), ad_hoc_tag!("style"), Object!())
        .enter(&["children"])
        .value("p { background: #fff; }".into())
        .exit()
        .element(
          Default::default(),
          ad_hoc_tag!("link"),
          json_expr!({
            "rel": "preconnect",
            "href": "https://rsms.me/"
          })
          .object()
          .expect("object"),
        )
        .element(Default::default(), ad_hoc_tag!("p"), Object!())
        .enter(&["children"])
        .value("Lorem ipsum".into())
        .exit()
        .exit();
    });
    insta::assert_snapshot!(document);
  }
}
