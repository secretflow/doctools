use std::marker::PhantomData;

use serde::Deserialize;
use swc_core::{
  common::util::take::Take,
  ecma::{
    ast::CallExpr,
    visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith},
  },
};
use swc_ecma_utils2::jsx::{jsx_builder2, unpack::unpack_jsx, JSXRuntime};

use crate::{components::Transformed, macros::basic_attributes};

#[basic_attributes(#[serde(default)])]
#[derive(Deserialize)]
enum ParagraphElement {
  #[serde(rename = "paragraph")]
  Paragraph,
  #[serde(rename = "strong")]
  Strong,
  #[serde(rename = "emphasis")]
  Emphasis,
  #[serde(rename = "literal")]
  Literal,
  #[serde(rename = "transition")]
  Transition,
}

struct TransformParagraph<R: JSXRuntime> {
  jsx: PhantomData<R>,
}

impl<R: JSXRuntime> VisitMut for TransformParagraph<R> {
  noop_visit_mut_type!();

  fn visit_mut_call_expr(&mut self, call: &mut CallExpr) {
    call.visit_mut_children_with(self);
    match unpack_jsx::<R, ParagraphElement>(call) {
      Err(err) => {
        dbg!(err);
      }
      Ok(ParagraphElement::Paragraph { .. }) => {
        *call = jsx_builder2::<R>(call.take())
          .tag(Transformed::Paragraph)
          .guarantee();
      }
      Ok(ParagraphElement::Strong { .. }) => {
        *call = jsx_builder2::<R>(call.take())
          .tag(Transformed::Strong)
          .guarantee();
      }
      Ok(ParagraphElement::Emphasis { .. }) => {
        *call = jsx_builder2::<R>(call.take())
          .tag(Transformed::Emphasis)
          .guarantee();
      }
      Ok(ParagraphElement::Literal { .. }) => {
        *call = jsx_builder2::<R>(call.take())
          .tag(Transformed::Code)
          .guarantee();
      }
      Ok(ParagraphElement::Transition { .. }) => {
        *call = jsx_builder2::<R>(call.take())
          .tag(Transformed::HorizontalRule)
          .guarantee();
      }
    }
  }
}

pub fn render_typograph<R: JSXRuntime>() -> impl Fold + VisitMut {
  as_folder(TransformParagraph::<R> { jsx: PhantomData })
}
