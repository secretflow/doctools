use std::marker::PhantomData;

use swc_core::{
  common::{util::take::Take as _, Span, Spanned},
  ecma::{
    ast::{ArrayLit, CallExpr, Expr, ExprOrSpread, Lit, ObjectLit, Str},
    visit::{noop_visit_mut_type, VisitMut, VisitMutWith as _},
  },
};

use swc_ecma_utils2::{
  collections::MutableMapping,
  jsx::{JSXElement, JSXElementMut, JSXRuntime},
  span::{union_span, with_span},
};

use crate::{
  message::{Message, MessageProps, Palpable},
  symbols::I18nSymbols,
};

#[derive(Debug)]
enum Block {
  Message(MessageProps),
  Expr(ExprOrSpread),
}

#[derive(Debug)]
struct FlowContentCollector<R: JSXRuntime, S: I18nSymbols> {
  pre: bool,
  blocks: Vec<(Block, Span)>,
  jsx: PhantomData<R>,
  i18n: PhantomData<S>,
}

macro_rules! current_message {
  ($this:expr) => {{
    match $this.blocks.last_mut() {
      Some((Block::Message(message), span)) => (message, span),
      _ => {
        let message = MessageProps::new($this.pre);
        let span = Span::dummy();
        $this.blocks.push((Block::Message(message), span));
        current_message!($this, yes)
      }
    }
  }};
  ($this:expr, yes) => {{
    match $this.blocks.last_mut() {
      Some((Block::Message(message), span)) => (message, span),
      _ => unreachable!(),
    }
  }};
}

impl<R, S> FlowContentCollector<R, S>
where
  R: JSXRuntime,
  S: I18nSymbols,
{
  fn text(&mut self, lit: Str) {
    let (message, span) = current_message!(self);
    match message.text(lit.value.as_str(), lit.span()) {
      Palpable(true) => *span = union_span(*span, lit.span()),
      Palpable(false) => self.other(Expr::from(lit).into()),
    }
  }

  fn interpolate(&mut self, expr: Expr) {
    let (message, span) = current_message!(self);
    *span = union_span(*span, expr.span());
    message.interpolate(expr);
  }

  fn other(&mut self, expr: ExprOrSpread) {
    let span = expr.span();
    self.blocks.push((Block::Expr(expr), span));
  }
}

impl<R, S> VisitMut for FlowContentCollector<R, S>
where
  R: JSXRuntime,
  S: I18nSymbols,
{
  noop_visit_mut_type!();

  fn visit_mut_object_lit(&mut self, props: &mut ObjectLit) {
    let Some(mut children) = props.del_item("children") else {
      return;
    };

    match children {
      Expr::Lit(Lit::Str(lit)) => self.text(lit),
      Expr::Array(ArrayLit { ref mut elems, .. }) => {
        elems.drain(..).for_each(|item| match item {
          None => (),
          Some(mut expr) => {
            if expr.spread.is_some() {
              self.other(expr)
            } else {
              match *expr.expr.take() {
                Expr::Lit(Lit::Str(lit)) => self.text(lit),
                Expr::Call(call) => {
                  if call.is_jsx::<R>() {
                    self.other(ExprOrSpread {
                      expr: Box::from(call),
                      spread: None,
                    });
                  } else {
                    self.interpolate(Expr::Call(call));
                  }
                }
                expr => {
                  self.interpolate(expr);
                }
              }
            }
          }
        });
      }
      expr => self.other(expr.into()),
    };
  }
}

impl<R, S> FlowContentCollector<R, S>
where
  R: JSXRuntime,
  S: I18nSymbols,
{
  pub fn new(pre: bool) -> Self {
    Self {
      pre,
      blocks: vec![],
      jsx: PhantomData,
      i18n: PhantomData,
    }
  }

  pub fn results(mut self) -> (Vec<Message>, ArrayLit) {
    let mut messages = vec![];
    let mut children = vec![];
    self.blocks.drain(..).for_each(|(block, span)| match block {
      Block::Message(message) => {
        if message.is_empty() {
          return;
        }
        let (message, elem) = message.make_trans::<R, S>();
        messages.push(message);
        children.push(Some(with_span(span)(elem).into()));
      }
      Block::Expr(expr) => children.push(Some(expr)),
    });
    let children = ArrayLit {
      span: Span::dummy(),
      elems: children,
    };
    (messages, children)
  }
}

pub fn translate_block<R: JSXRuntime, S: I18nSymbols>(
  pre: bool,
  call: &mut CallExpr,
) -> Vec<Message> {
  let mut collector = <FlowContentCollector<R, S>>::new(pre);

  let Some(props) = call.as_mut_jsx_props::<R>() else {
    return vec![];
  };

  props.visit_mut_with(&mut collector);

  let (messages, children) = collector.results();

  props.set_item("children", children.into());

  messages
}
