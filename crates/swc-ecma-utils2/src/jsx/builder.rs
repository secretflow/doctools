use std::marker::PhantomData;

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
  span::with_span,
  Array,
};

use super::{jsx_mut, runtime::JSXRuntime, tag::JSXTag, JSXElement, JSXElementMut, JSXTagDef};

#[derive(Debug, thiserror::Error)]
pub enum JSXBuilderError {
  #[error("cannot serialize value as AST: {0}")]
  RepackError(#[source] RepackError),
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
      Some(_) => self,
      None => {
        jsx_mut::<R>(&mut self.call).set_type(tag.into_expr::<R>());
        self
      }
    }
  }

  pub fn prop<T: Serialize>(mut self, name: &str, value: &T) -> Self {
    match self.err {
      Some(_) => self,
      None => match repack_expr(value) {
        Err(err) => {
          self.err = Some(JSXBuilderError::RepackError(err));
          self
        }
        Ok(value) => {
          match jsx_mut::<R>(&mut self.call).get_props_mut() {
            None => {
              self.err = Some(JSXBuilderError::InvalidProps);
            }
            Some(props) => {
              props.set_item(name, value);
            }
          }
          self
        }
      },
    }
  }

  pub fn prop2(mut self, name: &str, value: Expr) -> Self {
    match self.err {
      Some(_) => self,
      None => {
        match jsx_mut::<R>(&mut self.call).get_props_mut() {
          None => {
            self.err = Some(JSXBuilderError::InvalidProps);
          }
          Some(props) => {
            props.set_item(name, value);
          }
        }
        self
      }
    }
  }

  pub fn child(mut self, child: Expr) -> Self {
    match self.err {
      Some(_) => self,
      None => match jsx_mut::<R>(&mut self.call).get_props_mut() {
        None => {
          self.err = Some(JSXBuilderError::InvalidChildren);
          self
        }
        Some(props) => match props.get_item_mut("children") {
          None => {
            props.set_item("children", child);
            self
          }
          Some(children) => match children {
            Expr::Array(children) => {
              children.append(child);
              self
            }
            children => {
              *children = Array!(children.take(), child).into();
              self
            }
          },
        },
      },
    }
  }

  pub fn props<T: Serialize>(mut self, props: &T) -> Self {
    match self.err {
      Some(_) => self,
      None => match repack_expr(props) {
        Err(err) => {
          self.err = Some(JSXBuilderError::RepackError(err));
          self
        }
        Ok(Expr::Object(props)) => {
          match jsx_mut::<R>(&mut self.call).get_props_mut() {
            None => {
              self.err = Some(JSXBuilderError::InvalidProps);
            }
            Some(old) => {
              *old = props;
            }
          }
          self
        }
        Ok(_) => {
          self.err = Some(JSXBuilderError::InvalidProps);
          self
        }
      },
    }
  }

  pub fn arg1(mut self, props: Expr) -> Self {
    match self.err {
      Some(_) => self,
      None => {
        match jsx_mut::<R>(&mut self.call).set_arg1(props) {
          None => {
            self.err = Some(JSXBuilderError::InvalidProps);
          }
          Some(_) => {}
        };
        self
      }
    }
  }

  pub fn span(mut self, span: Span) -> Self {
    self.call.span = span;
    self
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

#[inline(always)]
pub fn create_element<R: JSXRuntime>(tag: impl JSXTagDef) -> JSXBuilder<R> {
  let component = tag.into_expr::<R>();
  jsx_builder2(<CallExpr as JSXElement<R>>::create_element(component))
}

#[inline(always)]
pub fn create_fragment<R: JSXRuntime>() -> JSXBuilder<R> {
  jsx_builder2(<CallExpr as JSXElement<R>>::create_fragment())
}

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
    let mut elem = create_fragment::<R>().guarantee();

    let mut children = ArrayLit::dummy();
    children.extend(&mut self.head.into_iter());
    children.extend(&mut self.body.into_iter());

    jsx_mut::<R>(&mut elem)
      .get_props_mut()
      .set_item("children", children.into());

    elem
  }
}

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
    tag: JSXTag,
    props: Option<ObjectLit>,
    span: Option<Span>,
  ) -> &mut Self {
    let mut elem = create_element::<R>(tag).guarantee();

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

#[cfg(test)]
mod tests {
  use swc_ecma_testing2::{insta, print_one_unchecked};

  use crate::{ad_hoc_tag, json_expr, jsx::JSXRuntimeDefault};

  use super::{create_element, create_fragment, DocumentBuilder};

  #[test]
  fn test_fragment() {
    let document = print_one_unchecked(&create_fragment::<JSXRuntimeDefault>().guarantee());
    insta::assert_snapshot!(document);
  }

  #[test]
  fn test_intrinsic() {
    let document = print_one_unchecked(&{
      create_element::<JSXRuntimeDefault>(ad_hoc_tag!("div"))
        .child("foo".into())
        .guarantee()
    });
    insta::assert_snapshot!(document);
  }

  #[test]
  fn test_component() {
    let document =
      print_one_unchecked(&create_element::<JSXRuntimeDefault>(ad_hoc_tag!(Foo)).guarantee());
    insta::assert_snapshot!(document);
  }

  #[test]
  fn test_props() {
    let document = print_one_unchecked(&{
      create_element::<JSXRuntimeDefault>(ad_hoc_tag!("div"))
        .prop("className", &"foo")
        .prop("id", &"bar")
        .guarantee()
    });
    insta::assert_snapshot!(document);
  }

  fn document_builder(build: impl Fn(&mut DocumentBuilder<JSXRuntimeDefault>)) -> String {
    let mut builder = DocumentBuilder::new();
    build(&mut builder);
    print_one_unchecked(&builder.to_document().to_module())
  }

  #[test]
  fn test_document_fragment() {
    let document = document_builder(|builder| {
      builder.element(ad_hoc_tag!(<>), json_expr!({}).object(), None);
    });
    insta::assert_snapshot!(document);
  }

  #[test]
  fn test_document_intrinsic() {
    let document = document_builder(|builder| {
      builder
        .element(ad_hoc_tag!("div"), json_expr!({}).object(), None)
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
          ad_hoc_tag!("a"),
          json_expr!({
            "href": "https://example.com",
            "title": "Example"
          })
          .object(),
          None,
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
        .element(ad_hoc_tag!("section"), None, None)
        .enter(&["children"])
        .element(ad_hoc_tag!("style"), None, None)
        .enter(&["children"])
        .value("p { background: #fff; }".into())
        .exit()
        .element(
          ad_hoc_tag!("link"),
          json_expr!({
            "rel": "preconnect",
            "href": "https://rsms.me/"
          })
          .object(),
          None,
        )
        .element(ad_hoc_tag!("p"), None, None)
        .enter(&["children"])
        .value("Lorem ipsum".into())
        .exit()
        .exit();
    });
    insta::assert_snapshot!(document);
  }
}
