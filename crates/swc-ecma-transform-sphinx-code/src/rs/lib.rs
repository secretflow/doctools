use std::marker::PhantomData;

use deno_lite::{anyhow, esm_source, DenoLite, ESFunction, ESModule};
use html5jsx::html_to_jsx;
use serde::{Deserialize, Serialize};
use sphinx_jsx_macros::basic_attributes;
use swc_core::{
  common::{FileName, SourceMap},
  ecma::{
    ast::CallExpr,
    visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith as _},
  },
};
use swc_ecma_utils2::{
  jsx::{JSXDocument, JSXRuntime},
  jsx_tag, unpack_jsx, JSX,
};

esm_source!(SERVER, "render-code", "../../dist/server/index.js");

#[derive(Serialize, Deserialize)]
struct LineHighlight {
  hl_lines: Option<Vec<usize>>,
  linenostart: Option<usize>,
}

#[basic_attributes]
#[derive(Serialize, Deserialize)]
struct LiteralBlock {
  #[serde(alias = "children")]
  code: String,
  language: Option<String>,
  linenos: Option<bool>,
  highlight_args: Option<LineHighlight>,
}

enum Literal {
  Block { attrs: LiteralBlock },
}

#[derive(Serialize, ESFunction)]
#[serde(rename_all = "camelCase")]
struct RenderCode {
  code: String,
  lang: String,
  highlighted_lines: Option<Vec<usize>>,
}

#[basic_attributes]
#[derive(Serialize)]
struct CodeBlock {
  code: String,
  lang: String,
  start_line: usize,
  show_line_numbers: bool,
}

struct CodeBlockRenderer<R: JSXRuntime> {
  module: ESModule,
  deno: DenoLite,
  jsx: PhantomData<R>,
}

fn match_language(lang: &String) -> Option<&'static str> {
  match &*lang.to_lowercase() {
    "python" | "py" | "python3" | "ipython" | "ipython3" => Some("python"),
    "javascript" | "js" => Some("javascript"),
    "typescript" | "ts" => Some("typescript"),
    "jsx" | "javascriptreact" => Some("jsx"),
    "tsx" | "typescriptreact" => Some("tsx"),
    "rust" | "rs" => Some("rust"),
    "sql" | "mysql" | "sqlite" | "postgresql" => Some("sql"),
    "proto" | "protobuf" | "proto3" => Some("proto"),
    "go" | "golang" => Some("go"),
    "markdown" | "md" => Some("markdown"),
    "cpp" | "c++" => Some("cpp"),
    _ => None,
  }
}

impl<R: JSXRuntime> CodeBlockRenderer<R> {
  fn render_code_block(
    &mut self,
    code: String,
    lang: String,
    highlighted_lines: Option<Vec<usize>>,
  ) -> anyhow::Result<JSXDocument> {
    let html: String = self.deno.call_function(
      self.module,
      RenderCode {
        code,
        lang,
        highlighted_lines,
      },
    )?;
    let sources = SourceMap::default();
    let file = sources.new_source_file(FileName::Anon, html);
    let document = html_to_jsx::<R>(&file)
      .map_err(|err| anyhow::anyhow!("failed to parse math as JSX: {:?}", err))?;
    Ok(document)
  }

  fn process_call_expr(&mut self, call: &mut CallExpr) -> Option<()> {
    let elem = unpack_jsx!(
      [Literal, R, call],
      jsx_tag!(literal_block?) = [Block, attrs as LiteralBlock],
    )?;

    let Literal::Block {
      attrs:
        LiteralBlock {
          code,
          language,
          linenos,
          highlight_args,
          ids,
          classes,
          names,
          dupnames,
        },
    } = elem;

    let lang = language.unwrap_or_else(|| "text".into());
    let lang = match_language(&lang).unwrap_or(&lang).to_lowercase();

    let start_line = highlight_args
      .as_ref()
      .and_then(|f| f.linenostart)
      .unwrap_or(1);

    let show_line_numbers = linenos.unwrap_or(false);

    let document = self.render_code_block(
      code.clone(),
      lang.clone(),
      highlight_args.and_then(|f| f.hl_lines),
    );

    let props = CodeBlock {
      code,
      lang,
      show_line_numbers,
      start_line,
      ids,
      classes,
      names,
      dupnames,
    };

    *call = match document {
      Ok(document) => {
        let children = document.to_fragment::<R>();
        JSX!([CodeBlock, R, call.span], props, [children])
      }
      Err(error) => {
        let error = format!("{}", error);
        JSX!([CodeBlock, R, call.span], props, [error])
      }
    }
    .ok()?;

    Some(())
  }
}

impl<R: JSXRuntime> VisitMut for CodeBlockRenderer<R> {
  noop_visit_mut_type!();

  fn visit_mut_call_expr(&mut self, call: &mut CallExpr) {
    call.visit_mut_children_with(self);
    self.process_call_expr(call);
  }
}

pub fn render_code<R: JSXRuntime>(deno: DenoLite) -> impl Fold + VisitMut {
  let mut deno = deno;
  let module = SERVER.load_into(&mut deno).unwrap();
  as_folder(CodeBlockRenderer::<R> {
    module,
    deno,
    jsx: PhantomData,
  })
}
