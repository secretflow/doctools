use std::marker::PhantomData;

use deno_lite::{anyhow, ESFunction, ESModule};
use html5jsx::html_str_to_jsx;
use serde::{Deserialize, Serialize};
use sphinx_jsx_macros::basic_attributes;
use swc_core::{
  common::{util::take::Take as _, Spanned},
  ecma::{
    ast::CallExpr,
    visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith as _},
  },
};
use swc_ecma_utils2::{
  collections::MutableMapping,
  jsx::{create_element, jsx_mut, unpack::unpack_jsx, JSXDocument, JSXElementMut, JSXRuntime},
  tag,
};

use crate::move_basic_attributes;

#[derive(Deserialize)]
enum SphinxCodeBlock {
  #[serde(rename = "container")]
  Container(Container),
  #[serde(rename = "caption")]
  Caption,
  #[serde(rename = "literal_block")]
  LiteralBlock(LiteralBlock),
}

#[basic_attributes]
#[derive(Deserialize)]
struct Container {
  #[allow(unused)]
  #[serde(deserialize_with = "is_true")]
  literal_block: bool,
}

#[basic_attributes]
#[derive(Deserialize)]
struct LiteralBlock {
  #[serde(alias = "children")]
  code: String,
  language: Option<String>,
  #[serde(alias = "linenos")]
  show_line_numbers: Option<bool>,
  #[serde(alias = "highlight_args")]
  line_options: Option<LineOptions>,
}

#[derive(Deserialize)]
struct LineOptions {
  #[serde(alias = "hl_lines")]
  highlighted: Option<Vec<usize>>,
  #[serde(alias = "linenostart")]
  start_line: Option<usize>,
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

enum State {
  Empty,
  Container {
    container: Container,
  },
  Caption {
    container: Container,
    caption: CallExpr,
  },
  CodeBlock(CallExpr),
}

impl Default for State {
  fn default() -> Self {
    State::Empty
  }
}

struct CodeBlockRenderer<R: JSXRuntime> {
  state: State,
  module: ESModule,
  jsx: PhantomData<R>,
}

impl<R: JSXRuntime> VisitMut for CodeBlockRenderer<R> {
  noop_visit_mut_type!();

  fn visit_mut_call_expr(&mut self, call: &mut CallExpr) {
    let result = match unpack_jsx::<R, SphinxCodeBlock>(call) {
      Ok(SphinxCodeBlock::Container(elem)) => self.process_container(call, elem),
      Ok(SphinxCodeBlock::Caption) => self.process_caption(call),
      Ok(SphinxCodeBlock::LiteralBlock(elem)) => self.process_literal_block(call, elem),
      Err(_) => {
        call.visit_mut_children_with(self);
        Ok(())
      }
    };

    match result {
      Ok(()) => {}
      Err(_) => todo!(),
    }
  }
}

impl<R: JSXRuntime> CodeBlockRenderer<R> {
  fn process_container(&mut self, call: &mut CallExpr, elem: Container) -> anyhow::Result<()> {
    let State::Empty = self.state else {
      return Ok(());
    };

    self.state = State::Container { container: elem };

    call.visit_mut_children_with(self);

    if let State::CodeBlock(code_block) = std::mem::take(&mut self.state) {
      *call = code_block;
    }

    Ok(())
  }

  fn process_caption(&mut self, call: &mut CallExpr) -> anyhow::Result<()> {
    let State::Container { container } = std::mem::take(&mut self.state) else {
      return Ok(());
    };

    call.visit_mut_children_with(self);

    self.state = State::Caption {
      container,
      caption: call.take(),
    };

    Ok(())
  }

  fn process_literal_block(
    &mut self,
    call: &mut CallExpr,
    mut elem: LiteralBlock,
  ) -> anyhow::Result<()> {
    if let State::Container { ref mut container }
    | State::Caption {
      ref mut container, ..
    } = self.state
    {
      move_basic_attributes!(container, &mut elem);
    }

    let LiteralBlock {
      code,
      language,
      show_line_numbers,
      line_options,
      ids,
      classes,
      names,
      dupnames,
    } = elem;

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

    let mut result = match document {
      Ok(document) => {
        let child = document.to_fragment::<R>();
        create_element::<R>(tag!(CodeBlock))
          .props(&props)
          .child(child.into())
          .span(call.span())
          .build()?
      }
      Err(error) => {
        let error = format!("{}", error);
        create_element::<R>(tag!(CodeBlock))
          .props(&props)
          .child(error.into())
          .span(call.span())
          .build()?
      }
    };

    match std::mem::take(&mut self.state) {
      State::Empty => {
        *call = result;
      }
      State::Container { .. } => {
        self.state = State::CodeBlock(result);
      }
      State::Caption { caption, .. } => {
        jsx_mut::<R>(&mut result)
          .get_props_mut()
          .set_item("caption", caption.into());
        self.state = State::CodeBlock(result);
      }
      State::CodeBlock { .. } => unreachable!(),
    };

    Ok(())
  }

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
    let document = html_str_to_jsx::<R>(&*html)
      .map_err(|err| anyhow::anyhow!("failed to parse math as JSX: {:?}", err))?;
    Ok(document)
  }
}

pub fn render_code<R: JSXRuntime>(esm: &ESModule) -> impl Fold + VisitMut {
  as_folder(CodeBlockRenderer {
    state: Default::default(),
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

fn is_true<'de, D>(de: D) -> Result<bool, D::Error>
where
  D: serde::de::Deserializer<'de>,
{
  let value: bool = serde::Deserialize::deserialize(de)?;
  if value {
    Ok(value)
  } else {
    Err(serde::de::Error::custom("expected true"))
  }
}
