use adler::adler32_slice;
use serde::{Deserialize, Serialize};
use std::vec;
use swc_core::{
    atoms::Atom,
    ecma::ast::{Expr, Ident, KeyValueProp, Prop, PropName, PropOrSpread},
};
use swc_html_ast::{Document, DocumentFragment, Element, Namespace};
use swc_html_visit::{Visit, VisitWith as _};

use crate::element::create_element;
use crate::{props::convert_attribute, Fragment};

#[derive(Serialize, Deserialize)]
pub struct JSXFactory {
    pub jsx: Atom,
    pub jsxs: Atom,
    #[serde(rename = "Fragment")]
    pub fragment: Atom,
}

#[derive(Serialize, Deserialize)]
pub struct JSXOptions {
    pub factory: JSXFactory,
}

pub struct DOMVisitor {
    options: JSXOptions,
    parents: Vec<Vec<Box<Expr>>>,
    head: Vec<Box<Expr>>,
    styles: Vec<Atom>,
}

fn style_selector(style: &str) -> String {
    format!(".jsx-styled-{:x}{{", adler32_slice(style.as_bytes()))
}

fn style_classname(style: &str) -> String {
    format!("jsx-styled-{:x}", adler32_slice(style.as_bytes()))
}

impl DOMVisitor {
    pub fn new(options: JSXOptions) -> Self {
        if options.factory.jsx.contains("eval")
            || options.factory.jsxs.contains("eval")
            || options.factory.fragment.contains("eval")
            || options.factory.jsx.contains("Function")
            || options.factory.jsxs.contains("Function")
            || options.factory.fragment.contains("Function")
        {
            panic!("JSX factories cannot contain 'eval' or 'Function' in name");
        }
        Self {
            options,
            parents: vec![],
            head: vec![],
            styles: vec![],
        }
    }

    pub fn get(&mut self) -> Result<Fragment, swc_html_parser::error::Error> {
        let mut head = vec![];

        let mut stylesheet: Vec<Box<Expr>> = vec![];

        self.styles.iter().for_each(|inline| {
            stylesheet.push(Expr::from(style_selector(&inline)).into());
            stylesheet.push(Expr::from(inline.as_str()).into());
            stylesheet.push(Expr::from("}").into());
        });

        if stylesheet.len() > 0 {
            head.push(create_element(
                Some(&"style".into()),
                None,
                Some(stylesheet),
                &self.options.factory,
            ));
        }

        head.append(&mut self.head);

        let children = self.parents.pop().unwrap_or(vec![]);
        let body = create_element(None, None, Some(children), &self.options.factory);

        Ok(Fragment { head, body })
    }
}

impl Visit for DOMVisitor {
    fn visit_element(&mut self, elem: &Element) {
        match elem.tag_name.as_str() {
            "script" | "base" => {
                panic!("refuse to parse {} tags", elem.tag_name);
            }
            _ => (),
        };

        let children = vec![];
        self.parents.push(children);
        elem.visit_children_with(self);
        let children = self.parents.pop().expect("expected children");

        let mut props = vec![];
        let mut classes = String::new();
        let mut styled: Option<String> = None;

        for attr in &elem.attributes {
            if attr.name == "style" {
                match &attr.value {
                    Some(value) => {
                        self.styles.push(value.as_str().into());
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
                props.push(prop)
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
            props.push(PropOrSpread::Prop(
                Prop::KeyValue(KeyValueProp {
                    key: PropName::Ident(Ident {
                        sym: "className".into(),
                        span: Default::default(),
                        optional: false,
                    }),
                    value: Expr::from(classes).into(),
                })
                .into(),
            ))
        }

        let element = create_element(
            Some(&elem.tag_name),
            Some(props),
            Some(children),
            &self.options.factory,
        );

        match elem.tag_name.as_str() {
            "base" | "link" | "meta" | "noscript" | "script" | "style" | "title"
                if elem.namespace == Namespace::HTML =>
            {
                self.head.push(element.into());
            }
            _ => {
                let parent = self.parents.last_mut().expect("expected parent");
                parent.push(element.into());
            }
        };
    }

    fn visit_text(&mut self, text: &swc_html_ast::Text) {
        let parent = self.parents.last_mut().expect("expected parent");
        parent.push(Expr::from(text.data.as_str()).into());
    }

    fn visit_document(&mut self, d: &Document) {
        self.parents.push(vec![]);
        d.visit_children_with(self);
    }

    fn visit_document_fragment(&mut self, d: &DocumentFragment) {
        self.parents.push(vec![]);
        d.visit_children_with(self);
    }
}

impl Default for JSXFactory {
    fn default() -> Self {
        Self {
            jsx: "jsx".into(),
            jsxs: "jsxs".into(),
            fragment: "Fragment".into(),
        }
    }
}

impl Default for JSXOptions {
    fn default() -> Self {
        Self {
            factory: Default::default(),
        }
    }
}
