use itertools::join;
use serde::{Deserialize, Serialize};
use swc_core::{
  atoms::Atom,
  ecma::{
    ast::CallExpr,
    visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith},
  },
};

use swc_ecma_utils::{ast::expr_to_json_lossy, jsx::factory::JSXFactory, jsx_or_pass};

struct BuiltInPropsVisitor<'factory> {
  factory: &'factory JSXFactory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Attributes {
  #[serde(default)]
  ids: Vec<Atom>,

  #[serde(default)]
  classes: Vec<Atom>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Props {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  id: Option<Atom>,

  #[serde(default, skip_serializing_if = "Option::is_none")]
  ids: Option<Atom>,

  #[serde(default, skip_serializing_if = "Option::is_none")]
  class_name: Option<Atom>,
}

impl VisitMut for BuiltInPropsVisitor<'_> {
  noop_visit_mut_type!();

  fn visit_mut_call_expr(&mut self, call: &mut CallExpr) {
    let (_, attrs) = jsx_or_pass!(self, self.factory, mut call);

    let attrs: Attributes = expr_to_json_lossy(&*attrs).unwrap();

    let props = Props {
      id: match attrs.ids.first() {
        Some(atom) => Some((&**atom).into()),
        None => None,
      },
      ids: if attrs.ids.len() > 1 {
        Some(join(&attrs.ids[1..], " ").into())
      } else {
        None
      },
      class_name: if attrs.classes.len() > 0 {
        Some(join(attrs.classes, " ").into())
      } else {
        None
      },
    };

    self.factory.replace_props(call, props).unwrap();

    call.visit_mut_children_with(self);
  }
}

pub fn built_in_props<'factory>(factory: &'factory JSXFactory) -> impl Fold + VisitMut + 'factory {
  as_folder(BuiltInPropsVisitor { factory })
}
