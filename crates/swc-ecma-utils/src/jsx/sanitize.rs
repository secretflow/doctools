use swc_core::{
  common::{chain, util::take::Take},
  ecma::{
    ast::{
      ArrayLit, CallExpr, Expr, ExprOrSpread, KeyValueProp, Lit, ObjectLit, Prop, PropOrSpread,
    },
    visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith},
  },
};

use crate::jsx_or_return;

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

fn is_invalid(value: &Box<Expr>) -> bool {
  match **value {
    Expr::Invalid(_) => true,
    Expr::Call(ref call) => call.callee.is_super_(),
    _ => false,
  }
}

struct FoldFragments {
  jsx: JSXRuntime,
}

impl FoldFragments {
  fn should_remove_child(&self, elem: &Expr) -> bool {
    match elem {
      Expr::Call(call) => {
        let jsx = self.jsx.as_jsx(call);
        match jsx {
          None => false,
          Some((tag, _)) => match tag {
            JSXTagName::Intrinsic(_) | JSXTagName::Ident(_) => false,
            JSXTagName::Fragment => {
              let (_, props) = self.jsx.as_jsx(call).unwrap();
              let children = self.jsx.get_prop(props, &["children"]).get();
              match children {
                None => true,
                Some(Expr::Array(ArrayLit { elems, .. })) => elems.iter().all(|i| match i {
                  None => true,
                  Some(ExprOrSpread { expr, .. }) => is_nullish(expr),
                }),
                Some(children) => is_nullish(children),
              }
            }
          },
        }
      }
      _ => is_nullish(elem),
    }
  }
}

impl VisitMut for FoldFragments {
  noop_visit_mut_type!();

  fn visit_mut_call_expr(&mut self, elem: &mut CallExpr) {
    elem.visit_mut_children_with(self);

    jsx_or_return!(self.jsx, elem);

    self.jsx.mut_prop(
      self.jsx.as_mut_jsx_props(elem).unwrap(),
      &["children"],
      |children| match **children {
        Expr::Array(ref mut children) => {
          children.elems = children
            .elems
            .drain(..)
            .filter(|elem| match elem {
              None => false,
              Some(ExprOrSpread { expr, .. }) => !self.should_remove_child(&*expr),
            })
            .collect();
        }
        ref mut child => {
          if self.should_remove_child(&child) {
            child.take();
          }
        }
      },
    );

    let (tag, _) = self.jsx.as_jsx(elem).unwrap();

    if tag == JSXTagName::Fragment {
      let swap = self.jsx.mut_prop(
        self.jsx.as_mut_jsx_props(elem).unwrap(),
        &["children"],
        |children| match **children {
          Expr::Array(ref mut children) => {
            if children.elems.len() > 1 {
              None
            } else {
              match children.elems.first_mut() {
                None => None,
                Some(None) => None,
                Some(Some(ExprOrSpread {
                  ref mut expr,
                  spread,
                })) => {
                  if spread.is_some() {
                    None
                  } else {
                    match expr.as_mut_call() {
                      Some(call) => Some(call.take()),
                      None => None,
                    }
                  }
                }
              }
            }
          }
          ref mut child => match child.as_mut_call() {
            Some(call) => Some(call.take()),
            None => None,
          },
        },
      );

      match swap {
        Some(Some(swap)) => *elem = swap,
        _ => {}
      }
    }
  }
}

struct FixJSXFactory {
  jsx: JSXRuntime,
}

impl VisitMut for FixJSXFactory {
  fn visit_mut_call_expr(&mut self, elem: &mut CallExpr) {
    elem.visit_mut_children_with(self);

    let (_, props) = jsx_or_return!(self.jsx, elem);

    let children = self.jsx.get_prop(props, &["children"]).get();

    match children {
      Some(Expr::Array(ArrayLit { elems, .. })) => {
        let len = elems.len();
        if len > 1 {
          elem.callee = self.jsx.jsxs()
        } else {
          elem.callee = self.jsx.jsx()
        }
        if len == 1 {
          self.jsx.mut_prop(
            self.jsx.as_mut_jsx_props(elem).unwrap(),
            &["children"],
            |children| {
              let child = children.as_mut_array().unwrap().elems.first_mut().unwrap();
              match child {
                None => {}
                Some(ExprOrSpread { expr, spread }) => {
                  if spread.is_none() {
                    *children = expr.take();
                  }
                }
              }
            },
          );
        }
      }
      _ => elem.callee = self.jsx.jsx(),
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

pub fn fold_fragments(jsx: JSXRuntime) -> impl Fold + VisitMut {
  as_folder(FoldFragments { jsx })
}

pub fn fix_jsx_factories(jsx: JSXRuntime) -> impl Fold + VisitMut {
  as_folder(FixJSXFactory { jsx })
}

pub fn sanitize_jsx(jsx: JSXRuntime) -> impl Fold + VisitMut {
  chain!(
    fold_fragments(jsx.clone()),
    remove_invalid(),
    fix_jsx_factories(jsx),
  )
}
