use std::{borrow::Cow, marker::PhantomData};

use html5jsx::html_str_to_jsx;
use serde::{Deserialize, Serialize};
use sphinx_jsx_macros::basic_attributes;
use swc_core::{
  common::{util::take::Take, Span, Spanned},
  ecma::{
    ast::{CallExpr, Expr, ExprOrSpread, Ident, KeyValueProp, PropName, Str},
    visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith},
  },
};
use swc_ecma_utils2::{
  anyhow,
  jsx::{create_element, create_fragment, jsx_builder2, unpack::unpack_jsx, JSXRuntime},
  span::with_span,
  tag,
};

use crate::move_basic_attributes;

#[derive(Deserialize)]
enum SphinxRaw {
  #[serde(rename = "paragraph")]
  Paragraph,
  #[serde(rename = "raw")]
  Raw(Raw),
}

#[basic_attributes]
#[derive(Deserialize, Debug, Default)]
struct Raw {
  format: String,
  #[serde(alias = "children")]
  value: String,
}

#[basic_attributes]
#[derive(Serialize)]
struct RawProps {
  formats: Vec<(String, String)>,
}

impl Raw {
  fn into_tuple(self) -> (String, String) {
    (
      format_to_mime_type(&self.format)
        .unwrap_or(&self.format)
        .into(),
      self.value,
    )
  }
}

enum State {
  Empty,
  NonHTML(RawProps, Span),
}

impl Default for State {
  fn default() -> Self {
    State::Empty
  }
}

struct RawRenderer<R: JSXRuntime> {
  state: State,
  jsx: PhantomData<R>,
}

impl<R: JSXRuntime> VisitMut for RawRenderer<R> {
  noop_visit_mut_type!();

  fn visit_mut_call_expr(&mut self, call: &mut CallExpr) {
    match match unpack_jsx::<R, SphinxRaw>(call) {
      Err(_) => self.process_other_call(call),
      Ok(SphinxRaw::Paragraph) => self.process_paragraph(call),
      Ok(SphinxRaw::Raw(raw)) => self.process_raw(call, raw),
    } {
      Err(err) => todo!("error handling: {:?}", err),
      Ok(None) => {}
      Ok(Some(state)) => self.state = state,
    }
  }
}

impl<R: JSXRuntime> RawRenderer<R> {
  fn process_raw(&mut self, call: &mut CallExpr, raw: Raw) -> anyhow::Result<Option<State>> {
    match self.state {
      State::Empty => match &*raw.format {
        "html" => {
          let document = html_str_to_jsx::<R>(&raw.value)
            .map_err(|err| anyhow::anyhow!("failed to parse math as JSX: {:?}", err))?;
          *call = with_span(Some(call.span()))(document.to_fragment::<R>());
          Ok(None)
        }

        _ => {
          let mut raw = raw;
          let mut props = RawProps {
            ids: vec![],
            classes: vec![],
            names: vec![],
            dupnames: vec![],
            formats: vec![],
          };
          move_basic_attributes!(raw, props);
          props.formats.push(raw.into_tuple());
          Ok(Some(State::NonHTML(props, call.span())))
        }
      },

      State::NonHTML(ref mut sibling, ref mut span) => {
        let mut raw = raw;
        move_basic_attributes!(raw, sibling);
        sibling.formats.push(raw.into_tuple());
        *span = span.to(call.span());
        Ok(None)
      }
    }
  }

  fn process_other_call(&mut self, call: &mut CallExpr) -> anyhow::Result<Option<State>> {
    match self.state {
      State::NonHTML(ref mut props, ref mut span) => {
        *call = create_fragment::<R>()
          .child(
            create_element::<R>(tag!(Raw))
              .span(span.to(call.span()))
              .props(props)
              .build()?
              .into(),
          )
          .child(call.take().into())
          .build()?;

        Ok(Some(State::Empty))
      }

      State::Empty => {
        call.visit_mut_children_with(self);
        Ok(None)
      }
    }
  }

