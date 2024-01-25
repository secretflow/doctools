use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use swc_core::{
  common::{sync::Lrc, util::take::Take},
  ecma::{
    ast::CallExpr,
    visit::{as_folder, noop_visit_mut_type, Fold, VisitMut},
  },
};

use swc_ecma_utils::{
  continue_visit,
  jsx::factory::{JSXRuntime, JSXTagName},
  jsx_or_continue_visit,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Drop {
  /// Replace self with children, discarding all props
  Unwrap,
  /// Delete self and children completely
  Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementDropperOptions {
  elements: HashMap<JSXTagName, Drop>,
}

struct ElementDropper {
  runtime: Lrc<JSXRuntime>,
  options: ElementDropperOptions,
}

impl VisitMut for ElementDropper {
  noop_visit_mut_type!();

  fn visit_mut_call_expr(&mut self, elem: &mut CallExpr) {
    let (tag, _) = jsx_or_continue_visit!(self, self.runtime, mut elem);

    let drop = match self.options.elements.get(&tag) {
      Some(drop) => drop,
      None => continue_visit!(self, mut elem),
    };

    match drop {
      Drop::Unwrap => {
        let props = self.runtime.as_mut_jsx_props(elem).unwrap();
        let children = self.runtime.take_prop(props, &["children"]);
        match children {
          Some(children) => {
            *elem = self
              .runtime
              .create(&JSXTagName::Fragment)
              .arg1(Box::new(children))
              .build();
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

pub fn drop_elements(runtime: Lrc<JSXRuntime>) -> impl Fold + VisitMut {
  as_folder(ElementDropper {
    runtime,
    options: ElementDropperOptions {
      elements: {
        let mut elements = HashMap::new();
        elements.insert(JSXTagName::Ident("comment".into()), Drop::Delete);
        elements
      },
    },
  })
}
