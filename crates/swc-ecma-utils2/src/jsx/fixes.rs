use std::{collections::HashMap, marker::PhantomData};

use serde::{Deserialize, Serialize};
use swc_core::{
  common::util::take::Take,
  ecma::{
    ast::{ArrayLit, CallExpr, Expr, ExprOrSpread},
    visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith},
  },
};

use super::{JSXElement as _, JSXElementMut, JSXRuntime, JSXTagType};
use crate::{
  collections::{Mapping as _, MutableMapping as _, MutableSequence, Sequence as _},
  ecma::itertools::{is_invalid_call, is_nullish},
  jsx::create_fragment,
  tag_test, Object,
};

struct FoldFragments<R: JSXRuntime>(PhantomData<R>);

impl<R: JSXRuntime> FoldFragments<R> {
  fn should_remove_child(&self, elem: &Expr) -> bool {
    let Some(call) = elem.as_call() else {
      return is_nullish(elem);
    };

    let Some(elem) = call.as_jsx_type::<R>() else {
      return is_invalid_call(call);
    };

    match elem {
      tag_test!(*?) | tag_test!("*"?) => false,
      tag_test!(<>?) => match call.as_jsx_props::<R>().get_item("children") {
        None => true,
        Some(children) => match children.as_array() {
          Some(array) => array.iter().all(is_nullish),
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

    let children = {
      let Some(props) = call.as_mut_jsx_props::<R>() else {
        return;
      };
      let Some(children) = props.del_item("children") else {
        return;
      };
      children
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
      call.as_mut_jsx_props::<R>().set_item("children", children);
    }

    let orphan = match call.as_jsx_type::<R>() {
      Some(tag_test!(<>?)) => match call.as_mut_jsx_props::<R>().get_item_mut("children") {
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
        call.as_mut_jsx_props::<R>().set_item("children", expr);
      }
    }
  }
}

struct FixJSXFactory<R: JSXRuntime>(PhantomData<R>);

impl<R: JSXRuntime> VisitMut for FixJSXFactory<R> {
  fn visit_mut_call_expr(&mut self, call: &mut CallExpr) {
    call.visit_mut_children_with(self);

    let num_children = if let Some(children) = call.as_jsx_props::<R>().get_item("children") {
      match children.as_array() {
        Some(array) => array.len(),
        None => 1,
      }
    } else {
      0
    };

    call.set_jsx_factory::<R>(num_children);

    if num_children == 0 {
      call.as_mut_jsx_props::<R>().del_item("children");
    }
  }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Drop {
  /// Replace self with children, discarding all props
  Unwrap,
  /// Delete self and children completely
  Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropElements<'a> {
  #[serde(borrow)]
  elements: HashMap<JSXTagType<'a>, Drop>,
}

impl Default for DropElements<'_> {
  fn default() -> Self {
    Self::new()
  }
}

impl<'a> DropElements<'a> {
  pub fn new() -> Self {
    Self {
      elements: HashMap::new(),
    }
  }

  pub fn unwrap(mut self, tag: JSXTagType<'a>) -> Self {
    self.elements.insert(tag, Drop::Unwrap);
    self
  }

  pub fn delete(mut self, tag: JSXTagType<'a>) -> Self {
    self.elements.insert(tag, Drop::Delete);
    self
  }

  pub fn build<R: JSXRuntime + 'a>(self) -> impl Fold + VisitMut + 'a {
    as_folder(ElementDropper::<R> {
      options: self,
      jsx: PhantomData,
    })
  }
}

struct ElementDropper<'a, R: JSXRuntime> {
  options: DropElements<'a>,
  jsx: PhantomData<R>,
}

impl<R: JSXRuntime> ElementDropper<'_, R> {
  fn unwrap_elem(&mut self, call: &mut CallExpr) -> Option<()> {
    let children = call.as_mut_jsx_props::<R>()?.del_item("children");
    match children {
      Some(children) => {
        *call = create_fragment::<R>(call.as_arg0_span::<R>())
          .arg1(Object!([children]))
          .guarantee();
      }
      None => {
        call.take();
      }
    };
    Some(())
  }

  fn process_call_expr(&mut self, call: &mut CallExpr) -> Option<()> {
    let tag = call.as_jsx_type::<R>()?;
    let drop = self.options.elements.get(&tag)?;
    match drop {
      Drop::Unwrap => {
        self.unwrap_elem(call);
      }
      Drop::Delete => {
        call.take();
      }
    };
    Some(())
  }
}

impl<R: JSXRuntime> VisitMut for ElementDropper<'_, R> {
  noop_visit_mut_type!();

  fn visit_mut_call_expr(&mut self, call: &mut CallExpr) {
    call.visit_mut_children_with(self);
    self.process_call_expr(call);
  }
}

pub fn fold_fragments<R: JSXRuntime>() -> impl Fold + VisitMut {
  as_folder(FoldFragments(PhantomData::<R>))
}

pub fn fix_jsx_factories<R: JSXRuntime>() -> impl Fold + VisitMut {
  as_folder(FixJSXFactory(PhantomData::<R>))
}

pub fn drop_elements() -> DropElements<'static> {
  DropElements::new()
}
