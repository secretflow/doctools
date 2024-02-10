use std::{collections::HashSet, marker::PhantomData};

use sha2::{Digest, Sha256};
use swc_core::{
  atoms::Atom,
  ecma::ast::{ArrayLit, Expr, Lit},
};
use swc_html_ast::{Document, DocumentFragment, Element, Namespace, Text};
use swc_html_visit::{Visit, VisitWith as _};

use swc_ecma_utils2::{
  collections::{MutableMapping, MutableSequence},
  jsx::{create_element, jsx_mut, JSXDocument, JSXElementMut, JSXRuntime},
  span::with_span,
};

use crate::props::convert_attribute;

pub struct DOMVisitor<R: JSXRuntime> {
  head: Vec<Expr>,
  ancestors: Vec<Vec<Expr>>,
  styles: HashSet<Atom>,
  jsx: PhantomData<R>,
}

fn style_hash(style: &str) -> String {
  let mut hasher = Sha256::new();
  hasher.update(style.as_bytes());
  let result = hasher.finalize();
  format!("{:x}", result)[..7].to_string()
}

fn style_selector(style: &str) -> String {
  format!(".jsx-styled-{}{{", style_hash(&style))
}

fn style_classname(style: &str) -> String {
  format!("jsx-styled-{}", style_hash(&style))
}

impl<R: JSXRuntime> Visit for DOMVisitor<R> {
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

    let name = &*elem.tag_name;

    let mut new = create_element::<R>(Lit::from(name).into());

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
      if let Some((key, value)) = convert_attribute(&attr) {
        jsx_mut::<R>(&mut new)
          .get_props_mut()
          .set_item(key, with_span(Some(attr.span))(value.into()));
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
      jsx_mut::<R>(&mut new)
        .get_props_mut()
        .set_item("className", classes.into());
    }

    jsx_mut::<R>(&mut new)
      .get_props_mut()
      .set_item("children", ArrayLit::from_iterable(children).into());

    let element = with_span(Some(elem.span))(new);

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
    parent.push(with_span(Some(text.span))(Expr::from(text.data.as_str())).into());
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

impl<R: JSXRuntime> DOMVisitor<R> {
  pub fn new() -> Self {
    Self {
      jsx: PhantomData,
      ancestors: vec![],
      head: vec![],
      styles: HashSet::new(),
    }
  }

  pub fn get(mut self) -> Result<JSXDocument, swc_html_parser::error::Error> {
    let mut head = vec![];

    let mut stylesheet: Vec<Expr> = vec![];

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
      let mut style = create_element::<R>("style".into());

      jsx_mut::<R>(&mut style)
        .get_props_mut()
        .set_item("children", ArrayLit::from_iterable(stylesheet).into());

      head.push(style.into());
    }

    head.append(&mut self.head);

    let children = self.ancestors.pop().unwrap_or(vec![]);

    Ok(JSXDocument {
      head,
      body: children,
    })
  }
}
