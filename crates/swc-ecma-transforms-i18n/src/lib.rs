use std::{collections::HashMap, fmt::Debug};

use serde::{Deserialize, Serialize};
use swc_core::{
  common::Spanned,
  ecma::{
    ast::{CallExpr, Lit},
    visit::{noop_visit_mut_type, VisitMut, VisitMutWith as _, VisitWith},
  },
};

use swc_utils::{
  jsx::factory::{JSXElement, JSXFactory},
  span::with_span,
};

mod attribute;
mod flow_content;
mod message;
mod phrasing_content;

use crate::attribute::translate_attribute;
use crate::flow_content::FlowContentCollector;
use crate::message::Message;
use crate::phrasing_content::{PhrasingContentCollector, PhrasingContentPreflight};

/// Content model determines how the element is translated.
///
/// https://html.spec.whatwg.org/#concept-element-content-model
///
/// [ContentModel::Flow] — The element is being treated like a container for
/// [flow content]. Elements like `<section>` `<div>` belong to this.
///
/// [flow content]: https://developer.mozilla.org/en-US/docs/Web/HTML/Content_categories#flow_content
///
/// For "flow content", translatable text is collected "shallowly": consecutive strings
/// along the element's immediate children are concatenated into a single translation.
///
/// [ContentModel::Phrasing] — The element is being treated like a container for
/// [phrasing content]. Elements like `<p>` `<strong>` `<em>` belong to this.
///
/// [phrasing content]: https://developer.mozilla.org/en-US/docs/Web/HTML/Content_categories#phrasing_content
///
/// For "phrasing content", translatable text is collected "deeply": each element,
/// if it contains any translatable text, gets one message, no matter how deeply nested
/// it is. This is meant to allow coherent paragraphs of potentially "rich" content
/// (interpolated with markups) to be translated as a whole.
///
/// This design is meant to make the resulting messages as friendly as possible, striking
/// a balance between piecemeal translations (when every element is translated separately)
/// and overly long messages mixed with many markups (when large sections of the
/// document is translated as a single message). It is also meant to handle arbitrary
/// and potentially non-semantic nesting of different elements (especially stray
/// texts directly inside generic containers like `<div>`).
///
/// ## Examples
///
/// Given:
///
/// ```html
/// <div>
///   The quick
///   <em>brown</em>
///   fox jumps
///   <strong>over</strong>
///   the lazy dog.
/// </div>
/// ```
///
/// If `<div>` is "flow content", then the extracted messages will be
///
/// - `The quick`
/// - `brown`
/// - `fox jumps`
/// - `over`
/// - `the lazy dog.`
///
/// Given:
///
/// ```html
/// <p>
///   The quick
///   <em>brown</em>
///   fox jumps
///   <strong>over</strong>
///   the lazy dog.
/// </p>
/// ```
///
/// If `<p>` `<em>` `<strong>` are all "phrasing content", then the extracted messages
/// will be
///
/// - `The quick <1>brown</1> fox jumps <2>over</2> the lazy dog.`
///
/// Given:
///
/// ```html
/// <section>
///   Famous pangrams
///
///   <ul>
///     <li>The quick <em>brown</em> fox jumps <strong>over</strong> the lazy dog.</li>
///   </ul>
/// </section>
/// ```
///
/// ... which is technically valid but not recommended because "Famous pangrams" is
/// [straddled] in between "paragraph" elements like `<h1>` `<h2>` or `<p>`.
///
/// [straddled]: https://html.spec.whatwg.org/#paragraphs
///
/// If `<section>` `<ul>` are "flow content", and `<li>` is "phrasing content", then the
/// extracted messages will be
///
/// - `Famous pangrams`
/// - `The quick <1>brown</1> fox jumps <2>over</2> the lazy dog.`
///
/// ## Notes
///
/// Note that "content model" isn't the same as "content category" in HTML spec.
/// "Content model" describes what kind of content the element can contain, while
/// "content category" describes what kind of content the element is. For example,
/// `<p>` is classified as "flow content", but its content model is "phrasing content".
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentModel {
  Flow,
  Phrasing,
}

