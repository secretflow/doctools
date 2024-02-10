use std::marker::PhantomData;

use serde::Serialize;
use swc_core::{
  common::{FileName, SourceMap},
  ecma::{
    ast::CallExpr,
    visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith},
  },
};

use deno_lite::{anyhow, export_function, DenoLite, ESModule};
use html5jsx::html_to_jsx;
use swc_ecma_utils2::{
  collections::Mapping,
  ecma::itertools::as_string,
  jsx::{jsx, tag::JSXTag, JSXDocument, JSXElement, JSXRuntime},
  Object, JSX,
};

static SERVER: &str = include_str!("../../dist/server/index.js");

static CLIENT_DTS: &str = include_str!("../js/client/index.d.ts");

#[derive(Serialize)]
struct RenderMath {
  code: String,
  inline: bool,
}

export_function!(render, RenderMath);

struct MathRenderer<R: JSXRuntime> {
  deno: DenoLite,
  module: ESModule,
  jsx: PhantomData<R>,
}

impl<R: JSXRuntime> MathRenderer<R> {
  fn render_math(&mut self, code: &str, inline: bool) -> anyhow::Result<JSXDocument> {
    let html: String = self.deno.call_function(
      self.module,
      RenderMath {
        code: String::from(code),
        inline,
      },
    )?;
    let sources = SourceMap::default();
    let file = sources.new_source_file(FileName::Anon, html);
    let document = html_to_jsx::<R>(&file)
      .map_err(|err| anyhow::anyhow!("failed to parse math as JSX: {:?}", err))?;
    Ok(document)
  }
}

impl<R: JSXRuntime> VisitMut for MathRenderer<R> {
  noop_visit_mut_type!();

  fn visit_mut_call_expr(&mut self, call: &mut CallExpr) {
    call.visit_mut_children_with(self);

    let Some(tag) = jsx::<R>(call).get_tag() else {
      return;
    };

    let inline = match tag.tuple() {
      (JSXTag::Component(_), "math") => true,
      (JSXTag::Component(_), "math_block") => false,
      _ => return,
    };

    let Some(tex) = jsx::<R>(call)
      .and_then(|e| e.get_props())
      .and_then(|e| e.get_item("children"))
      .and_then(as_string)
    else {
      return;
    };

    let document = self.render_math(tex, inline);

    *call = match document {
      Ok(document) => {
        let children = document.to_fragment::<R>();
        JSX!(
          [(Math), R],
          Object!("tex" = tex, "inline" = inline, "children" = children)
        )
      }
      Err(error) => {
        JSX!(
          [(Math), R],
          Object!(
            "tex" = tex,
            "inline" = inline,
            "error" = format!("{}", error)
          )
        )
      }
    };
  }
}

pub fn render_math<R: JSXRuntime>(deno: DenoLite) -> impl Fold + VisitMut {
  let mut deno = deno;
  let module = deno.load_module_once(SERVER).unwrap();
  as_folder(MathRenderer::<R> {
    deno,
    module,
    jsx: PhantomData,
  })
}

pub fn dts() -> &'static str {
  CLIENT_DTS
}
