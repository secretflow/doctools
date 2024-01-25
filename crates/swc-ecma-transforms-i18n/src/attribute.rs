use swc_core::{
  atoms::Atom,
  common::{sync::Lrc, util::take::Take as _, Spanned as _},
  ecma::ast::{Expr, Lit, ObjectLit, Tpl},
};
use swc_ecma_utils::{jsx::factory::JSXRuntime, span::with_span};

use crate::message::{Message, MessageProps};

fn translate_one(
  runtime: Lrc<JSXRuntime>,
  sym_gettext: Atom,
) -> impl FnMut(&mut Box<Expr>) -> Option<Message> {
  move |attr| {
    let expr = *attr.take();
    match expr {
      Expr::Lit(Lit::Str(lit)) => {
        let mut message = MessageProps::new(true);
        message.raw(lit.value.as_str(), lit.span());
        let (message, result) = message.make_i18n(runtime.clone(), sym_gettext.clone());
        *attr = result;
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
              let chunk = &quasis[i / 2];
              let text = match chunk.cooked {
                Some(ref text) => text,
                None => "",
              };
              message.raw(&text, chunk.span());
            }
            1 => {
              message.interpolate(exprs[i / 2].take());
            }
            _ => unreachable!(),
          };
        }
        let (message, result) = message.make_i18n(runtime.clone(), sym_gettext.clone());
        *attr = with_span(Some(span))(result);
        Some(message)
      }
      _ => {
        *attr = expr.into();
        None
      }
    }
  }
}

pub fn translate_attrs(
  runtime: Lrc<JSXRuntime>,
  sym_gettext: Atom,
  props: &mut ObjectLit,
  attrs: Vec<Vec<&str>>,
) -> Vec<Message> {
  attrs
    .iter()
    .filter_map(|path| {
      let message = runtime.mut_prop(
        props,
        path,
        translate_one(runtime.clone(), sym_gettext.clone()),
      );
      match message {
        Some(Some(message)) => Some(message),
        _ => None,
      }
    })
    .collect()
}
