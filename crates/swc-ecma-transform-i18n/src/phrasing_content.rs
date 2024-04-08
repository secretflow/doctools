use std::marker::PhantomData;

use swc_core::{
  common::Spanned as _,
  ecma::{
    ast::{ArrayLit, CallExpr, Expr, Lit, Str},
    visit::{
      noop_visit_mut_type, noop_visit_type, Visit, VisitMut, VisitMutWith as _, VisitWith as _,
    },
  },
};

use swc_ecma_utils2::{
  collections::{Mapping, MutableMapping, MutableSequence, Sequence as _},
  ecma::itertools::array_into_iter,
  jsx::{jsx, jsx_mut, JSXElement, JSXElementMut, JSXRuntime, JSXTagDef as _},
  span::with_span,
  tag_test,
};

use crate::{
  message::{is_empty_or_whitespace, Message, MessageProps, Palpable},
  symbols::I18nSymbols,
};

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
struct PhrasingContentPreflight<R: JSXRuntime> {
  is_translatable: bool,
  jsx: PhantomData<R>,
}

impl<R: JSXRuntime> PhrasingContentPreflight<R> {
  fn check_call_expr(&mut self, call: &CallExpr) -> Option<()> {
    if self.is_translatable {
      return Some(());
    }

    let children = jsx::<R>(call)?.get_props()?.get_item("children")?;

    self.is_translatable = match &children {
      Expr::Array(array) => array.iter().any(|expr| match expr {
        Expr::Lit(Lit::Str(Str { value, .. })) => !is_empty_or_whitespace(&value),
        _ => false,
      }),
      Expr::Lit(Lit::Str(text)) => !is_empty_or_whitespace(&text.value),
      _ => false,
    };

    Some(())
  }
}

impl<R: JSXRuntime> Visit for PhrasingContentPreflight<R> {
  noop_visit_type!();

  fn visit_call_expr(&mut self, call: &CallExpr) {
    self.check_call_expr(call);
    call.visit_children_with(self);
  }
}

impl<R: JSXRuntime> PhrasingContentPreflight<R> {
  pub fn new() -> Self {
    Self {
      is_translatable: false,
      jsx: PhantomData,
    }
  }

  pub fn is_translatable(&self) -> bool {
    self.is_translatable
  }
}

#[derive(Debug)]
pub struct PhrasingContentCollector<R: JSXRuntime, S: I18nSymbols> {
  message: MessageProps,
  jsx: PhantomData<R>,
  i18n: PhantomData<S>,
}

impl<R, S> VisitMut for PhrasingContentCollector<R, S>
where
  R: JSXRuntime,
  S: I18nSymbols,
{
  noop_visit_mut_type!();

  fn visit_mut_call_expr(&mut self, elem: &mut CallExpr) {
    let Some(children) = jsx_mut::<R>(elem).get_props_mut().del_item("children") else {
      elem.visit_mut_children_with(self);
      return;
    };

    let children = match children {
      Expr::Array(array) => array,
      expr => ArrayLit::from_iterable(std::iter::once(expr)),
    };

    array_into_iter(children).for_each(|expr| match expr {
      Expr::Lit(Lit::Str(lit)) => match self.message.text(&lit.value, lit.span()) {
        Palpable(true) => (),
        Palpable(false) => (),
      },
      Expr::Call(mut call) => match jsx::<R>(&call).get_tag() {
        Some(tag) => {
          let name = match tag.tag_type() {
            tag_test!(<>?) => None,
            tag_test!(* as name?) | tag_test!("*" as name?) => Some(name.into()),
          };
          let name = self.message.enter(name);
          call.visit_mut_with(self);
          self.message.exit(name, call.into());
        }
        None => {
          self.message.interpolate(call.into());
        }
      },
      expr => {
        self.message.interpolate(expr);
      }
    });
  }
}

impl<R, S> PhrasingContentCollector<R, S>
where
  R: JSXRuntime,
  S: I18nSymbols,
{
  pub fn new(pre: bool) -> Self {
    Self {
      message: MessageProps::new(pre),
      jsx: PhantomData,
      i18n: PhantomData,
    }
  }

  pub fn result(self) -> (Message, Expr) {
    self.message.make_trans::<R, S>()
  }
}

pub fn translate_phrase<R: JSXRuntime, S: I18nSymbols>(
  pre: bool,
  elem: &mut CallExpr,
) -> Option<Message> {
  let mut preflight = <PhrasingContentPreflight<R>>::new();

  elem.visit_with(&mut preflight);

  if preflight.is_translatable() {
    let mut collector = <PhrasingContentCollector<R, S>>::new(pre);

    elem.visit_mut_with(&mut collector);

    let (message, children) = collector.result();

    let children = with_span(Some(elem.span()))(children);

    jsx_mut::<R>(elem)
      .get_props_mut()
      .set_item("children", children);

    Some(message)
  } else {
    None
  }
}
