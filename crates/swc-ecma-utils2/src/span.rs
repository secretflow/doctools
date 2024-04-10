use swc_core::{
  common::Span,
  ecma::{
    ast::Expr,
    visit::{noop_visit_mut_type, VisitMut, VisitMutWith},
  },
};

struct SetSpan {
  span: Option<Span>,
}

impl VisitMut for SetSpan {
  noop_visit_mut_type!();

  fn visit_mut_expr(&mut self, expr: &mut Expr) {
    if self.span.is_some() {
      expr.visit_mut_children_with(self)
    }
  }

  fn visit_mut_span(&mut self, span: &mut Span) {
    if span.is_dummy() {
      if let Some(new_span) = self.span.take() {
        *span = new_span;
      }
    }
  }
}

#[allow(private_bounds)]
pub fn with_span<T: VisitMutWith<SetSpan>>(span: Span) -> impl Fn(T) -> T {
  move |mut node| {
    if !span.is_dummy() {
      let mut v = SetSpan { span: Some(span) };
      node.visit_mut_with(&mut v);
    }
    node
  }
}

/// See also <https://rustdoc.swc.rs/swc_common/source_map/struct.SourceMap.html#method.merge_spans>
pub fn union_span(low: Span, high: Span) -> Span {
  if low.lo() > high.hi() {
    unreachable!()
  }
  if low.is_dummy() {
    return high;
  }
  if high.is_dummy() {
    return low;
  }
  Span::new(low.lo(), high.hi(), high.ctxt())
}
