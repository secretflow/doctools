use serde::Serialize;
use swc_core::{
  common::{FileName, SourceMap},
  ecma::{
    ast::CallExpr,
    visit::{as_folder, Fold, VisitMut, VisitMutWith},
  },
};

use deno_lite::{anyhow, export_function, DenoLite, ESModule};
use html5jsx::html_to_jsx;
use swc_ecma_utils::{
  jsx::{
    builder::JSXDocument,
    factory::{JSXRuntime, JSXTagName},
  },
  jsx_or_return,
};

static ESM: &str = include_str!("../../dist/index.js");

#[derive(Serialize)]
struct RenderMath {
  code: String,
  inline: bool,
}

export_function!(render, RenderMath);

struct MathRenderer {
  deno: DenoLite,
  module: ESModule,
  jsx: JSXRuntime,
}

impl MathRenderer {
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
    let document = html_to_jsx(&file, Some(self.jsx.clone()))
      .map_err(|err| anyhow::anyhow!("failed to parse math as JSX: {:?}", err))?;
    Ok(document)
  }
}

impl VisitMut for MathRenderer {
  fn visit_mut_call_expr(&mut self, elem: &mut CallExpr) {
    elem.visit_mut_children_with(self);

    let (name, props) = jsx_or_return!(self.jsx, elem);

    // TODO: match macro
    let inline = match name {
      JSXTagName::Ident(name) if &*name == "math" => true,
      JSXTagName::Ident(name) if &*name == "math_block" => false,
      _ => return,
    };

    let code = self.jsx.get_prop(props, &["children"]).as_string();

    let code = match code {
      Some(code) => code,
      None => return,
    };

    let document = self.render_math(code, inline);

    match document {
      Ok(_) => todo!(),
      Err(_) => todo!(),
    }
  }
}

pub fn render_math(jsx: JSXRuntime, deno: DenoLite) -> impl Fold + VisitMut {
  let mut deno = deno;
  let module = deno.load_module_once(ESM).unwrap();
  as_folder(MathRenderer { jsx, deno, module })
}
