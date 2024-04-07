use swc_core::ecma::{
  ast::{ArrayLit, ExprOrSpread, KeyValueProp, ObjectLit, Prop, PropOrSpread},
  visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith},
};

use super::itertools::is_invalid;

struct CleanUpTakenValues;

impl VisitMut for CleanUpTakenValues {
  noop_visit_mut_type!();

  fn visit_mut_object_lit(&mut self, object: &mut ObjectLit) {
    object.visit_mut_children_with(self);

    object.props = object
      .props
      .drain(..)
      .filter(|prop| match prop {
        PropOrSpread::Prop(prop) => match **prop {
          Prop::KeyValue(KeyValueProp { ref value, .. }) => !is_invalid(value),
          _ => true,
        },
        _ => true,
      })
      .collect();
  }

  fn visit_mut_array_lit(&mut self, array: &mut ArrayLit) {
    array.visit_mut_children_with(self);

    array.elems = array
      .elems
      .drain(..)
      .filter(|elem| match elem {
        Some(ExprOrSpread { ref expr, .. }) => !is_invalid(expr),
        _ => true,
      })
      .collect();
  }
}

pub fn remove_invalid() -> impl Fold + VisitMut {
  as_folder(CleanUpTakenValues)
}
