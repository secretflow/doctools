use swc_core::{
    common::Span,
    ecma::visit::{noop_visit_mut_type, VisitMut, VisitMutWith},
};

pub struct SetSpan {
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

pub fn with_span<T: VisitMutWith<SetSpan>>(span: Option<Span>) -> impl Fn(T) -> T {
    move |mut node| {
        match span {
            Some(span) => {
                let mut v = SetSpan { span };
                node.visit_mut_with(&mut v);
            }
            None => {}
        }
        node
    }
}

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
