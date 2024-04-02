use std::marker::PhantomData;

use deno_lite::{anyhow, ESFunction, ESModule};
use html5jsx::html_to_jsx;
use serde::{Deserialize, Serialize};
use sphinx_jsx_macros::basic_attributes;
use swc_core::{
  common::{util::take::Take as _, FileName, SourceMap, Span, Spanned},
  ecma::{
    ast::{CallExpr, Expr},
    visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith as _},
  },
};
use swc_ecma_utils2::{
  collections::{Mapping, MutableMapping, MutableSequence},
  jsx::{
    del_first_of_type, jsx_mut, tag::JSXTagMatch as _, JSXDocument, JSXElementMut, JSXRuntime,
  },
  jsx_tag, unpack_jsx, JSX,
};

use crate::move_basic_attributes;

struct CodeBlockRenderer<R: JSXRuntime> {
  module: ESModule,
  jsx: PhantomData<R>,
}

impl<R: JSXRuntime> VisitMut for CodeBlockRenderer<R> {
  noop_visit_mut_type!();

  fn visit_mut_call_expr(&mut self, call: &mut CallExpr) {
    call.visit_mut_children_with(self);

    let elem = unpack_jsx!(
      [LiteralBlockElement, R, call],
      [Content, attrs as LiteralBlock] = [jsx_tag!(literal_block?)],
      [Container as elem] = [
        jsx_tag!(container?),
        literal_block = true,
        has(jsx_tag!(CodeBlock?)),
      ],
    );

    let Some(elem) = elem else {
      return;
    };

    *call = match elem {
      LiteralBlockElement::Content { attrs } => {
        self.process_code_block(attrs, call.span()).expect("TODO:")
      }
      LiteralBlockElement::Container { elem } => self.process_container(elem),
    };
  }
}

impl<R: JSXRuntime> CodeBlockRenderer<R> {
  fn render_code_block(
    &mut self,
    code: String,
    lang: String,
    highlighted_lines: Option<Vec<usize>>,
  ) -> anyhow::Result<JSXDocument> {
    let html: String = self.module.call_function(RenderCode {
      code,
      lang,
      line_highlight: highlighted_lines,
    })?;
    let sources = SourceMap::default();
    let file = sources.new_source_file(FileName::Anon, html);
    let document = html_to_jsx::<R>(&file)
      .map_err(|err| anyhow::anyhow!("failed to parse math as JSX: {:?}", err))?;
    Ok(document)
  }

  fn process_code_block(&mut self, block: LiteralBlock, span: Span) -> anyhow::Result<CallExpr> {
    let LiteralBlock {
      code,
      language,
      show_line_numbers,
      line_options,
      ids,
      classes,
      names,
      dupnames,
    } = block;

    let lang = language.unwrap_or_else(|| "text".into());
    let lang = match_language(&lang).unwrap_or(&lang).to_lowercase();

    let start_line = line_options
      .as_ref()
      .and_then(|f| f.start_line)
      .unwrap_or(1);

    let show_line_numbers = show_line_numbers.unwrap_or(false);

    let document = self.render_code_block(
      code.clone(),
      lang.clone(),
      line_options.and_then(|f| f.highlighted),
    );

    let props = CodeBlockProps {
      code,
      lang,
      show_line_numbers,
      start_line,
      ids,
      classes,
      names,
      dupnames,
    };

    match document {
      Ok(document) => {
        let children = document.to_fragment::<R>();
        JSX!([CodeBlock, R, span], props, [children])
      }
      Err(error) => {
        let error = format!("{}", error);
        JSX!([CodeBlock, R, span], props, [error])
      }
    }
    .map_err(|err| anyhow::anyhow!("{}", err))
  }

  fn process_container(&mut self, mut elem: CallExpr) -> CallExpr {
    let Some(mut code_block) = del_first_of_type::<R>(&mut elem, jsx_tag!(CodeBlock?)) else {
      return elem;
    };

    if let Some(caption) = del_first_of_type::<R>(&mut elem, jsx_tag!(caption?)) {
      jsx_mut::<R>(&mut code_block)
        .get_props_mut()
        .set_item("caption", caption.into());
    }

    move_basic_attributes!(R, Expr(elem), code_block);

    code_block
  }
}

#[derive(Serialize, Deserialize)]
struct LineOptions {
  #[serde(alias = "hl_lines")]
  highlighted: Option<Vec<usize>>,
  #[serde(alias = "linenostart")]
  start_line: Option<usize>,
}

#[basic_attributes]
#[derive(Serialize, Deserialize)]
struct LiteralBlock {
  #[serde(alias = "children")]
  code: String,
  language: Option<String>,
  #[serde(alias = "linenos")]
  show_line_numbers: Option<bool>,
  #[serde(alias = "highlight_args")]
  line_options: Option<LineOptions>,
}

#[basic_attributes]
#[derive(Serialize, Deserialize)]
struct Container {}

#[basic_attributes]
#[derive(Serialize, Deserialize)]
struct Caption {
  children: Expr,
}

enum LiteralBlockElement {
  Content { attrs: LiteralBlock },
  Container { elem: CallExpr },
}

#[derive(Serialize, ESFunction)]
#[serde(rename_all = "camelCase")]
struct RenderCode {
  code: String,
  lang: String,
  line_highlight: Option<Vec<usize>>,
}

#[basic_attributes]
#[derive(Serialize)]
struct CodeBlockProps {
  code: String,
  lang: String,
  start_line: usize,
  show_line_numbers: bool,
}

pub fn render_code<R: JSXRuntime>(esm: &ESModule) -> impl Fold + VisitMut {
  as_folder(CodeBlockRenderer {
    module: esm.clone(),
    jsx: PhantomData::<R>,
  })
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
