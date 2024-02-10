use swc_core::{
  common::{util::take::Take as _, Spanned as _},
  ecma::ast::{Expr, Lit, ObjectLit, Tpl},
};
use swc_ecma_utils2::{collections::MutableMapping as _, jsx::JSXRuntime, span::with_span};

use crate::{
  message::{Message, MessageProps},
  symbols::I18nSymbols,
};

fn translate_one<R: JSXRuntime, S: I18nSymbols>(attr: &mut Expr) -> Option<Message> {
  let expr = attr.take();

  match expr {
    Expr::Lit(Lit::Str(lit)) => {
      let mut message = MessageProps::new(true);
      message.raw(lit.value.as_str(), lit.span());
      let (message, result) = message.make_i18n::<R, S>();
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
            message.interpolate(*exprs[i / 2].take());
          }
          _ => unreachable!(),
        };
      }
      let (message, result) = message.make_i18n::<R, S>();
      *attr = with_span(Some(span))(result);
      Some(message)
    }

    _ => {
      *attr = expr.into();
      None
    }
  }
}

pub fn translate_attrs<R: JSXRuntime, S: I18nSymbols>(
  props: &mut ObjectLit,
  attrs: Vec<Vec<&str>>,
) -> Vec<Message> {
  attrs
    .iter()
    .filter_map(|path| {
      let Some(value) = props.get_item_mut_at_path(path.iter().map(|s| *s)) else {
        return None;
      };
      translate_one::<R, S>(value)
    })
    .collect()
}
