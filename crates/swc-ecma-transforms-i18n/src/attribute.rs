use swc_core::{
  atoms::Atom,
  common::{util::take::Take as _, Spanned as _},
  ecma::ast::{Expr, Lit, ObjectLit, Tpl},
};
use swc_ecma_utils::{jsx::factory::JSXFactory, span::with_span};

use crate::message::{Message, MessageProps};

fn translate_one<'f>(
  factory: &'f JSXFactory,
  sym_gettext: &'f Atom,
) -> impl FnMut(&mut Box<Expr>) -> Option<Message> + 'f {
  |attr| {
    let expr = *attr.take();
    match expr {
      Expr::Lit(Lit::Str(lit)) => {
        let mut message = MessageProps::new(true);
        message.raw(lit.value.as_str(), lit.span());
        let (message, result) = message.make_i18n(factory, sym_gettext);
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
        let (message, result) = message.make_i18n(factory, sym_gettext);
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
  factory: &JSXFactory,
  sym_gettext: &Atom,
  props: &mut ObjectLit,
  attrs: Vec<Vec<&str>>,
) -> Vec<Message> {
  attrs
    .iter()
    .filter_map(|path| {
      let message = factory.mut_prop(props, path, translate_one(factory, sym_gettext));
      match message {
        Some(Some(message)) => Some(message),
        _ => None,
      }
    })
    .collect()
}
