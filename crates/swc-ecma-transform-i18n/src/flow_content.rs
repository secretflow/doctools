use swc_core::{
  atoms::Atom,
  common::{util::take::Take as _, Span, Spanned},
  ecma::{
    ast::{ArrayLit, CallExpr, Expr, ExprOrSpread, Lit, ObjectLit, Str},
    visit::{noop_visit_mut_type, VisitMut, VisitMutWith as _},
  },
};

use swc_ecma_utils::{
  jsx::factory::JSXRuntime,
  span::{union_span, with_span},
};

use crate::message::{Message, MessageProps, Palpable};

#[derive(Debug)]
enum Block {
  Message(MessageProps),
  Expr(ExprOrSpread),
}

#[derive(Debug)]
struct FlowContentCollector {
  jsx: JSXRuntime,
  sym_trans: Atom,
  pre: bool,
  blocks: Vec<(Block, Span)>,
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

impl FlowContentCollector {
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
    message.interpolate(Box::from(expr));
  }

  fn other(&mut self, expr: ExprOrSpread) {
    let span = expr.span();
    self.blocks.push((Block::Expr(expr), span));
  }
}

impl VisitMut for FlowContentCollector {
  noop_visit_mut_type!();

  fn visit_mut_object_lit(&mut self, props: &mut ObjectLit) {
    let mut children = match self.jsx.take_prop(props, &["children"]) {
      Some(children) => children,
      None => return,
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
                Expr::Call(call) => match self.jsx.as_jsx(&call) {
                  Some(_) => {
                    self.other(ExprOrSpread {
                      expr: Box::from(call),
                      spread: None,
                    });
                  }
                  None => {
                    self.interpolate(Expr::Call(call));
                  }
                },
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

impl FlowContentCollector {
  pub fn new(jsx: JSXRuntime, sym_trans: Atom, pre: bool) -> Self {
    Self {
      jsx,
      sym_trans,
      pre,
      blocks: vec![],
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
        let (message, elem) = message.make_trans(self.jsx.clone(), self.sym_trans.clone());
        messages.push(message);
        children.push(Some(with_span(Some(span))(elem).into()));
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

pub fn translate_block(
  jsx: JSXRuntime,
  sym_trans: Atom,
  pre: bool,
  elem: &mut CallExpr,
) -> Vec<Message> {
  let mut collector = FlowContentCollector::new(jsx.clone(), sym_trans, pre);

  let props = jsx.as_mut_jsx_props(elem).unwrap();

  props.visit_mut_with(&mut collector);

  let (messages, children) = collector.results();

  jsx.mut_or_set_prop(jsx.as_mut_jsx_props(elem).unwrap(), &["children"], |expr| {
    *expr = Box::new(Expr::Array(children))
  });

  messages
}
