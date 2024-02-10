use std::{collections::HashMap, marker::PhantomData};

use serde::{Deserialize, Serialize};
use swc_core::{
  common::util::take::Take,
  ecma::{
    ast::CallExpr,
    visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith as _},
  },
};
use swc_ecma_utils2::{
  collections::MutableMapping,
  jsx::{jsx, jsx_mut, tag::JSXTag, JSXElement, JSXElementMut, JSXRuntime},
  Object, JSX,
};

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

  pub fn unwrap(&mut self, tag: JSXTag) -> &mut Self {
    self.elements.insert(tag, Drop::Unwrap);
    self
  }

  pub fn delete(&mut self, tag: JSXTag) -> &mut Self {
    self.elements.insert(tag, Drop::Delete);
    self
  }
}

struct ElementDropper<R: JSXRuntime> {
  options: DropElements,
  jsx: PhantomData<R>,
}

impl<R: JSXRuntime> VisitMut for ElementDropper<R> {
  noop_visit_mut_type!();

  fn visit_mut_call_expr(&mut self, call: &mut CallExpr) {
    call.visit_mut_children_with(self);

    let Some(tag) = jsx::<R>(call).get_tag() else {
      return;
    };

    let drop = match self.options.elements.get(&tag) {
      Some(drop) => drop,
      None => {
        call.visit_mut_children_with(self);
        return;
      }
    };

    match drop {
      Drop::Unwrap => {
        let children = jsx_mut::<R>(call)
          .and_then(|e| e.get_props_mut())
          .and_then(|e| e.del_item("children"));
        match children {
          Some(children) => *call = JSX!([(), R], Object!("children" = children)),
          None => {
            call.take();
          }
        }
      }
      Drop::Delete => {
        call.take();
      }
    };
  }
}

pub fn drop_elements<R: JSXRuntime>(
  configurer: impl Fn(&mut DropElements) -> &mut DropElements,
) -> impl Fold + VisitMut {
  let mut options = DropElements::new();
  configurer(&mut options);
  as_folder(ElementDropper::<R> {
    options,
    jsx: PhantomData,
  })
}
