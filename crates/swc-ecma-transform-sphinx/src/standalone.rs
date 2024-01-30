use swc_core::ecma::{
  ast::CallExpr,
  visit::{VisitMut, VisitMutWith as _},
};
use swc_ecma_utils::{jsx::factory::JSXRuntime, match_tag, tag};

struct StandaloneElements {
  jsx: JSXRuntime,
}

impl VisitMut for StandaloneElements {
  fn visit_mut_call_expr(&mut self, elem: &mut CallExpr) {
    elem.visit_mut_children_with(self);

    let name = match_tag!(
      (self.jsx, elem),
      JSX(attribution) >> { tag!(<Attribution>) },
      JSX(block_quote) >> { tag!(<Blockquote>) },
      JSX(bullet_list) >> { tag!(<BulletList>) },
      _ >> { return },
    );
  }
}
