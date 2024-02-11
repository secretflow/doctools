use std::marker::PhantomData;

use serde::{Deserialize, Serialize};
use swc_core::{
  common::{FileName, SourceMap},
  ecma::{
    ast::CallExpr,
    visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith},
  },
};

use deno_lite::{anyhow, define_deno_export, DenoLite, ESModule};
use html5jsx::html_to_jsx;
use swc_ecma_utils2::{
  jsx::{JSXDocument, JSXRuntime},
  jsx_tag, unpack_jsx, Object, JSX,
};

static SERVER: &str = include_str!("../../dist/server/index.js");

static CLIENT_DTS: &str = include_str!("../js/client/index.d.ts");

#[derive(Serialize)]
struct RenderMath {
  code: String,
  inline: bool,
}

define_deno_export!(render, RenderMath);

#[derive(Deserialize)]
struct MathProps {
  #[serde(default)]
  ids: Vec<String>,
  #[serde(default)]
  backrefs: Vec<String>,
  #[serde(default)]
  names: Vec<String>,
  #[serde(default)]
  classes: Vec<String>,

  label: Option<String>,
  number: Option<f64>,

  #[serde(rename = "children")]
  tex: String,
}

enum Math {
  Inline(MathProps),
  Block(MathProps),
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
        code: tex.into(),
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
      [call, Math, R],
      jsx_tag!(math?) = [Math::Inline, MathProps],
      jsx_tag!(math_block?) = [Math::Block, MathProps],
    )?;

    let (inline, data) = match math {
      Math::Inline(props) => (true, props),
      Math::Block(props) => (false, props),
    };

    let MathProps {
      tex,
      label,
      number,
      ids,
      backrefs,
      classes,
      ..
    } = data;

    let document = self.render_math(&tex, inline);

    *call = match document {
      Ok(document) => {
        let children = document.to_fragment::<R>();
        JSX!(
          [Math, R],
          Object![
            [tex, inline, children],
            ["label"? = label],
            ["number"? = number]
          ]
        )
      }
      Err(error) => {
        JSX!(
          [Math, R],
          Object![[tex, inline], ["error" = format!("{}", error)]]
        )
      }
    };

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
  let module = deno.load_module_once(SERVER).unwrap();
  as_folder(MathRenderer::<R> {
    module,
    deno,
    jsx: PhantomData,
  })
}

pub fn dts() -> &'static str {
  CLIENT_DTS
}
