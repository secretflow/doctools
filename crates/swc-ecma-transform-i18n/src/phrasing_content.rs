use swc_core::{
  atoms::Atom,
  common::{util::take::Take as _, Spanned as _},
  ecma::{
    ast::{ArrayLit, CallExpr, Expr, ExprOrSpread, Lit, Str},
    visit::{
      noop_visit_mut_type, noop_visit_type, Visit, VisitMut, VisitMutWith as _, VisitWith as _,
    },
  },
};

use swc_ecma_utils::{jsx::factory::JSXRuntime, jsx_or_continue_visit, span::with_span, tag};

use crate::message::{is_empty_or_whitespace, Message, MessageProps, Palpable};

/// For [phrasing][crate::ContentModel::Phrasing] content, transform is done in two phases.
///
/// 1. [PhrasingContentPreflight] visits the tree **immutably** and determines if the element
///    is translatable i.e. if any non-whitespace text is present within the element
///    (think [Element.innerText])
/// 2. If it is indeed translatable, [PhrasingContentCollector] visits the tree **mutably**
///    and transform it into `<Trans>`
///
/// [Element.innerText]: https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/innerText
///
/// The first visit obviously adds extra overhead, but the alternative would be trying
/// to determine whether the element is translatable while borrowing it mutably. Because
/// whether the element has any text cannot be readily determined without visiting its
/// (arbitrarily deep) descendants, trying to avoid `mut` until proven necessary would
/// involve a lot of backtracking / conditionals / very fragile
/// [AST node taking][swc_core::common::util::take::Take]. This is much less ergonomic and
/// more error-prone than just visiting the tree twice.
struct PhrasingContentPreflight {
  jsx: JSXRuntime,
  is_translatable: bool,
}

impl Visit for PhrasingContentPreflight {
  noop_visit_type!();

  fn visit_call_expr(&mut self, call: &CallExpr) {
    if self.is_translatable {
      return;
    }

    let (_, props) = jsx_or_continue_visit!(self, self.jsx, call);

    let children = self.jsx.get_prop(props, &["children"]).get();

    self.is_translatable = match &children {
      Some(Expr::Array(ArrayLit { ref elems, .. })) => elems.iter().any(|expr| match expr {
        Some(ExprOrSpread { expr, .. }) => match &**expr {
          Expr::Lit(Lit::Str(Str { value, .. })) => !is_empty_or_whitespace(&value),
          _ => false,
        },
        None => false,
      }),
      Some(Expr::Lit(Lit::Str(text))) => !is_empty_or_whitespace(&text.value),
      _ => false,
    };

    call.visit_children_with(self);
  }
}

impl PhrasingContentPreflight {
  pub fn new(jsx: JSXRuntime) -> Self {
    Self {
      jsx,
      is_translatable: false,
    }
  }

  pub fn is_translatable(&self) -> bool {
    self.is_translatable
  }
}

#[derive(Debug)]
pub struct PhrasingContentCollector {
  jsx: JSXRuntime,
  sym_trans: Atom,
  message: MessageProps,
}

impl VisitMut for PhrasingContentCollector {
  noop_visit_mut_type!();

  fn visit_mut_call_expr(&mut self, elem: &mut CallExpr) {
    jsx_or_continue_visit!(self, self.jsx, mut elem);

    let children = match self
      .jsx
      .take_prop(self.jsx.as_mut_jsx_props(elem).unwrap(), &["children"])
    {
      Some(children) => children,
      None => return,
    };

    let children = match children {
      Expr::Array(ArrayLit { mut elems, .. }) => elems
        .iter_mut()
        .filter_map(|expr| match expr {
          None => None,
          Some(ExprOrSpread { expr, .. }) => Some(expr.take()),
        })
        .collect::<Vec<_>>(),
      expr => vec![Box::from(expr)],
    };

    children
      .into_iter()
      .for_each(|mut expr| match *expr.take() {
        Expr::Lit(Lit::Str(lit)) => match self.message.text(&lit.value, lit.span()) {
          Palpable(true) => (),
          Palpable(false) => (),
        },
        Expr::Call(mut call) => match self.jsx.as_jsx(&call) {
          Some((elem, _)) => {
            let name = match elem {
              tag!(<>) => None,
              tag!("*" name) => Some(name),
              tag!(let name) => Some(name),
            };
            let name = self.message.enter(name);
            call.visit_mut_with(self);
            self.message.exit(name, Box::from(call.take()));
          }
          None => {
            self.message.interpolate(Box::from(call.take()));
          }
        },
        expr => {
          self.message.interpolate(Box::from(expr));
        }
      });

    let props = self.jsx.as_mut_jsx_props(elem).unwrap();
    props.props = props
      .props
      .drain(..)
      .filter(|prop| {
        prop
          .as_prop()
          .and_then(|p| p.as_key_value())
          .and_then(|p| Some(!p.value.is_invalid()))
          .unwrap_or(false)
      })
      .collect();
  }
}

impl PhrasingContentCollector {
  pub fn new(jsx: JSXRuntime, sym_trans: Atom, pre: bool) -> Self {
    Self {
      jsx,
      sym_trans,
      message: MessageProps::new(pre),
    }
  }

  pub fn result(self) -> (Message, Box<Expr>) {
    self.message.make_trans(self.jsx, self.sym_trans)
  }
}

pub fn translate_phrase(
  jsx: JSXRuntime,
  sym_trans: Atom,
  pre: bool,
  elem: &mut CallExpr,
) -> Option<Message> {
  let mut preflight = PhrasingContentPreflight::new(jsx.clone());

  elem.visit_with(&mut preflight);

  if preflight.is_translatable() {
    let mut collector = PhrasingContentCollector::new(jsx.clone(), sym_trans.clone(), pre);

    elem.visit_mut_with(&mut collector);

    let (message, children) = collector.result();

    let children = with_span(Some(elem.span()))(children);

    jsx.mut_or_set_prop(jsx.as_mut_jsx_props(elem).unwrap(), &["children"], |expr| {
      *expr = children
    });

    Some(message)
  } else {
    None
  }
}
