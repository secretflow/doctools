use swc_core::{
  common::{chain, sync::Lrc, util::take::Take},
  ecma::{
    ast::{
      ArrayLit, CallExpr, Expr, ExprOrSpread, KeyValueProp, Lit, ObjectLit, Prop, PropOrSpread,
    },
    visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith},
  },
};

use crate::jsx_or_continue_visit;

use super::factory::{JSXRuntime, JSXTagName};

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
          Prop::KeyValue(KeyValueProp { ref value, .. }) => !value.is_invalid(),
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
        Some(ExprOrSpread { ref expr, .. }) => !expr.is_invalid(),
        _ => true,
      })
      .collect();
  }
}

struct FoldFragments {
  runtime: Lrc<JSXRuntime>,
}

impl VisitMut for FoldFragments {
  noop_visit_mut_type!();

  fn visit_mut_call_expr(&mut self, elem: &mut CallExpr) {
    let (tag, _) = jsx_or_continue_visit!(self, self.runtime, mut elem);

    elem.visit_mut_children_with(self);

    match tag {
      JSXTagName::Intrinsic(_) | JSXTagName::Ident(_) => {}
      JSXTagName::Fragment => {
        let (_, props) = self.runtime.as_jsx(elem).unwrap();
        let children = self.runtime.get_prop(props, &["children"]).get();
        match children {
          None => {
            elem.take();
          }
          Some(Expr::Array(ArrayLit { elems, .. })) => {
            if elems.iter().all(|i| match i {
              None => true,
              Some(ExprOrSpread { expr, .. }) => is_nullish(expr),
            }) {
              elem.take();
            };
          }
          Some(children) => {
            if is_nullish(children) {
              elem.take();
            }
          }
        }
      }
    }
  }
}

struct FixJSXFactory {
  runtime: Lrc<JSXRuntime>,
}

impl VisitMut for FixJSXFactory {
  fn visit_mut_call_expr(&mut self, elem: &mut CallExpr) {
    let (_, props) = jsx_or_continue_visit!(self, self.runtime, mut elem);
    let children = self.runtime.get_prop(props, &["children"]).get();
    match children {
      Some(Expr::Array(ArrayLit { elems, .. })) if elems.len() > 1 => {
        elem.callee = self.runtime.jsxs()
      }
      _ => elem.callee = self.runtime.jsx(),
    }
  }
}

fn is_nullish(expr: &Expr) -> bool {
  match expr {
    Expr::Lit(Lit::Null(_)) | Expr::Invalid(_) => true,
    Expr::Ident(ident) if ident.sym == "undefined" => true,
    _ => false,
  }
}

pub fn remove_invalid() -> impl Fold + VisitMut {
  as_folder(CleanUpTakenValues)
}

pub fn fold_fragments(runtime: Lrc<JSXRuntime>) -> impl Fold + VisitMut {
  as_folder(FoldFragments { runtime })
}

pub fn fix_jsx_factories(runtime: Lrc<JSXRuntime>) -> impl Fold + VisitMut {
  as_folder(FixJSXFactory { runtime })
}

pub fn sanitize_jsx(runtime: Lrc<JSXRuntime>) -> impl Fold + VisitMut {
  chain!(
    fold_fragments(runtime.clone()),
    remove_invalid(),
    fix_jsx_factories(runtime),
  )
}
