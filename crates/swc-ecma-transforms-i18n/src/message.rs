use base64::{prelude::BASE64_URL_SAFE, Engine};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use swc_core::{
  atoms::Atom,
  common::{sync::Lrc, util::take::Take as _, Span, Spanned as _},
  ecma::ast::{
    CallExpr, Callee, Expr, ExprOrSpread, Ident, KeyValueProp, Lit, ObjectLit, Prop, PropName,
    PropOrSpread,
  },
};

use swc_ecma_utils::{
  jsx::factory::{JSXRuntime, JSXTagName},
  object_lit,
  span::{union_span, with_span},
};

/// Represents a message to be translated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
  pub id: String,
  pub span: Span,
  pub message: String,
  pub plaintext: String,
}

#[derive(Debug)]
enum MessageToken {
  Text(String),
  Interpolation(Atom),
  OpeningTag(Atom),
  ClosingTag(Atom),
  LineFeed,
  LessThan,
  GreaterThan,
  LeftCurly,
  RightCurly,
}

#[derive(Debug)]
pub struct MessageProps {
  pre: bool,

  tokens: Vec<MessageToken>,
  values: IndexMap<Atom, Box<Expr>>,
  components: IndexMap<Atom, Option<Box<Expr>>>,

  span: Span,
}

/// https://developer.mozilla.org/en-US/docs/Web/HTML/Content_categories#palpable_content
#[must_use]
pub struct Palpable(pub bool);

impl MessageProps {
  pub fn raw(&mut self, text: &str, span: Span) {
    self.tokens.push(MessageToken::Text(text.to_string()));
    self.span = union_span(self.span, span);
  }

  #[must_use]
  pub fn text(&mut self, text: &str, span: Span) -> Palpable {
    if is_empty_or_whitespace(text) {
      match self.tokens.last() {
        Some(MessageToken::Text(last)) => {
          if is_empty_or_whitespace(last) {
            return Palpable(false);
          }
        }
        None => return Palpable(false),
        // preserve whitespace between elements
        _ => {}
      }
    }

    let text = if self.pre {
      text.to_string()
    } else {
      collapse_ascii_whitespace(text)
    };

    text
      .split_inclusive(['\n', '<', '>', '{', '}'])
      .for_each(|chunk| {
        let chunk = chunk.to_string();
        let (chunk, delimiter) = match chunk.chars().last() {
          Some(c) => match c {
            '\n' | '<' | '>' | '{' | '}' => (chunk[..chunk.len() - 1].to_string(), Some(c)),
            _ => (chunk, None),
          },
          None => unreachable!(),
        };
        let chunk = match self.tokens.last() {
          Some(MessageToken::Text(last)) => match last.chars().last() {
            Some(c1) => match chunk.chars().next() {
              Some(c2) => {
                if c1.is_ascii_whitespace() && c2.is_ascii_whitespace() {
                  chunk.trim_start().to_string()
                } else {
                  chunk
                }
              }
              None => chunk,
            },
            None => chunk,
          },
          _ => chunk,
        };
        self.tokens.push(MessageToken::Text(chunk));
        match delimiter {
          Some('\n') => self.tokens.push(MessageToken::LineFeed),
          Some('<') => self.tokens.push(MessageToken::LessThan),
          Some('>') => self.tokens.push(MessageToken::GreaterThan),
          Some('{') => self.tokens.push(MessageToken::LeftCurly),
          Some('}') => self.tokens.push(MessageToken::RightCurly),
          _ => {}
        }
      });

    self.span = union_span(self.span, span);

    Palpable(true)
  }

  pub fn interpolate(&mut self, expr: Box<Expr>) {
    self.span = union_span(self.span, expr.span());

    let idx = self.values.len();
    let name = match *expr {
      Expr::Ident(Ident { ref sym, .. }) => Atom::from(sym.as_str()),
      _ => Atom::from(idx.to_string().as_str()),
    };

    let name = match name.as_str() {
      "LF" | "LT" | "GT" | "LC" | "RC" => Atom::from(format!("{}_", name)),
      _ => name,
    };

    self
      .tokens
      .push(MessageToken::Interpolation(name.as_ref().into()));

    self.values.insert(name, expr);
  }

  pub fn enter(&mut self, name: Option<Atom>) -> Atom {
    let idx = self.components.len();

    let name = match name {
      Some(name) => {
        if self.components.get(&name).is_none() {
          name
        } else {
          Atom::from(format!("{}{}", name, idx))
        }
      }
      None => idx.to_string().into(),
    };

    self
      .tokens
      .push(MessageToken::OpeningTag(name.as_str().into()));
    self.components.insert(name.as_str().into(), None);

    name
  }

  pub fn exit(&mut self, name: Atom, expr: Box<Expr>) {
    self.span = union_span(self.span, expr.span());

    self
      .tokens
      .push(MessageToken::ClosingTag(name.as_str().into()));
    self.components.get_mut(&name).unwrap().replace(expr);
  }

  pub fn is_empty(&self) -> bool {
    !self
      .tokens
      .iter()
      .any(|t| matches!(t, MessageToken::Text(_)))
  }

