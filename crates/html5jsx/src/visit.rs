use adler::adler32_slice;
use std::{collections::HashSet, vec};
use swc_core::{
    atoms::Atom,
    ecma::ast::{Expr, KeyValueProp, Prop, PropName},
};
use swc_html_ast::{Document, DocumentFragment, Element, Namespace, Text};
use swc_html_visit::{Visit, VisitWith as _};

use swc_utils::{
    jsx::factory::{JSXElement, JSXFactory},
    span::with_span,
};

use crate::{props::convert_attribute, Fragment};

pub struct DOMVisitor {
    factory: JSXFactory,
    head: Vec<Box<Expr>>,
    ancestors: Vec<Vec<Box<Expr>>>,
    styles: HashSet<Atom>,
}

fn style_selector(style: &str) -> String {
    format!(".jsx-styled-{:x}{{", adler32_slice(style.as_bytes()))
}

fn style_classname(style: &str) -> String {
    format!("jsx-styled-{:x}", adler32_slice(style.as_bytes()))
}

impl Visit for DOMVisitor {
    fn visit_element(&mut self, elem: &Element) {
        match elem.tag_name.as_str() {
            "script" | "base" => {
                if cfg!(feature = "unsafe-ignore") {
                    return;
                } else if cfg!(feature = "unsafe-ignore") {
                    ();
                } else {
                    panic!("refuse to parse {} tags", elem.tag_name);
                }
            }
            _ => (),
        };

        let children = vec![];
        self.ancestors.push(children);

        elem.visit_children_with(self);

        let children = self.ancestors.pop().expect("expected children");

        let mut props: Vec<Box<Prop>> = vec![];
        let mut classes = String::new();
        let mut styled: Option<String> = None;

        for attr in &elem.attributes {
            if attr.name == "style" {
                match &attr.value {
                    Some(value) => {
                        self.styles.insert(value.as_str().into());
                        styled = Some(style_classname(value.as_str()));
                    }
                    None => (),
                }
                continue;
            }
            if attr.name == "class" {
                match &attr.value {
                    Some(value) => {
                        classes.push_str(value.as_str());
                    }
                    None => (),
                }
                continue;
            }
            if let Some(prop) = convert_attribute(&attr) {
                props.push(with_span(elem.span)(prop.into()))
            }
        }

        match styled {
            None => (),
            Some(classname) => {
                if !classes.is_empty() {
                    classes.push(' ');
                }
                classes.push_str(&classname);
            }
        };

        if !classes.is_empty() {
            props.push(with_span(elem.span)(
                Prop::KeyValue(KeyValueProp {
                    key: PropName::Str("className".into()),
                    value: classes.into(),
                })
                .into(),
            ))
        }

        let element = with_span(elem.span)(
            self.factory
                .create(&elem.tag_name.as_str().into())
                .props(Some(props))
                .children(Some(children))
                .build(),
        );

        match elem.tag_name.as_str() {
            "base" | "link" | "meta" | "noscript" | "script" | "style" | "title"
                if elem.namespace == Namespace::HTML =>
            {
                self.head.push(element.into());
            }
            _ => {
                let parent = self.ancestors.last_mut().expect("expected parent");
                parent.push(element.into());
            }
        };
    }

    fn visit_text(&mut self, text: &Text) {
        let parent = self.ancestors.last_mut().expect("expected parent");
        parent.push(with_span(text.span)(Expr::from(text.data.as_str())).into());
    }

    fn visit_document(&mut self, d: &Document) {
        self.ancestors.push(vec![]);
        d.visit_children_with(self);
    }

    fn visit_document_fragment(&mut self, d: &DocumentFragment) {
        self.ancestors.push(vec![]);
        d.visit_children_with(self);
    }
}

impl DOMVisitor {
    pub fn new(factory: JSXFactory) -> Self {
        if factory
            .names()
            .iter()
            .any(|name| name.contains("eval") || name.contains("Function"))
        {
            panic!("JSX factories cannot contain 'eval' or 'Function' in name");
        }
        Self {
            factory,
            ancestors: vec![],
            head: vec![],
            styles: HashSet::new(),
        }
    }

    pub fn get(mut self) -> Result<Fragment, swc_html_parser::error::Error> {
        let mut head = vec![];

        let mut stylesheet: Vec<Box<Expr>> = vec![];

        Some(self.styles.iter().collect::<Vec<_>>())
            .and_then(|mut c| {
                c.sort_unstable();
                Some(c)
            })
            .unwrap()
            .iter()
            .for_each(|style| {
                stylesheet.push(style_selector(&style).into());
                stylesheet.push(style.as_str().into());
                stylesheet.push("}".into());
            });

        if stylesheet.len() > 0 {
            head.push(
                self.factory
                    .create(&"style".into())
                    .children(Some(stylesheet))
                    .build(),
            );
        }

        head.append(&mut self.head);

        let children = self.ancestors.pop().unwrap_or(vec![]);
        let body = self
            .factory
            .create(&JSXElement::Fragment)
            .children(Some(children))
            .build();

        Ok(Fragment { head, body })
    }
}
