use base64::{prelude::BASE64_URL_SAFE, Engine};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use swc_core::{
  common::{util::take::Take as _, Span, Spanned as _},
  ecma::ast::{
    CallExpr, Callee, Expr, ExprOrSpread, Ident, KeyValueProp, Lit, ObjectLit, Prop, PropName,
    PropOrSpread,
  },
};

use swc_utils::{
  jsx::factory::{JSXElement, JSXFactory},
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
  Interpolation((String, Box<Expr>)),
  OpeningTag((usize,)),
  ClosingTag((usize, Box<Expr>)),
  NewLine,
}

#[derive(Debug)]
pub struct MessageProps {
  pre: bool,
  tokens: Vec<MessageToken>,
  span: Span,
}

/// https://developer.mozilla.org/en-US/docs/Web/HTML/Content_categories#palpable_content
#[must_use]
pub struct Palpable(pub bool);

impl MessageProps {
  /// Returns [Err] if `text` is empty or whitespace.
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
    if self.pre {
      // currently, newlines need to be converted to <br>s
      text.split_inclusive('\n').for_each(|line| {
        self.span = union_span(self.span, span);
        self.tokens.push(MessageToken::Text(line.to_string()));
        if line.ends_with('\n') {
          self.tokens.push(MessageToken::NewLine);
        }
      });
      Palpable(true)
    } else {
      let this = collapse_ascii_whitespace(text);
      let text = match self.tokens.last() {
        Some(MessageToken::Text(last)) => match last.chars().last() {
          Some(c1) => match this.chars().next() {
            Some(c2) => {
              if c1.is_ascii_whitespace() && c2.is_ascii_whitespace() {
                this.trim_start().to_string()
              } else {
                this
              }
            }
            None => this,
          },
          None => this,
        },
        _ => this,
      };
      self.span = union_span(self.span, span);
      self.tokens.push(MessageToken::Text(text));
      Palpable(true)
    }
  }

  /// Index starts at 1 because this is for humans.
  pub fn interpolate(&mut self, expr: Box<Expr>) {
    let placeholder = match *expr {
      Expr::Ident(Ident { ref sym, .. }) => sym.to_string(),
      _ => (self
        .tokens
        .iter()
        .filter(|t| matches!(t, MessageToken::Interpolation(_)))
        .count()
        + 1)
        .to_string(),
    };
    self.span = union_span(self.span, expr.span());
    self
      .tokens
      .push(MessageToken::Interpolation((placeholder, expr)));
  }

  /// Index starts at 1 because this is for humans.
  pub fn enter(&mut self) -> usize {
    let idx = self
      .tokens
      .iter()
      .filter(|t| matches!(t, MessageToken::OpeningTag(_)))
      .count()
      + 1;
    self.tokens.push(MessageToken::OpeningTag((idx,)));
    idx
  }

  pub fn exit(&mut self, idx: usize, expr: Box<Expr>) {
    self.span = union_span(self.span, expr.span());
    self.tokens.push(MessageToken::ClosingTag((idx, expr)));
  }

  pub fn is_empty(&self) -> bool {
    !self
      .tokens
      .iter()
      .any(|t| matches!(t, MessageToken::Text(_)))
  }

  fn to_message(&self) -> (String, String) {
    let message = self
      .tokens
      .iter()
      .map(|token| match token {
        MessageToken::Text(text) => text
          // https://github.com/lingui/js-lingui/issues/1075
          .replace("{", "'{")
          .replace("}", "}'")
          .to_string(),
        MessageToken::Interpolation((idx, _)) => format!("{{{}}}", idx),
        MessageToken::OpeningTag((idx,)) => format!("<{}>", idx),
        MessageToken::ClosingTag((idx, _)) => format!("</{}>", idx),
        MessageToken::NewLine => "<0/>".to_string(),
      })
      .collect::<Vec<_>>()
      .join("")
      .trim()
      .to_string();
    let id = self.generate_id(&message);
    (message, id)
  }

  fn to_plaintext(&self) -> String {
    (self
      .tokens
      .iter()
      .map(|c| match c {
        MessageToken::Text(text) => text.trim().to_string(),
        MessageToken::Interpolation(_) => String::from("..."),
        MessageToken::OpeningTag(_) => "".into(),
        MessageToken::ClosingTag(_) => "".into(),
        MessageToken::NewLine => " ".into(),
      })
      .collect::<Vec<_>>()
      .join(" ")
      .trim())
    .trim()
    .to_string()
  }

  fn to_props(mut self, factory: &JSXFactory) -> (ObjectLit, ObjectLit) {
    let mut components: Vec<PropOrSpread> = vec![];
    let mut values: Vec<PropOrSpread> = vec![];

    let insert = |arr: &mut Vec<PropOrSpread>, mut expr: Box<Expr>, idx: &str| {
      arr.push(PropOrSpread::Prop(
        Prop::from(KeyValueProp {
          key: PropName::Str(idx.into()),
          value: expr.take(),
        })
        .into(),
      ))
    };

    insert(
      &mut components,
      factory.create(&"br".into()).build().into(),
      "0",
    );

    self.tokens.drain(..).for_each(|token| match token {
      MessageToken::ClosingTag((idx, expr)) => insert(&mut components, expr, &idx.to_string()),
      MessageToken::Interpolation((idx, expr)) => insert(&mut values, expr, &idx),
      _ => {}
    });

    (
      ObjectLit {
        props: components,
        span: self.span,
      },
      ObjectLit {
        props: values,
        span: self.span,
      },
    )
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
  pub fn make_trans(self, factory: &JSXFactory, trans: &str) -> (Message, Box<Expr>) {
    let span = self.span;

    let (message, id) = self.to_message();

    if is_empty_or_whitespace(&message) {
      unreachable!("Message is empty")
    }

    let plaintext = self.to_plaintext();

    let (components, values) = self.to_props(factory);

    let trans = with_span(Some(span))(
      factory
        .create(&JSXElement::Ident(trans.into()))
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

  pub fn make_i18n(self, factory: &JSXFactory, i18n: &str) -> (Message, Box<Expr>) {
    let span = self.span;

    let (message, id) = self.to_message();

    let plaintext = self.to_plaintext();

    let (_, values) = self.to_props(factory);

    let call = Expr::Call(CallExpr {
      callee: Callee::Expr(Ident::from(i18n).into()),
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
    }
  }
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
