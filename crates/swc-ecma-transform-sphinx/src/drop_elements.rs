use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use swc_core::{
  common::util::take::Take,
  ecma::{
    ast::CallExpr,
    visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith as _},
  },
};

use swc_ecma_utils::{
  jsx::factory::{JSXRuntime, JSXTagName},
  match_jsx, tag,
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
  elements: HashMap<JSXTagName, Drop>,
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

  pub fn unwrap(&mut self, tag: JSXTagName) -> &mut Self {
    self.elements.insert(tag, Drop::Unwrap);
    self
  }

  pub fn delete(&mut self, tag: JSXTagName) -> &mut Self {
    self.elements.insert(tag, Drop::Delete);
    self
  }
}

struct ElementDropper {
  jsx: JSXRuntime,
  options: DropElements,
}

impl VisitMut for ElementDropper {
  noop_visit_mut_type!();

  fn visit_mut_call_expr(&mut self, elem: &mut CallExpr) {
    elem.visit_mut_children_with(self);

    let name = match_jsx!((self.jsx, elem), Any(name) >> { name }, _ >> { return },);

    let drop = match self.options.elements.get(&name) {
      Some(drop) => drop,
      None => {
        elem.visit_mut_children_with(self);
        return;
      }
    };

    match drop {
      Drop::Unwrap => {
        let props = self.jsx.as_mut_jsx_props(elem).unwrap();
        let children = self.jsx.take_prop(props, &["children"]);
        match children {
          Some(children) => {
            *elem = self.jsx.create(&tag!(<>)).arg1(Box::new(children)).build();
          }
          None => {
            elem.take();
          }
        }
      }
      Drop::Delete => {
        elem.take();
      }
    };
  }
}

pub fn drop_elements(
  jsx: JSXRuntime,
  configurer: impl Fn(&mut DropElements) -> &mut DropElements,
) -> impl Fold + VisitMut {
  let mut options = DropElements::new();
  configurer(&mut options);
  as_folder(ElementDropper { jsx, options })
}
