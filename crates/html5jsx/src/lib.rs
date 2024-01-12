use swc_core::{
    common::{FileName, SourceMap},
    ecma::ast::Expr,
};
use swc_html_ast::{DocumentMode, Element, Namespace};
use swc_html_parser::{parse_file_as_document_fragment, parser::ParserConfig};
use swc_html_visit::VisitWith as _;

mod element;
mod props;
mod visit;

use crate::visit::DOMVisitor;
pub use crate::visit::{JSXFactory, JSXOptions};

pub struct Fragment {
    pub head: Vec<Box<Expr>>,
    pub body: Box<Expr>,
}

pub fn html_to_jsx(
    html: &str,
    options: Option<JSXOptions>,
) -> Result<Fragment, swc_html_parser::error::Error> {
    let files: SourceMap = Default::default();
    let file = files.new_source_file(FileName::Anon, html.to_string());

    let parent = Element {
        namespace: Namespace::HTML,
        span: Default::default(),
        tag_name: "div".into(),
        attributes: vec![],
        children: vec![],
        content: None,
        is_self_closing: false,
    };

    let mut errors: Vec<swc_html_parser::error::Error> = vec![];

    let dom = parse_file_as_document_fragment(
        &file,
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

    let options = options.unwrap_or_default();

    let mut visitor = DOMVisitor::new(options);

    dom.visit_with(&mut visitor);

    visitor.get()
}
