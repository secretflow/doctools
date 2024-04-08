use std::marker::PhantomData;

use serde::{Deserialize, Serialize};
use swc_core::ecma::{
  ast::CallExpr,
  visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith},
};

use deno_lite::{anyhow, ESFunction, ESModule};
use html5jsx::html_str_to_jsx;
use swc_ecma_utils2::{
  ad_hoc_tag,
  jsx::{create_element, unpack::unpack_jsx, JSXDocument, JSXRuntime},
};

use crate::macros::basic_attributes;

#[derive(Serialize, ESFunction)]
struct RenderMath {
  tex: String,
  inline: bool,
}

#[basic_attributes(#[serde(default)])]
#[derive(Serialize, Deserialize)]
struct Math {
  #[serde(alias = "children")]
  tex: String,
  label: Option<String>,
  number: Option<u32>,
}

#[derive(Deserialize)]
enum SphinxMath {
  #[serde(rename = "math")]
  Inline(Math),
  #[serde(rename = "math_block")]
  Block(Math),
}

pub struct MathRenderer<R: JSXRuntime> {
  module: ESModule,
  jsx: PhantomData<R>,
}

impl<R: JSXRuntime> MathRenderer<R> {
  fn render_math(&mut self, tex: &str, inline: bool) -> anyhow::Result<JSXDocument> {
    let html: String = self.module.call_function(RenderMath {
      tex: tex.into(),
      inline,
    })?;
    let document = html_str_to_jsx::<R>(&*html)
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
      Ok(document) => create_element::<R>(ad_hoc_tag!(Math))
        .props(&props)
        .prop("inline", &inline)
        .child(document.to_fragment::<R>().into())
        .span(call.span)
        .build()?,
      Err(error) => create_element::<R>(ad_hoc_tag!(Math))
        .props(&props)
        .prop("inline", &inline)
        .prop("error", &format!("{}", error))
        .span(call.span)
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

pub fn render_math<R: JSXRuntime>(esm: &ESModule) -> impl Fold + VisitMut {
  as_folder(MathRenderer {
    module: esm.clone(),
    jsx: PhantomData::<R>,
  })
}
