use std::marker::PhantomData;

use deno_lite::{anyhow, ESFunction, ESModule};
use html5jsx::html_str_to_jsx;
use serde::{Deserialize, Serialize};
use swc_core::{
  common::{util::take::Take as _, Span, Spanned},
  ecma::{
    ast::CallExpr,
    visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith as _},
  },
};
use swc_ecma_utils2::{
  collections::MutableMapping,
  jsx::{create_element, jsx_mut, unpack::unpack_jsx, JSXDocument, JSXElementMut, JSXRuntime},
};

use crate::{components::Transformed, macros::basic_attributes, move_basic_attributes};

#[derive(Deserialize)]
enum SphinxCodeBlock<'ast> {
  #[serde(rename = "container")]
  Container(Container),
  #[serde(rename = "caption")]
  Caption,
  #[serde(rename = "literal_block")]
  #[serde(borrow)]
  LiteralBlock(LiteralBlock<'ast>),
  #[serde(rename = "doctest_block")]
  #[serde(borrow)]
  DoctestBlock(DoctestBlock<'ast>),
}

#[basic_attributes(#[serde(default)])]
#[derive(Deserialize)]
struct Container {
  #[allow(unused)]
  #[serde(deserialize_with = "is_true")]
  literal_block: bool,
}

#[basic_attributes(#[serde(default)])]
#[derive(Deserialize, Debug)]
struct LiteralBlock<'ast> {
  #[serde(alias = "children")]
  code: &'ast str,
  language: Option<String>,
  #[serde(alias = "linenos")]
  show_line_numbers: Option<bool>,
  #[serde(alias = "highlight_args")]
  line_options: Option<LineOptions>,
}

#[basic_attributes(#[serde(default)])]
#[derive(Deserialize)]
struct DoctestBlock<'ast> {
  #[serde(alias = "children")]
  code: &'ast str,
}

impl<'ast> Into<LiteralBlock<'ast>> for DoctestBlock<'ast> {
  fn into(self) -> LiteralBlock<'ast> {
    LiteralBlock {
      code: self.code,
      language: Some("python".into()),
      show_line_numbers: Some(false),
      line_options: None,
      ids: self.ids,
      classes: self.classes,
      names: self.names,
      dupnames: self.dupnames,
    }
  }
}

#[derive(Deserialize, Debug)]
struct LineOptions {
  #[serde(alias = "hl_lines")]
  highlighted: Option<Vec<usize>>,
  #[serde(alias = "linenostart")]
  start_line: Option<usize>,
}

#[derive(Serialize, ESFunction)]
#[serde(rename_all = "camelCase")]
struct RenderCode<'ast> {
  code: &'ast str,
  lang: &'ast str,
  line_highlight: Option<Vec<usize>>,
}

#[basic_attributes(#[serde(default)])]
#[derive(Serialize)]
struct CodeBlockProps<'ast> {
  code: &'ast str,
  lang: &'ast str,
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

enum VisitResult {
  Continue,
  Replace(CallExpr),
}

struct CodeBlockRenderer<R: JSXRuntime> {
  state: State,
  module: ESModule,
  jsx: PhantomData<R>,
}

impl<R: JSXRuntime> VisitMut for CodeBlockRenderer<R> {
  noop_visit_mut_type!();

  fn visit_mut_call_expr(&mut self, call: &mut CallExpr) {
    match match unpack_jsx::<R, SphinxCodeBlock>(call) {
      Ok(SphinxCodeBlock::Container(elem)) => self.process_container(call, elem),
      Ok(SphinxCodeBlock::Caption) => self.process_caption(call),
      Ok(SphinxCodeBlock::LiteralBlock(elem)) => self.process_literal_block(call.span(), elem),
      Ok(SphinxCodeBlock::DoctestBlock(elem)) => {
        self.process_literal_block(call.span(), elem.into())
      }
      Err(_) => {
        call.visit_mut_children_with(self);
        Ok(VisitResult::Continue)
      }
    } {
      Ok(VisitResult::Replace(replace)) => *call = replace,
      Ok(VisitResult::Continue) => {}
      Err(_) => todo!(),
    }
  }
}

impl<R: JSXRuntime> CodeBlockRenderer<R> {
  fn process_container(
    &mut self,
    call: &mut CallExpr,
    elem: Container,
  ) -> anyhow::Result<VisitResult> {
    let State::Empty = self.state else {
      return Ok(VisitResult::Continue);
    };

    self.state = State::Container { container: elem };

    call.visit_mut_children_with(self);

    if let State::CodeBlock(code_block) = std::mem::take(&mut self.state) {
      Ok(VisitResult::Replace(code_block))
    } else {
      Ok(VisitResult::Continue)
    }
  }

  fn process_caption(&mut self, call: &mut CallExpr) -> anyhow::Result<VisitResult> {
    let State::Container { container } = std::mem::take(&mut self.state) else {
      return Ok(VisitResult::Continue);
    };

    call.visit_mut_children_with(self);

    self.state = State::Caption {
      container,
      caption: call.take(),
    };

    Ok(VisitResult::Continue)
  }

  fn process_literal_block(
    &mut self,
    span: Span,
    mut elem: LiteralBlock,
  ) -> anyhow::Result<VisitResult> {
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
    let lang = &*match_language(&lang).unwrap_or(&lang).to_lowercase();

    let start_line = line_options
      .as_ref()
      .and_then(|f| f.start_line)
      .unwrap_or(1);

    let show_line_numbers = show_line_numbers.unwrap_or(false);

    let document = self.render_code_block(code, lang, line_options.and_then(|f| f.highlighted));

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

    let result = match document {
      Ok(document) => {
        let child = document.to_fragment::<R>();
        create_element::<R>(Transformed::CodeBlock)
          .props(&props)
          .child(child.into())
          .span(span)
          .build()?
      }
      Err(error) => {
        let error = format!("{}", error);
        create_element::<R>(Transformed::CodeBlock)
          .props(&props)
          .child(error.into())
          .span(span)
          .build()?
      }
    };

    match std::mem::take(&mut self.state) {
      State::Empty => Ok(VisitResult::Replace(result)),
      State::Container { .. } => {
        self.state = State::CodeBlock(result);
        Ok(VisitResult::Continue)
      }
      State::Caption { caption, .. } => {
        let mut result = result;
        jsx_mut::<R>(&mut result)
          .get_props_mut()
          .set_item("caption", caption.into());
        self.state = State::CodeBlock(result);
        Ok(VisitResult::Continue)
      }
      State::CodeBlock { .. } => unreachable!(),
    }
  }

  fn render_code_block(
    &mut self,
    code: &str,
    lang: &str,
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
