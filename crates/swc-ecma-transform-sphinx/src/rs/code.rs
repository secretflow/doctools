use std::marker::PhantomData;

use deno_lite::{anyhow, ESFunction, ESModule};
use html5jsx::html_to_jsx;
use serde::{Deserialize, Serialize};
use sphinx_jsx_macros::basic_attributes;
use swc_core::{
  common::{FileName, SourceMap},
  ecma::{
    ast::{CallExpr, Expr},
    visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith as _},
  },
};
use swc_ecma_utils2::{
  collections::{MutableMapping, MutableSequence},
  jsx::{
    jsx, jsx_mut,
    tag::{JSXTagMatch, JSXTagType},
    JSXDocument, JSXElement, JSXElementMut, JSXRuntime,
  },
  jsx_tag, unpack_jsx, JSX,
};

use crate::move_basic_attributes;

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

#[basic_attributes]
#[derive(Serialize, Deserialize)]
struct Container {
  #[serde(deserialize_with = "Container::is_literal_block")]
  literal_block: bool,
}

impl Container {
  fn is_literal_block<'de, D>(de: D) -> Result<bool, D::Error>
  where
    D: serde::de::Deserializer<'de>,
  {
    let result = bool::deserialize(de)?;
    if result == true {
      Ok(result)
    } else {
      Err(serde::de::Error::custom("expected true"))
    }
  }
}

#[basic_attributes]
#[derive(Serialize, Deserialize)]
struct Caption {
  children: Expr,
}

enum CodeBlockRoot {
  Content { attrs: LiteralBlock },
  Container { attrs: Container, children: Expr },
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
struct CodeBlockProps {
  code: String,
  lang: String,
  start_line: usize,
  show_line_numbers: bool,
}

struct CodeBlockRenderer<R: JSXRuntime> {
  module: ESModule,
  jsx: PhantomData<R>,
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
      highlighted_lines,
    })?;
    let sources = SourceMap::default();
    let file = sources.new_source_file(FileName::Anon, html);
    let document = html_to_jsx::<R>(&file)
      .map_err(|err| anyhow::anyhow!("failed to parse math as JSX: {:?}", err))?;
    Ok(document)
  }

  fn process_code_block(&mut self, call: &mut CallExpr, block: LiteralBlock) -> Option<()> {
    let LiteralBlock {
      code,
      language,
      linenos,
      highlight_args,
      ids,
      classes,
      names,
      dupnames,
    } = block;

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

  fn process_container(
    &mut self,
    call: &mut CallExpr,
    container: Container,
    mut children: Expr,
  ) -> Option<()> {
    let mut code_block: Option<CallExpr> = None;
    let mut caption: Option<CallExpr> = None;

    loop {
      let Some((_, child)) = children.pop_item() else {
        break;
      };

      let Expr::Call(child) = child else {
        continue;
      };

      match jsx::<R>(&child).get_tag().tag_type() {
        Some(JSXTagType::Component("CodeBlock")) => code_block = Some(child),
        Some(JSXTagType::Component("caption")) => caption = Some(child),
        _ => continue,
      }
    }

    let Some(mut code_block) = code_block else {
      return None;
    };

    if let Some(caption) = caption {
      jsx_mut::<R>(&mut code_block)
        .get_props_mut()
        .set_item("caption", caption.into());
    }

    move_basic_attributes!(R, container, code_block);

    *call = code_block;

    Some(())
  }

  fn process_call_expr(&mut self, call: &mut CallExpr) -> Option<()> {
    let elem = unpack_jsx!(
      [CodeBlockRoot, R, call],
      jsx_tag!(literal_block?) = [Content, attrs as LiteralBlock],
      jsx_tag!(container?) = [Container, attrs as Container, children],
    )?;

    match elem {
      CodeBlockRoot::Content { attrs } => self.process_code_block(call, attrs),
      CodeBlockRoot::Container { attrs, children } => self.process_container(call, attrs, children),
    }
  }
}

impl<R: JSXRuntime> VisitMut for CodeBlockRenderer<R> {
  noop_visit_mut_type!();

  fn visit_mut_call_expr(&mut self, call: &mut CallExpr) {
    call.visit_mut_children_with(self);
    self.process_call_expr(call);
  }
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
