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

pub fn with_span<T: VisitMutWith<SetSpan>>(span: Span) -> impl Fn(T) -> T {
    move |mut node| {
        let mut v = SetSpan { span };
        node.visit_mut_with(&mut v);
        node
    }
}
