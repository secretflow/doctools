use swc_core::{
  common::sync::Lrc,
  ecma::{
    ast::CallExpr,
    visit::{VisitMut, VisitMutWith as _},
  },
};
use swc_ecma_utils::{jsx::factory::JSXRuntime, jsx_or_return, tag};

struct StandaloneElements {
  runtime: Lrc<JSXRuntime>,
}

impl VisitMut for StandaloneElements {
  fn visit_mut_call_expr(&mut self, call: &mut CallExpr) {
    call.visit_mut_children_with(self);
    jsx_or_return!(self.runtime, call);

    let name = match self.runtime.as_jsx(call) {
      Some((tag!(let name), _)) => name,
      _ => return,
    };

    let _ = match &*name {
      "attribution" => tag!(<Attribution>),
      "block_quote" => tag!(<Blockquote>),
      "bullet_list" => tag!(<BulletList>),
      _ => return,
    };
  }
}
