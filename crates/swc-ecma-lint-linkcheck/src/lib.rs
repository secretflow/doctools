use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use swc_core::{
  common::{util::take::Take, MultiSpan, Span},
  ecma::{
    ast::CallExpr,
    visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith},
  },
};
use url::Url;

use swc_ecma_utils::{
  jsx::factory::{JSXRuntime, JSXTagName},
  jsx_or_return,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Link {
  Internal(String),
  Repo((String, String)),
  URL(String),
}

struct LinkCollector<'links> {
  jsx: JSXRuntime,
  links: &'links mut Vec<(Link, MultiSpan)>,
  targets: HashMap<Link, MultiSpan>,
}

impl LinkCollector<'_> {
  fn add_reference(&mut self, link: Link, span: Span) {
    let mut span = MultiSpan::from_span(span);

    if let Some(targets) = self.targets.get(&link) {
      maybe_mark_target(&mut span, targets);
    }

    self.links.push((link, span));
  }

  fn add_target(&mut self, link: Link, span: Span) {
    let span = {
      self
        .targets
        .entry(link.clone())
        .and_modify(|multi| multi.push_span_label(span, String::from("link also defined here")))
        .or_insert_with(|| MultiSpan::from_span(span));
      self.targets.remove(&link).unwrap()
    };

    self
      .links
      .iter_mut()
      .filter_map(|(peer, peer_span)| {
        if *peer == link {
          Some((peer, peer_span))
        } else {
          None
        }
      })
      .for_each(|(_, peer_span)| {
        maybe_mark_target(peer_span, &span);
      });

    self.targets.insert(link, span);
  }
}

fn maybe_mark_target(reference: &mut MultiSpan, targets: &MultiSpan) {
  // FIXME: dedup here
  targets.span_labels().iter().for_each(|label| {
    if let Some(def) = reference.primary_span() {
      if def.contains(label.span) {
        return;
      }
    }
    let message = if label.is_primary {
      "link defined here"
    } else {
      "link also defined here"
    };
    reference.push_span_label(label.span, String::from(message))
  })
}

impl VisitMut for LinkCollector<'_> {
  noop_visit_mut_type!();

  fn visit_mut_call_expr(&mut self, call: &mut CallExpr) {
    call.visit_mut_children_with(self);

    let (name, props) = jsx_or_return!(self.jsx, call);

    let refuri = self.jsx.get_prop(props, &["refuri"]).as_string();

    let url = match refuri {
      Some(refuri) => refuri,
      None => return,
    };

    let link = match Url::parse(url) {
      Ok(url) => Link::URL(url.to_string()),
      Err(_) => Link::Internal(url.to_string()), // TODO: differentiate errors
    };

    match name {
      JSXTagName::Ident(name) if &*name == "target" => {
        self.add_target(link, call.span);
        call.take();
      }
      JSXTagName::Ident(name) if &*name == "reference" => {
        // TODO: allow third party nodes
        self.add_reference(link, call.span);
      }
      _ => (),
    }
  }
}

pub fn collect_links(
  jsx: JSXRuntime,
  links: &mut Vec<(Link, MultiSpan)>,
) -> impl Fold + VisitMut + '_ {
  as_folder(LinkCollector {
    jsx,
    links,
    targets: HashMap::new(),
  })
}
