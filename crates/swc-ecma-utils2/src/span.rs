use swc_core::{
  common::Span,
  ecma::visit::{noop_visit_mut_type, VisitMut, VisitMutWith},
};

struct SetSpan {
  span: Span,
}

impl VisitMut for SetSpan {
  noop_visit_mut_type!();

  fn visit_mut_span(&mut self, span: &mut Span) {
    if span.is_dummy() {
      *span = self.span;
    }
  }
}

#[allow(private_bounds)]
pub fn with_span<T: VisitMutWith<SetSpan>>(span: Option<Span>) -> impl Fn(T) -> T {
  move |mut node| {
    match span {
      Some(span) if !span.is_dummy() => {
        let mut v = SetSpan { span };
        node.visit_mut_with(&mut v);
      }
      _ => {}
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