  fn to_props(mut self, runtime: Lrc<JSXRuntime>) -> Props {
    let mut message = String::new();
    let mut plaintext = String::new();

    macro_rules! make_prop {
      ($name:expr, $expr:expr) => {
        PropOrSpread::Prop(
          Prop::from(KeyValueProp {
            key: PropName::Str($name.into()),
            value: $expr.into(),
          })
          .into(),
        )
      };
    }

    let mut has_newline = false;
    let mut has_less_than = false;
    let mut has_greater_than = false;
    let mut has_left_curly = false;
    let mut has_right_curly = false;

    self.tokens.drain(..).for_each(|token| match token {
      MessageToken::Text(text) => {
        message.push_str(&text);
        plaintext.push_str(&text);
      }
      MessageToken::Interpolation(name) => {
        message.push_str(&format!("{{{}}}", name));
        plaintext.push_str(" ... ");
      }
      MessageToken::OpeningTag(name) => {
        message.push_str(&format!("<{}>", name));
      }
      MessageToken::ClosingTag(name) => {
        message.push_str(&format!("</{}>", name));
      }
      MessageToken::LineFeed => {
        message.push_str("{LF}");
        plaintext.push_str(" ");
        has_newline = true;
      }
      MessageToken::LeftCurly => {
        message.push_str("{LC}");
        plaintext.push_str("{");
        has_left_curly = true;
      }
      MessageToken::RightCurly => {
        message.push_str("{RC}");
        plaintext.push_str("}");
        has_right_curly = true;
      }
      MessageToken::LessThan => {
        message.push_str("{LT}");
        plaintext.push_str("<");
        has_less_than = true;
      }
      MessageToken::GreaterThan => {
        message.push_str("{GT}");
        plaintext.push_str(">");
        has_greater_than = true;
      }
    });

    let mut components: Vec<PropOrSpread> = vec![];
    let mut values: Vec<PropOrSpread> = vec![];

    self.components.drain(..).for_each(|(name, expr)| {
      components.push(make_prop!(name, expr.unwrap().as_mut().take()));
    });

    self.values.drain(..).for_each(|(name, mut expr)| {
      values.push(make_prop!(name, expr.take()));
    });

    if has_newline {
      values.push(make_prop!("LF", runtime.create(&"br".into()).build()));
    }

    if has_less_than {
      values.push(make_prop!(
        "LT",
        runtime
          .create(&JSXTagName::Fragment)
          .children(vec!["<".into()])
          .build()
      ));
    }

    if has_greater_than {
      values.push(make_prop!(
        "GT",
        runtime
          .create(&JSXTagName::Fragment)
          .children(vec![">".into()])
          .build()
      ));
    }

    if has_left_curly {
      values.push(make_prop!(
        "LC",
        runtime
          .create(&JSXTagName::Fragment)
          .children(vec!["{".into()])
          .build()
      ));
    }

    if has_right_curly {
      values.push(make_prop!(
        "RC",
        runtime
          .create(&JSXTagName::Fragment)
          .children(vec!["}".into()])
          .build()
      ));
    }

    Props {
      id: self.generate_id(&message),
      message,
      plaintext,
      components: ObjectLit {
        props: components,
        span: self.span,
      },
      values: ObjectLit {
        props: values,
        span: self.span,
      },
    }
  }

  fn generate_id(&self, msg: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(msg);
    let result = hasher.finalize();
    let mut msg = String::new();
    BASE64_URL_SAFE.encode_string(result, &mut msg);
    msg
  }

  /// # Panics
  ///
  /// Expect message collectors to ensure messages aren't empty.
  ///
  /// Since calling this function implies the end of a [swc_core::ecma::visit::VisitMut],
  /// mutating the tree only to result in an empty message is unexpected and likely a bug
  pub fn make_trans(self, runtime: Lrc<JSXRuntime>, trans: Atom) -> (Message, Box<Expr>) {
    let span = self.span;

    let Props {
      id,
      message,
      plaintext,
      components,
      values,
    } = self.to_props(runtime.clone());

    if is_empty_or_whitespace(&message) {
      unreachable!("Message is empty")
    }

    let trans = with_span(Some(span))(
      runtime
        .create(&JSXTagName::Ident((&*trans).into()))
        .prop("id", id.as_str().into(), None)
        .prop("message", message.as_str().into(), None)
        .prop("components", components.into(), None)
        .prop("values", values.into(), None)
        .build(),
    );

    (
      Message {
        id,
        message,
        plaintext,
        span,
      },
      trans.into(),
    )
  }

  pub fn make_i18n(self, runtime: Lrc<JSXRuntime>, i18n: Atom) -> (Message, Box<Expr>) {
    let span = self.span;

    let Props {
      id,
      message,
      plaintext,
      components: _,
      values,
    } = self.to_props(runtime);

    let call = Expr::Call(CallExpr {
      callee: Callee::Expr(Ident::from(&*i18n).into()),
      args: vec![ExprOrSpread {
        expr: object_lit!(
          "id" = id.as_str(),
          "message" = with_span(Some(span))(Lit::from(message.as_str())),
          "values" = values
        )
        .into(),
        spread: None,
      }],
      type_args: None,
      span,
    });

    (
      Message {
        id,
        message,
        plaintext,
        span,
      },
      call.into(),
    )
  }

  pub fn new(pre: bool) -> Self {
    Self {
      pre,
      tokens: vec![],
      span: Default::default(),
      values: Default::default(),
      components: Default::default(),
    }
  }
}

struct Props {
  id: String,
  message: String,
  plaintext: String,
  components: ObjectLit,
  values: ObjectLit,
}

pub fn is_empty_or_whitespace(str: &str) -> bool {
  str.chars().all(|c| c.is_ascii_whitespace())
}

/// Collapse whitespace according to HTML's whitespace rules.
///
/// https://infra.spec.whatwg.org/#ascii-whitespace
pub fn collapse_ascii_whitespace(str: &str) -> String {
  let mut result = String::new();
  let mut last_char = '\0';
  str.chars().for_each(|c: char| {
    if c.is_ascii_whitespace() {
      if last_char.is_ascii_whitespace() {
        ()
      } else {
        last_char = c;
        result.push(' ');
      }
    } else {
      last_char = c;
      result.push(c);
    }
  });
  result
}
