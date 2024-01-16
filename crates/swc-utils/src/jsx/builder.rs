use swc_core::{
    common::Span,
    ecma::ast::{Expr, Ident},
};

use crate::span::with_span;

use super::factory::{JSXElement, JSXFactory};

struct PropPath(Vec<String>);

struct Children(Vec<Box<Expr>>);

struct Context {
    parent: Box<Expr>,
    prop: PropPath,
    children: Children,
}

enum LastElement {
    Head,
    Body,
    Context,
}

pub struct JSXSnippet {
    pub name: Ident,
    pub tree: Box<Expr>,
    pub html_id: String,
}

pub struct JSXDocument {
    pub head: Box<Expr>,
    pub body: Box<Expr>,
    pub snippets: Vec<JSXSnippet>,
}

pub struct DocumentBuilder {
    factory: JSXFactory,

    state: Option<LastElement>,
    context: Vec<Context>,

    head: Children,
    body: Children,
    snippets: Vec<JSXSnippet>,
}

impl DocumentBuilder {
    pub fn element(
        &mut self,
        name: &JSXElement,
        props: Option<Box<Expr>>,
        span: Option<Span>,
    ) -> &mut Self {
        let elem = self.factory.create(name).props_with_children(props).build();
        let elem = if let Some(span) = span {
            with_span(span)(elem)
        } else {
            elem
        };
        self.push(elem);
        self
    }

    pub fn enter(&mut self, path: &[&str]) -> &mut Self {
        let parent = self.pop();
        self.context.push(Context {
            parent,
            prop: PropPath(path.iter().map(|s| s.to_string()).collect()),
            children: Children(vec![]),
        });
        self
    }

    pub fn value(&mut self, value: Box<Expr>) -> &mut Self {
        self.push(value);
        self
    }

    pub fn exit(&mut self) -> &mut Self {
        let Context {
            mut parent,
            prop,
            children,
        } = match self.context.pop() {
            Some(v) => v,
            None => return self,
        };
        self.factory
            .set_children(&mut parent, &prop.as_strs()[..], children.0);
        self.push(parent);
        self
    }

    pub fn id(&mut self, id: String) -> &mut Self {
        let tree = self.pop();
        let name = self.snippet_name();
        self.element(
            &Ident::from(name.as_str()).into(),
            Some(Ident::from("props").into()),
            None,
        );
        self.snippets.push(JSXSnippet {
            html_id: id,
            name: Ident::from(name.as_str()),
            tree,
        });
        self
    }

    pub fn flush(&mut self) {
        while self.context.len() > 0 {
            self.exit();
        }
    }

    fn pop(&mut self) -> Box<Expr> {
        match self.state {
            Some(LastElement::Head) => self.head.0.pop(),
            Some(LastElement::Body) => self.body.0.pop(),
            Some(LastElement::Context) => {
                self.context.last_mut().and_then(|ctx| ctx.children.0.pop())
            }
            None => None,
        }
        .expect("no element to enter")
    }

    fn push(&mut self, value: Box<Expr>) {
        let kind = self.factory.expr_is_jsx(&value);

        match kind {
            Some(ref elem) if elem.is_metadata() => {
                self.head.0.push(value);
                self.state = Some(LastElement::Head);
            }
            _ => match self.context.last_mut() {
                Some(Context { children, .. }) => {
                    children.0.push(value);
                    self.state = if kind.is_some() {
                        Some(LastElement::Context)
                    } else {
                        None
                    };
                }
                None => {
                    self.body.0.push(value);
                    self.state = if kind.is_some() {
                        Some(LastElement::Body)
                    } else {
                        None
                    };
                }
            },
        }
    }

    fn snippet_name(&self) -> String {
        format!("$snippet{}", self.snippets.len())
    }

    pub fn new(jsx: JSXFactory) -> Self {
        Self {
            factory: jsx,
            state: Some(LastElement::Body),
            context: vec![],
            head: Children(vec![]),
            body: Children(vec![]),
            snippets: vec![],
        }
    }

    pub fn declare(self) -> JSXDocument {
        let wrap_tree = |elements: Vec<Box<Expr>>| {
            if elements.len() == 1 {
                elements.into_iter().next().unwrap()
            } else {
                self.factory
                    .create(&JSXElement::Fragment)
                    .children(Some(elements))
                    .build()
            }
        };

        let head = wrap_tree(self.head.0);
        let body = wrap_tree(self.body.0);

        JSXDocument {
            head,
            body,
            snippets: self.snippets,
        }
    }
}

impl PropPath {
    fn as_strs(&self) -> Vec<&str> {
        self.0.iter().map(String::as_str).collect()
    }
}
