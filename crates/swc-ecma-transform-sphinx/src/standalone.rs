use std::marker::PhantomData;

use swc_core::ecma::{
  ast::CallExpr,
  visit::{VisitMut, VisitMutWith as _},
};
use swc_ecma_utils2::jsx::JSXRuntime;

struct StandaloneElements<R: JSXRuntime> {
  jsx: PhantomData<R>,
}

impl<R: JSXRuntime> VisitMut for StandaloneElements<R> {
  fn visit_mut_call_expr(&mut self, elem: &mut CallExpr) {
    elem.visit_mut_children_with(self);
  }
}
