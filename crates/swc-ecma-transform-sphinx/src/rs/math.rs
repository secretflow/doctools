use std::{borrow::Cow, marker::PhantomData};

use serde::{Deserialize, Serialize};
use swc_core::{
  common::{sync::Lrc, FileName, SourceMap},
  ecma::{
    ast::CallExpr,
    visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith},
  },
};

use deno_lite::{anyhow, ESFunction, ESModule};
use html5jsx::html_to_jsx;
use swc_ecma_utils2::{
  jsx::{replace_element, unpack_jsx, JSXDocument, JSXRuntime, TextNode},
  span::with_span,
};

use crate::{components::Transformed, macros::basic_attributes};

#[derive(Serialize, ESFunction)]
struct RenderMath {
  tex: String,
  inline: bool,
}

#[basic_attributes(#[serde(default)])]
#[derive(Serialize, Deserialize)]
struct Math<'ast> {
  #[serde(alias = "children")]
  #[serde(deserialize_with = "TextNode::into_cow")]
  #[serde(borrow)]
  tex: Cow<'ast, str>,
  label: Option<String>,
  number: Option<u32>,
}

#[derive(Deserialize)]
enum SphinxMath<'ast> {
  #[serde(rename = "math")]
  #[serde(borrow)]
  Inline(Math<'ast>),
  #[serde(rename = "math_block")]
  #[serde(borrow)]
  Block(Math<'ast>),
}

pub struct MathRenderer<R: JSXRuntime> {
  sourcemap: Lrc<SourceMap>,
  esm: ESModule,
  jsx: PhantomData<R>,
}

impl<R: JSXRuntime> MathRenderer<R> {
  fn render_math(&mut self, tex: &str, inline: bool) -> anyhow::Result<JSXDocument> {
    let html: String = self.esm.call_function(RenderMath {
      tex: tex.into(),
      inline,
    })?;
    let html = self.sourcemap.new_source_file(FileName::Anon, html);
    let document = html_to_jsx::<R>(&html)
      .map_err(|err| anyhow::anyhow!("failed to parse math as JSX: {:?}", err))?;
    Ok(document)
  }

  fn process_call_expr(&mut self, call: &mut CallExpr) -> anyhow::Result<()> {
    let Ok(math) = unpack_jsx::<R, SphinxMath>(call) else {
      return Ok(());
    };

    let (inline, props) = match math {
      SphinxMath::Inline(props) => (true, props),
      SphinxMath::Block(props) => (false, props),
    };

    let document = self.render_math(&props.tex, inline);

    *call = match document {
      Ok(document) => replace_element::<R, _>(call, Transformed::Math, &props)
        .prop(Default::default(), "inline", &inline)
        .child(with_span(call.span)(document.to_fragment::<R>().into()))
        .build()?,
      Err(error) => replace_element::<R, _>(call, Transformed::Math, &props)
        .prop(Default::default(), "inline", &inline)
        .prop(Default::default(), "error", &format!("{}", error))
        .build()?,
    };

    Ok(())
  }
}

impl<R: JSXRuntime> VisitMut for MathRenderer<R> {
  noop_visit_mut_type!();

  fn visit_mut_call_expr(&mut self, call: &mut CallExpr) {
    call.visit_mut_children_with(self);
    match self.process_call_expr(call) {
      Ok(()) => {}
      Err(_) => todo!(),
    }
  }
}

pub fn render_math<R: JSXRuntime>(
  sourcemap: Lrc<SourceMap>,
  esm: &ESModule,
) -> impl Fold + VisitMut {
  as_folder(MathRenderer {
    sourcemap,
    esm: esm.clone(),
    jsx: PhantomData::<R>,
  })
}
