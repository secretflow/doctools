use swc_core::{
    common::util::take::Take as _,
    ecma::ast::{CallExpr, Expr, Lit, Str, Tpl},
};
use swc_utils::{
    ast::{mut_call_by_path, PropVisitorError},
    span::with_span,
};

use crate::message::{Message, MessageProps};

pub fn translate_attribute<'a>(
    call: &mut CallExpr,
    path: &[Lit],
    i18n: &'a str,
) -> Option<Message> {
    let path = [
        // argument 1, which is the props object
        &[Lit::from(1)],
        path,
    ]
    .concat();
    let result = mut_call_by_path(call, &path[..], |source| {
        let expr = *source.take();
        match expr {
            Expr::Lit(Lit::Str(Str { value, span, .. })) => {
                let mut message = MessageProps::new(true);
                let _ = message.text(&value);
                let (message, result) = message.make_i18n(i18n);
                *source = with_span(Some(span))(result);
                Some(message)
            }
            Expr::Tpl(Tpl {
                quasis,
                mut exprs,
                span,
            }) => {
                let mut message = MessageProps::new(true);
                let count = quasis.len() + exprs.len();
                for i in 0..count {
                    match i % 2 {
                        0 => {
                            let _ = message.text(&quasis[i / 2].raw);
                        }
                        1 => {
                            message.interpolate(exprs[i / 2].take());
                        }
                        _ => unreachable!(),
                    };
                }
                let (message, result) = message.make_i18n(i18n);
                *source = with_span(Some(span))(result);
                Some(message)
            }
            _ => {
                *source = expr.into();
                None
            }
        }
    });

    match result {
        Ok(message) => message,
        Err(PropVisitorError::NotFound) => None,
        _ => unreachable!("{:?}", result),
    }
}
