use std::marker::PhantomData;

use swc_core::{
  common::util::take::Take,
  ecma::{
    ast::{ArrayLit, CallExpr, Expr, ExprOrSpread},
    visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith},
  },
};

use crate::{
  collections::{Mapping, MutableMapping, MutableSequence, Sequence},
  ecma::{itertools::is_invalid_call, itertools::is_nullish},
};

use super::{
  jsx, jsx_mut,
  runtime::JSXRuntime,
  tag::{JSXTagMatch, JSXTagType},
  JSXElement, JSXElementMut,
};

struct FoldFragments<R: JSXRuntime>(PhantomData<R>);

impl<R: JSXRuntime> FoldFragments<R> {
  fn should_remove_child(&self, elem: &Expr) -> bool {
    let Some(call) = elem.as_call() else {
      return is_nullish(elem);
    };

    let Some(elem) = jsx::<R>(call) else {
      return is_invalid_call(call);
    };

    match elem.get_tag().tag_type() {
      None => false,
      Some(JSXTagType::Intrinsic(_) | JSXTagType::Component(_)) => false,
      Some(JSXTagType::Fragment) => match elem.get_props().get_item("children") {
        None => true,
        Some(children) => match children.as_array() {
          Some(array) => array.iter().all(|item| is_nullish(item)),
          None => is_nullish(children),
        },
      },
    }
  }
}

impl<R: JSXRuntime> VisitMut for FoldFragments<R> {
  noop_visit_mut_type!();

  fn visit_mut_call_expr(&mut self, call: &mut CallExpr) {
    call.visit_mut_children_with(self);

    let Some(elem) = jsx_mut::<R>(call) else {
      return;
    };

    let Some(children) = elem.get_props_mut().del_item("children") else {
      return;
    };

    let children = match children {
      Expr::Array(mut array) => {
        let mut items = array
          .drain()
          .filter_map(|item| {
            if self.should_remove_child(&item) {
              None
            } else {
              Some(ExprOrSpread {
                expr: item.into(),
                spread: None,
              })
            }
          })
          .map(Some)
          .collect::<Vec<_>>();
        if items.is_empty() {
          None
        } else if items.len() == 1 {
          Some(*items.pop().unwrap().unwrap().expr)
        } else {
          Some(Expr::Array(ArrayLit {
            elems: items,
            span: array.span,
          }))
        }
      }
      child => {
        if self.should_remove_child(&child) {
          None
        } else {
          Some(child)
        }
      }
    };

    if let Some(children) = children {
      elem.get_props_mut().set_item("children", children);
    }

    let orphan = match elem.get_tag().tag_type() {
      Some(JSXTagType::Fragment) => match elem.get_props_mut().get_item_mut("children") {
        None => None,
        Some(children) => match children.as_mut_array() {
          None => Some(children.take()),
          Some(children) => {
            if children.len() == 1 {
              children.pop(None)
            } else {
              None
            }
          }
        },
      },
      _ => None,
    };

    match orphan {
      None => {}
      Some(Expr::Call(orphan)) => {
        *call = orphan;
      }
      Some(expr) => {
        elem.get_props_mut().set_item("children", expr);
      }
    }
  }
}

struct FixJSXFactory<R: JSXRuntime>(PhantomData<R>);

impl<R: JSXRuntime> VisitMut for FixJSXFactory<R> {
  fn visit_mut_call_expr(&mut self, call: &mut CallExpr) {
    call.visit_mut_children_with(self);

    let Some(elem) = jsx_mut::<R>(call) else {
      return;
    };

    let props = elem.get_props();

    if let Some(children) = props.get_item("children") {
      match children.as_array() {
        Some(array) => {
          let len = array.len();
          elem.set_factory(len);
        }
        None => {
          elem.set_factory(1);
        }
      }
    } else {
      elem.set_factory(1);
    };
  }
}

pub fn fold_fragments<R: JSXRuntime>() -> impl Fold + VisitMut {
  as_folder(FoldFragments(PhantomData::<R>))
}

pub fn fix_jsx_factories<R: JSXRuntime>() -> impl Fold + VisitMut {
  as_folder(FixJSXFactory(PhantomData::<R>))
}
