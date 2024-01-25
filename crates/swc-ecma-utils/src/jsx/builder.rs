use serde::{Deserialize, Serialize};
use swc_core::{
  common::Span,
  ecma::ast::{ArrayLit, Expr, Ident},
};

use crate::span::with_span;

use super::factory::{JSXBuilder, JSXFactory, JSXTagName};

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
}

#[derive(Debug)]
pub struct DocumentBuilder {
  factory: JSXFactory,

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
    let mut builder = self.factory.create(name);
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
      .factory
      .as_mut_jsx_props(parent.as_mut_call().unwrap())
      .unwrap();

    self
      .factory
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
      Expr::Call(ref call) => self.factory.as_jsx(call).and_then(|t| Some(t.0)),
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

  pub fn new(jsx: JSXFactory) -> Self {
    Self {
      factory: jsx,
      state: Some(LastElement::Body),
      context: vec![],
      head: Children(vec![]),
      body: Children(vec![]),
    }
  }

  pub fn declare(self) -> JSXDocument {
    let wrap_tree = |elements: Vec<Box<Expr>>| {
      if elements.len() == 1 {
        elements.into_iter().next().unwrap()
      } else {
        self
          .factory
          .create(&JSXTagName::Fragment)
          .children(elements)
          .build()
          .into()
      }
    };

    let head = wrap_tree(self.head.0);
    let body = wrap_tree(self.body.0);

    JSXDocument { head, body }
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

#[cfg(test)]
mod tests {
  use serde_json::json;
  use swc_core::{ecma::codegen, testing::DebugUsingDisplay};

  use crate::{ast::json_to_expr, jsx::factory::JSXTagName, testing::print_one};

  use super::DocumentBuilder;

  fn test(build: impl Fn(&mut DocumentBuilder), head: &str, body: &str) {
    let jsx = Default::default();
    let mut builder = DocumentBuilder::new(jsx);
    build(&mut builder);
    let document = builder.declare();
    assert_eq!(
      DebugUsingDisplay(
        print_one(
          &document.head,
          None,
          Some(codegen::Config::default().with_minify(true))
        )
        .unwrap()
        .as_str()
      ),
      DebugUsingDisplay(head),
    );
    assert_eq!(
      DebugUsingDisplay(
        print_one(
          &document.body,
          None,
          Some(codegen::Config::default().with_minify(true))
        )
        .unwrap()
        .as_str()
      ),
      DebugUsingDisplay(body),
    )
  }

  #[test]
  fn test_fragment() {
    test(
      |builder| {
        builder.element(&JSXTagName::Fragment, None, None);
      },
      "jsx(Fragment,{})",
      "jsx(Fragment,{})",
    );
  }

  #[test]
  fn test_intrinsic() {
    test(
      |builder| {
        builder
          .element(&"div".into(), None, None)
          .enter(&["children"])
          .value("foo".into())
          .exit();
      },
      "jsx(Fragment,{})",
      r#"jsx("div",{"children":["foo"]})"#,
    )
  }

  #[test]
  fn test_props() {
    test(
      |builder| {
        builder
          .element(
            &"a".into(),
            Some(json_to_expr(
              json!({"href": "https://example.com", "title": "Example"}),
            )),
            None,
          )
          .enter(&["children"])
          .value("Example".into())
          .exit();
      },
      "jsx(Fragment,{})",
      r#"jsx("a",{"href":"https://example.com","title":"Example","children":["Example"]})"#,
    );
  }

  #[test]
  fn test_head() {
    test(
      |builder| {
        builder
          .element(&"section".into(), None, None)
          .enter(&["children"])
          .element(&"style".into(), None, None)
          .enter(&["children"])
          .value("p { background: #fff; }".into())
          .exit()
          .element(
            &"link".into(),
            Some(json_to_expr(
              json!({"rel": "preconnect", "href": "https://rsms.me/"}),
            )),
            None,
          )
          .element(&"p".into(), None, None)
          .enter(&["children"])
          .value("Lorem ipsum".into())
          .exit()
          .exit();
      },
      r#"jsxs(Fragment,{"children":[jsx("style",{"children":["p { background: #fff; }"]}),jsx("link",{"rel":"preconnect","href":"https://rsms.me/"})]})"#,
      r#"jsx("section",{"children":[jsx("p",{"children":["Lorem ipsum"]})]})"#,
    );
  }
}
