use std::{collections::HashMap, marker::PhantomData};

use serde::{Deserialize, Serialize};
use swc_core::{
  common::util::take::Take,
  ecma::{
    ast::{ArrayLit, CallExpr, Expr, ExprOrSpread},
    visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith},
  },
};

use super::{
  jsx, jsx_mut,
  runtime::JSXRuntime,
  tag::{JSXTag, JSXTagMatch},
  JSXElement, JSXElementMut,
};
use crate::{
  collections::{Mapping, MutableMapping, MutableSequence, Sequence},
  ecma::itertools::{is_invalid_call, is_nullish},
  jsx::create_fragment,
  tag, Object,
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
      Some(tag!(*?) | tag!("*"?)) => false,
      Some(tag!(<>?)) => match elem.get_props().get_item("children") {
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
      Some(tag!(<>?)) => match elem.get_props_mut().get_item_mut("children") {
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

    let num_children = if let Some(children) = props.get_item("children") {
      match children.as_array() {
        Some(array) => {
          let len = array.len();
          elem.set_factory(len);
          len
        }
        None => {
          elem.set_factory(1);
          1
        }
      }
    } else {
      elem.set_factory(1);
      0
    };

    if num_children == 0 {
      elem.get_props_mut().del_item("children");
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
pub struct DropElements {
  elements: HashMap<JSXTag, Drop>,
}

impl Default for DropElements {
  fn default() -> Self {
    Self::new()
  }
}

impl DropElements {
  pub fn new() -> Self {
    Self {
      elements: HashMap::new(),
    }
  }

  pub fn unwrap(mut self, tag: JSXTag) -> Self {
    self.elements.insert(tag, Drop::Unwrap);
    self
  }

  pub fn delete(mut self, tag: JSXTag) -> Self {
    self.elements.insert(tag, Drop::Delete);
    self
  }

  pub fn build<R: JSXRuntime>(self) -> impl Fold + VisitMut {
    as_folder(ElementDropper::<R> {
      options: self,
      jsx: PhantomData,
    })
  }
}

struct ElementDropper<R: JSXRuntime> {
  options: DropElements,
  jsx: PhantomData<R>,
}

impl<R: JSXRuntime> ElementDropper<R> {
  fn unwrap_elem(&mut self, call: &mut CallExpr) -> Option<()> {
    let children = jsx_mut::<R>(call)?.get_props_mut().del_item("children");
    match children {
      Some(children) => {
        *call = create_fragment::<R>()
          .arg1(Object!([children]).into())
          .span(call.span)
          .guarantee();
      }
      None => {
        call.take();
      }
    };
    Some(())
  }

  fn process_call_expr(&mut self, call: &mut CallExpr) -> Option<()> {
    let tag = jsx::<R>(call)?.get_tag()?;
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

impl<R: JSXRuntime> VisitMut for ElementDropper<R> {
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

pub fn drop_elements() -> DropElements {
  DropElements::new()
}
