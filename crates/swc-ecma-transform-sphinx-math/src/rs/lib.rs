use std::marker::PhantomData;

use serde::{Deserialize, Serialize};
use swc_core::{
  common::{FileName, SourceMap},
  ecma::{
    ast::CallExpr,
    visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith},
  },
};

use deno_lite::{anyhow, esm_source, DenoLite, ESFunction, ESModule};
use html5jsx::html_to_jsx;
use sphinx_jsx_macros::basic_attributes;
use swc_ecma_utils2::{
  jsx::{JSXDocument, JSXRuntime},
  jsx_tag, unpack_jsx, JSX,
};

esm_source!(SERVER, "render-math", "../../dist/server/index.js");

#[derive(Serialize, ESFunction)]
struct RenderMath {
  tex: String,
  inline: bool,
}

#[basic_attributes]
#[derive(Serialize, Deserialize)]
struct MathProps {
  #[serde(alias = "children")]
  tex: String,
  label: Option<String>,
  number: Option<u32>,
}

enum Math {
  Inline { props: MathProps },
  Block { props: MathProps },
}

struct MathRenderer<R: JSXRuntime> {
  module: ESModule,
  deno: DenoLite,
  jsx: PhantomData<R>,
}

impl<R: JSXRuntime> MathRenderer<R> {
  fn render_math(&mut self, tex: &str, inline: bool) -> anyhow::Result<JSXDocument> {
    let html: String = self.deno.call_function(
      self.module,
      RenderMath {
        tex: tex.into(),
        inline,
      },
    )?;
    let sources = SourceMap::default();
    let file = sources.new_source_file(FileName::Anon, html);
    let document = html_to_jsx::<R>(&file)
      .map_err(|err| anyhow::anyhow!("failed to parse math as JSX: {:?}", err))?;
    Ok(document)
  }

  fn process_call_expr(&mut self, call: &mut CallExpr) -> Option<()> {
    let math = unpack_jsx!(
      [Math, R, call],
      jsx_tag!(math?) = [Inline, props as MathProps],
      jsx_tag!(math_block?) = [Block, props as MathProps],
    )?;

    let (inline, props) = match math {
      Math::Inline { props } => (true, props),
      Math::Block { props } => (false, props),
    };

    let document = self.render_math(&props.tex, inline);

    *call = match document {
      Ok(document) => {
        let children = document.to_fragment::<R>();
        JSX!([Math, R, call.span], props, [inline, children])
      }
      Err(error) => {
        let error = format!("{}", error);
        JSX!([Math, R, call.span], props, [inline, error])
      }
    }
    .ok()?;

    Some(())
  }
}

impl<R: JSXRuntime> VisitMut for MathRenderer<R> {
  noop_visit_mut_type!();

  fn visit_mut_call_expr(&mut self, call: &mut CallExpr) {
    call.visit_mut_children_with(self);
    self.process_call_expr(call);
  }
}

pub fn render_math<R: JSXRuntime>(deno: DenoLite) -> impl Fold + VisitMut {
  let mut deno = deno;
  let module = SERVER.load_into(&mut deno).unwrap();
  as_folder(MathRenderer::<R> {
    module,
    deno,
    jsx: PhantomData,
  })
}