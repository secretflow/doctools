use swc_core::common::{FileName, SourceFile, SourceMap};
use swc_ecma_utils2::jsx::{JSXDocument, JSXRuntime};
use swc_html_ast::{DocumentMode, Element, Namespace};
use swc_html_parser::{error::Error, parse_file_as_document_fragment, parser::ParserConfig};
use swc_html_visit::VisitWith as _;

mod props;
mod visit;

use crate::visit::DOMVisitor;

pub fn html_to_jsx<R: JSXRuntime>(html: &SourceFile) -> Result<JSXDocument, Error> {
  let parent = Element {
    namespace: Namespace::HTML,
    span: Default::default(),
    tag_name: "div".into(),
    attributes: vec![],
    children: vec![],
    content: None,
    is_self_closing: false,
  };

  let mut errors: Vec<Error> = vec![];

  let dom = parse_file_as_document_fragment(
    html,
    &parent,
    DocumentMode::NoQuirks,
    None,
    ParserConfig {
      allow_self_closing: true,
      scripting_enabled: false,
      iframe_srcdoc: false,
    },
    &mut errors,
  )?;

  let mut visitor = <DOMVisitor<R>>::new();

  dom.visit_with(&mut visitor);

  visitor.get()
}

pub fn html_str_to_jsx<R: JSXRuntime>(html: &str) -> Result<JSXDocument, Error> {
  let sources = SourceMap::default();
  let file = sources.new_source_file(FileName::Anon, html.into());
  html_to_jsx::<R>(&file)
}
