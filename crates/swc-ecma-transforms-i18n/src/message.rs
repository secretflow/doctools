use serde::{Deserialize, Serialize};
use swc_core::{
    common::util::take::Take as _,
    ecma::ast::{
        CallExpr, Callee, Expr, ExprOrSpread, Ident, KeyValueProp, ObjectLit, Prop, PropName,
        PropOrSpread,
    },
};

use swc_utils::{
    jsx::factory::{JSXElement, JSXFactory},
    object_lit,
};

/// Represents a message to be translated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub message: String,
    pub plaintext: String,
}

#[derive(Debug)]
enum MessageToken {
    Text(String),
    Interpolation((String, Box<Expr>)),
    OpeningTag((usize,)),
    ClosingTag((usize, Box<Expr>)),
}

#[derive(Debug)]
pub struct MessageProps {
    pre: bool,
    tokens: Vec<MessageToken>,
}

/// https://developer.mozilla.org/en-US/docs/Web/HTML/Content_categories#palpable_content
#[must_use]
pub struct Palpable(pub bool);

impl MessageProps {
    /// Returns [Err] if `text` is empty or whitespace.
    #[must_use]
    pub fn text(&mut self, text: &str) -> Palpable {
        if is_empty_or_whitespace(text) {
            Palpable(false)
        } else {
            let text = if self.pre {
                String::from(text)
            } else {
                let this = collapse_ascii_whitespace(text);
                match self.tokens.last() {
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
                }
            };
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
        self.tokens
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
        self.tokens.push(MessageToken::ClosingTag((idx, expr)))
    }

    pub fn is_empty(&self) -> bool {
        !self
            .tokens
            .iter()
            .any(|t| matches!(t, MessageToken::Text(_)))
    }

    fn to_message(&self) -> String {
        self.tokens
            .iter()
            .map(|token| match token {
                MessageToken::Text(text) => text.to_string(),
                MessageToken::Interpolation((idx, _)) => format!("{{{}}}", idx),
                MessageToken::OpeningTag((idx,)) => format!("<{}>", idx),
                MessageToken::ClosingTag((idx, _)) => format!("</{}>", idx),
            })
            .collect::<Vec<_>>()
            .join("")
            .trim()
            .to_string()
    }

    fn to_plaintext(&self) -> String {
        collapse_ascii_whitespace(
            self.tokens
                .iter()
                .map(|c| match c {
                    MessageToken::Text(text) => text.trim().to_string(),
                    MessageToken::Interpolation(_) => String::from("..."),
                    MessageToken::OpeningTag(_) => "".into(),
                    MessageToken::ClosingTag(_) => "".into(),
                })
                .collect::<Vec<_>>()
                .join(" ")
                .trim(),
        )
        .trim()
        .to_string()
    }

    fn to_props(mut self) -> (ObjectLit, ObjectLit) {
        let mut components: Vec<PropOrSpread> = vec![];
        let mut values: Vec<PropOrSpread> = vec![];

        self.tokens.drain(..).for_each(|token| match token {
            MessageToken::ClosingTag((idx, mut expr)) => components.push(PropOrSpread::Prop(
                Prop::from(KeyValueProp {
                    key: PropName::Num(idx.into()),
                    value: expr.take(),
                })
                .into(),
            )),
            MessageToken::Interpolation((idx, mut expr)) => values.push(PropOrSpread::Prop(
                Prop::from(KeyValueProp {
                    key: PropName::Str(idx.into()),
                    value: expr.take(),
                })
                .into(),
            )),
            _ => {}
        });

        (
            ObjectLit {
                props: components,
                span: Default::default(),
            },
            ObjectLit {
                props: values,
                span: Default::default(),
            },
        )
    }

    /// # Panics
    ///
    /// Expect message collectors to ensure messages aren't empty.
    ///
    /// Since calling this function implies the end of a [swc_core::ecma::visit::VisitMut],
    /// mutating the tree only to result in an empty message is unexpected and likely a bug
    pub fn make_trans(self, factory: &JSXFactory, trans: &str) -> (Message, Box<Expr>) {
        let message = self.to_message();

        if is_empty_or_whitespace(&message) {
            unreachable!("Message is empty")
        }

        let plaintext = self.to_plaintext();

        let (components, values) = self.to_props();

        let trans = factory
            .create(&JSXElement::Ident(trans.into()))
            .prop("message", message.as_str().into(), None)
            .prop("components", components.into(), None)
            .prop("values", values.into(), None)
            .build();

        (Message { message, plaintext }, trans.into())
    }

    pub fn make_i18n(self, i18n: &str) -> (Message, Box<Expr>) {
        let message = self.to_message();

        let plaintext = self.to_plaintext();

        let (_, values) = self.to_props();

        let call = Expr::Call(CallExpr {
            callee: Callee::Expr(Ident::from(i18n).into()),
            args: vec![ExprOrSpread {
                expr: object_lit!("message" = message.as_str(), "values" = values).into(),
                spread: None,
            }],
            type_args: None,
            span: Default::default(),
        });

        (Message { message, plaintext }, call.into())
    }

    pub fn new(pre: bool) -> Self {
        Self {
            pre,
            tokens: vec![],
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
