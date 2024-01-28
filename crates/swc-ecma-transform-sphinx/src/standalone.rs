use swc_core::ecma::{
  ast::CallExpr,
  visit::{VisitMut, VisitMutWith as _},
};
use swc_ecma_utils::{jsx::factory::JSXRuntime, jsx_or_return, tag};

struct StandaloneElements {
  jsx: JSXRuntime,
}

impl VisitMut for StandaloneElements {
  fn visit_mut_call_expr(&mut self, elem: &mut CallExpr) {
    elem.visit_mut_children_with(self);
    jsx_or_return!(self.jsx, elem);

    let name = match self.jsx.as_jsx(elem) {
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