  fn process_paragraph(&mut self, call: &mut CallExpr) -> anyhow::Result<Option<State>> {
    #[derive(Debug)]
    enum InlineHTML {
      HTML(Raw),
      Expr(Expr),
      Taken,
    }

    impl Default for InlineHTML {
      fn default() -> Self {
        InlineHTML::Taken
      }
    }

    #[derive(Debug)]
    enum State {
      Empty,
      KeyValue,
      Children,
      Found(Vec<InlineHTML>),
      Unsupported,
    }

    struct FindInlineHTML<R: JSXRuntime> {
      state: State,
      jsx: PhantomData<R>,
    }

    impl<R: JSXRuntime> VisitMut for FindInlineHTML<R> {
      noop_visit_mut_type!();

      fn visit_mut_key_value_prop(&mut self, prop: &mut KeyValueProp) {
        match self.state {
          State::Empty => {
            self.state = State::KeyValue;
            prop.visit_mut_children_with(self);
          }
          _ => {
            prop.visit_mut_children_with(self);
          }
        }
      }

      fn visit_mut_prop_name(&mut self, name: &mut PropName) {
        match self.state {
          State::KeyValue => match name {
            PropName::Str(Str { value, .. }) | PropName::Ident(Ident { sym: value, .. })
              if value.as_str() == "children" =>
            {
              self.state = State::Children;
            }
            _ => {}
          },
          _ => {}
        }
      }

      fn visit_mut_expr(&mut self, expr: &mut Expr) {
        match self.state {
          State::Found(_) => unreachable!(),

          State::Children => match expr {
            Expr::Array(children) => {
              if children.elems.iter().any(|e| {
                matches!(
                  e,
                  Some(ExprOrSpread {
                    spread: Some(_),
                    ..
                  })
                )
              }) {
                self.state = State::Unsupported;
              } else {
                children.elems.iter_mut().for_each(|e| match e {
                  None => {}
                  Some(ExprOrSpread { expr, .. }) => self.send_elem(expr),
                })
              }
            }
            expr => self.send_elem(expr),
          },

          _ => expr.visit_mut_children_with(self),
        }
      }
    }

    impl<R: JSXRuntime> FindInlineHTML<R> {
      fn send_elem(&mut self, elem: &mut Expr) {
        match elem {
          Expr::Call(call) => match unpack_jsx::<R, SphinxRaw>(call) {
            Ok(SphinxRaw::Raw(raw)) if raw.format == "html" => {
              call.take();
              match self.state {
                State::Children => {
                  self.state = State::Found(vec![InlineHTML::HTML(raw)]);
                }
                State::Found(ref mut found) => {
                  found.push(InlineHTML::HTML(raw));
                }
                _ => {}
              }
            }
            _ => match self.state {
              State::Found(ref mut found) => found.push(InlineHTML::Expr({
                let mut expr = call.take();
                expr.visit_mut_with(&mut render_raw::<R>());
                expr.into()
              })),
              _ => {}
            },
          },
          _ => match self.state {
            State::Found(ref mut found) => found.push(InlineHTML::Expr({
              let mut expr = elem.take();
              expr.visit_mut_with(&mut render_raw::<R>());
              expr.into()
            })),
            _ => {}
          },
        }
      }
    }

    struct RestoreExpr<R: JSXRuntime> {
      children: Vec<InlineHTML>,
      jsx: PhantomData<R>,
    }

    impl<R: JSXRuntime> RestoreExpr<R> {
      fn process_call(&mut self, call: &mut CallExpr) -> anyhow::Result<Option<Expr>> {
        #[derive(Deserialize)]
        enum Placeholder {
          #[serde(rename = "swc-passthru")]
          Passthru { id: String },
        }

        let Ok(Placeholder::Passthru { id }) = unpack_jsx::<R, Placeholder>(call) else {
          return Ok(None);
        };

        let id = id.parse::<usize>()?;

        let repl = self
          .children
          .get_mut(id)
          .and_then(|c| match std::mem::take(c) {
            InlineHTML::Expr(expr) => Some(expr),
            other => {
              *c = other;
              None
            }
          })
          .ok_or_else(|| anyhow::anyhow!("failed to match HTML result with source"))?;

        Ok(Some(repl))
      }
    }

    impl<R: JSXRuntime> VisitMut for RestoreExpr<R> {
      noop_visit_mut_type!();

      fn visit_mut_expr(&mut self, expr: &mut Expr) {
        expr.visit_mut_children_with(self);

        let Some(call) = expr.as_mut_call() else {
          return;
        };

        match self.process_call(call) {
          Err(_) => todo!(),
          Ok(None) => {}
          Ok(Some(result)) => *expr = result,
        }
      }
    }

    let mut visitor = FindInlineHTML {
      state: State::Empty,
      jsx: PhantomData::<R>,
    };

    call.visit_mut_children_with(&mut visitor);

    let mut children = match visitor.state {
      State::Found(children) => children,
      State::Unsupported => return Err(anyhow::anyhow!("spread elements are not supported")),
      _ => return Ok(None),
    };

    let mut attributes = Raw::default();

    let html = children
      .iter_mut()
      .enumerate()
      .map(|(idx, child)| match child {
        InlineHTML::HTML(raw) => {
          move_basic_attributes!(raw, attributes);
          Cow::from(&*raw.value)
        }
        InlineHTML::Expr { .. } => {
          Cow::from(format!("<swc-passthru id=\"{}\"></swc-passthru>", idx))
        }
        InlineHTML::Taken => unreachable!(),
      })
      .collect::<Vec<_>>()
      .join("");

    let mut document = html_str_to_jsx::<R>(&html)
      .and_then(|document| Ok(document.to_fragment::<R>()))
      .or_else(|err| Err(anyhow::anyhow!("failed to parse math as JSX: {:?}", err)))?;

    let mut visitor = RestoreExpr {
      children,
      jsx: PhantomData::<R>,
    };

    document.visit_mut_children_with(&mut visitor);

    visitor
      .children
      .iter()
      .map(|c| match c {
        InlineHTML::Expr { .. } => Err(anyhow::anyhow!("failed to match HTML result with source")),
        _ => Ok(()),
      })
      .collect::<Result<(), _>>()?;

    *call = jsx_builder2::<R>(call.take())
      .child(document.into())
      .span(call.span())
      .build()?;

    Ok(None)
  }
}

fn format_to_mime_type(format: &str) -> Option<&'static str> {
  match format {
    "html" => Some("text/html"),
    "latex" => Some("text/latex"),
    "rst" => Some("text/x-rst"),
    _ => None,
  }
}

pub fn render_raw<R: JSXRuntime>() -> impl Fold + VisitMut {
  as_folder(RawRenderer {
    state: Default::default(),
    jsx: PhantomData::<R>,
  })
}