impl Default for ContentModel {
  fn default() -> Self {
    Self::Flow
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Translatable {
  /// What element is this?
  pub tag: JSXElement,
  /// Is the element preformatted like `<pre>` (whitespace is significant)?
  /// If not, then whitespace is collapsed according to HTML's whitespace rules.
  /// Default is `false`.
  #[serde(default)]
  pub pre: bool,
  /// How will this element be translated? See [ContentModel].
  #[serde(default)]
  pub content: ContentModel,
  /// List of props to translate.
  pub props: Vec<Vec<String>>,
}

impl Default for Translatable {
  fn default() -> Self {
    Self {
      tag: JSXElement::Fragment,
      content: ContentModel::Flow,
      pre: false,
      props: vec![],
    }
  }
}

impl Translatable {
  pub fn flow<E: Into<JSXElement>>(tag: E) -> Self {
    Self {
      tag: tag.into(),
      content: ContentModel::Flow,
      pre: false,
      props: vec![],
    }
  }

  pub fn phrasing<E: Into<JSXElement>>(tag: E) -> Self {
    Self {
      tag: tag.into(),
      content: ContentModel::Phrasing,
      pre: false,
      props: vec![],
    }
  }

  pub fn preformatted<E: Into<JSXElement>>(tag: E, content: ContentModel) -> Self {
    Self {
      tag: tag.into(),
      content,
      pre: true,
      props: vec![],
    }
  }

  pub fn prop(mut self, prop: &[&str]) -> Self {
    self.props.push(prop.iter().map(|s| (*s).into()).collect());
    self
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslatorOptions {
  #[serde(rename = "Trans")]
  #[serde(default = "TranslatorOptions::default_trans")]
  trans: String,

  #[serde(rename = "i18n._")]
  #[serde(default = "TranslatorOptions::default_gettext")]
  gettext: String,

  #[serde(default)]
  elements: Vec<Translatable>,
}

impl TranslatorOptions {
  fn default_trans() -> String {
    "Trans".into()
  }

  fn default_gettext() -> String {
    "i18n".into()
  }
}

impl Default for TranslatorOptions {
  fn default() -> Self {
    Self {
      trans: TranslatorOptions::default_trans(),
      gettext: TranslatorOptions::default_gettext(),
      elements: vec![],
    }
  }
}

#[derive(Debug)]
pub struct Translator {
  factory: JSXFactory,
  options: TranslatorOptions,

  elements: HashMap<JSXElement, Translatable>,

  messages: Vec<Message>,
  pre: bool,
}

impl VisitMut for Translator {
  noop_visit_mut_type!();

  fn visit_mut_call_expr(&mut self, call: &mut CallExpr) {
    let element = match self.factory.call_is_jsx(call) {
      Some(elem) => elem,
      None => {
        call.visit_mut_children_with(self);
        return;
      }
    };

    if matches!(element, JSXElement::Intrinsic(_)) {
      [
        &[Lit::from("title")],
        &[Lit::from("aria-label")],
        &[Lit::from("aria-placeholder")],
        &[Lit::from("aria-roledescription")],
        &[Lit::from("aria-valuetext")],
      ]
      .iter()
      .for_each(|attr| {
        if let Some(message) =
          translate_attribute(&self.factory, &self.options.gettext.as_str(), call, *attr)
        {
          self.messages.push(message);
        }
      })
    }

    let translatable = self.elements.get(&element);

    match translatable {
      Some(options) => {
        let props = &options.props;
        let gettext = &self.options.gettext.as_str();

        props.iter().for_each(|prop| {
          let path = prop
            .iter()
            .map(|s| Lit::from(s.as_str()))
            .collect::<Vec<_>>();
          if let Some(message) = translate_attribute(&self.factory, gettext, call, &path) {
            self.messages.push(message);
          }
        });

        let pre_parent = self.pre;
        self.pre = self.pre || options.pre;

        match options.content {
          ContentModel::Flow => {
            let mut collector =
              FlowContentCollector::new(self.factory.clone(), &self.options.trans, self.pre);

            call.visit_mut_children_with(&mut collector);

            let (messages, children) = collector.results();

            let children = self.factory.ensure_fragment(&["children"], children);
            self.factory.set_prop(call, &["children"], children);

            self.messages.extend(messages);
          }
          ContentModel::Phrasing => {
            let mut preflight = PhrasingContentPreflight::new();

            call.visit_children_with(&mut preflight);

            if preflight.is_translatable() {
              let mut collector = PhrasingContentCollector::new(
                self.factory.clone(),
                &self.options.trans,
                options.pre,
              );

              call.visit_mut_children_with(&mut collector);

              let (message, children) = collector.result();

              self
                .factory
                .set_prop(call, &["children"], with_span(Some(call.span()))(children));

              self.messages.push(message);
            }
          }
        };

        call.visit_mut_children_with(self);

        self.pre = pre_parent;
      }
      None => call.visit_mut_children_with(self),
    }
  }
}

impl Default for Translator {
  fn default() -> Self {
    Self {
      factory: Default::default(),
      options: Default::default(),
      elements: Default::default(),
      messages: vec![],
      pre: false,
    }
  }
}

impl Translator {
  pub fn new(factory: JSXFactory, mut options: TranslatorOptions) -> Self {
    let mut elements: HashMap<JSXElement, Translatable> = Default::default();

    options.elements.drain(..).for_each(|elem| {
      elements.insert(elem.tag.clone(), elem);
    });

    Self {
      factory,
      options,
      elements,
      messages: vec![],
      pre: false,
    }
  }

  pub fn flow<E: Into<JSXElement>>(mut self, tag: E) -> Self {
    self.elements.insert(
      tag.into(),
      Translatable {
        content: ContentModel::Flow,
        ..Default::default()
      },
    );
    self
  }

  pub fn phrasing<E: Into<JSXElement>>(mut self, tag: E) -> Self {
    self.elements.insert(
      tag.into(),
      Translatable {
        content: ContentModel::Phrasing,
        ..Default::default()
      },
    );
    self
  }

  pub fn preformatted<E: Into<JSXElement>>(mut self, tag: E, content: ContentModel) -> Self {
    self.elements.insert(
      tag.into(),
      Translatable {
        pre: true,
        content,
        ..Default::default()
      },
    );
    self
  }

  pub fn fragment(mut self) -> Self {
    self.elements.insert(
      JSXElement::Fragment,
      Translatable {
        content: ContentModel::Flow,
        ..Default::default()
      },
    );
    self
  }

  pub fn prop<E: Into<JSXElement> + Debug>(mut self, prop: &[&str], tags: Vec<E>) -> Self {
    tags.into_iter().for_each(|name| {
      let tag = name.into();
      let translatable = self.elements.entry(tag).or_default();
      translatable
        .props
        .push(prop.iter().map(|s| (*s).into()).collect())
    });
    self
  }

  pub fn inline(self) -> Self {
    // https://html.spec.whatwg.org/#phrasing-content
    self
      .phrasing("a")
      .phrasing("abbr")
      .phrasing("b")
      .phrasing("bdi")
      .phrasing("bdo")
      .phrasing("button")
      .phrasing("cite")
      .phrasing("code")
      .phrasing("data")
      .phrasing("del")
      .phrasing("dfn")
      .phrasing("em")
      .phrasing("i")
      .phrasing("img")
      .phrasing("ins")
      .phrasing("kbd")
      .phrasing("label")
      .phrasing("mark")
      .phrasing("meter")
      .phrasing("noscript")
      .phrasing("output")
      .phrasing("progress")
      .phrasing("q")
      .phrasing("rp")
      .phrasing("rt")
      .phrasing("ruby")
      .phrasing("s")
      .phrasing("samp")
      .phrasing("small")
      .phrasing("span")
      .phrasing("strong")
      .phrasing("sub")
      .phrasing("sup")
      .phrasing("time")
      .phrasing("u")
      .phrasing("var")
      .prop(&["alt"], vec!["area", "img", "input"])
      .prop(&["href"], vec!["a", "area", "link"])
      .prop(&["label"], vec!["optgroup", "option", "track"])
      .prop(&["placeholder"], vec!["input", "textarea"])
      .prop(&["poster"], vec!["video"])
      .prop(
        &["src"],
        vec![
          "audio", "embed", "iframe", "img", "input", "script", "source", "track", "video",
        ],
      )
  }

  pub fn paragraphs(self) -> Self {
    self
      .phrasing("p")
      .phrasing("h1")
      .phrasing("h2")
      .phrasing("h3")
      .phrasing("h4")
      .phrasing("h5")
      .phrasing("h6")
      .phrasing("dd")
      .phrasing("dt")
      .phrasing("figcaption")
      .phrasing("li")
      .phrasing("th")
      .phrasing("td")
  }

  pub fn sections(self) -> Self {
    self
      .flow("div")
      .flow("address")
      .flow("article")
      .flow("aside")
      .flow("blockquote")
      .flow("details")
      .flow("dialog")
      .flow("dl")
      .flow("fieldset")
      .flow("figure")
      .flow("footer")
      .flow("form")
      .flow("header")
      .flow("hgroup")
      .flow("main")
      .flow("nav")
      .flow("ol")
      .flow("section")
      .flow("table")
      .flow("tr")
      .flow("ul")
      .flow("video")
  }

  pub fn pre(self) -> Self {
    self.preformatted("pre", ContentModel::Phrasing)
  }

  pub fn build(self) -> Self {
    self
  }
}
