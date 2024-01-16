use swc_core::{common::SourceFile, ecma::ast::Expr};
use swc_html_ast::{DocumentMode, Element, Namespace};
use swc_html_parser::{error::Error, parse_file_as_document_fragment, parser::ParserConfig};
use swc_html_visit::VisitWith as _;
use swc_utils::jsx::factory::JSXFactory;

mod props;
mod visit;

use crate::visit::DOMVisitor;

pub struct Fragment {
    pub head: Vec<Box<Expr>>,
    pub body: Box<Expr>,
}

pub fn html_to_jsx(html: &SourceFile, jsx: Option<JSXFactory>) -> Result<Fragment, Error> {
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
        &html,
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

    let jsx = jsx.unwrap_or_default();

    let mut visitor = DOMVisitor::new(jsx);

    dom.visit_with(&mut visitor);

    visitor.get()
}
