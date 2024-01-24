use swc_core::ecma::{
  ast::CallExpr,
  visit::{noop_visit_mut_type, VisitMut, VisitMutWith as _},
};
use swc_ecma_utils::{jsx::factory::JSXFactory, jsx_or_pass};

struct BuiltInProps<'factory> {
  factory: &'factory JSXFactory,
}

impl VisitMut for BuiltInProps<'_> {
  noop_visit_mut_type!();

  fn visit_mut_call_expr(&mut self, call: &mut CallExpr) {
    jsx_or_pass!(self, self.factory, mut call);
  }
}
