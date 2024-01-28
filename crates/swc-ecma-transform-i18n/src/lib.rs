use std::{collections::HashMap, fmt::Debug};

use serde::{Deserialize, Serialize};
use swc_core::{
  atoms::Atom,
  ecma::{
    ast::CallExpr,
    visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith as _},
  },
};

use swc_ecma_utils::{
  jsx::factory::{JSXRuntime, JSXTagName},
  jsx_or_continue_visit, tag,
};

mod attribute;
mod flow_content;
mod message;
mod phrasing_content;

use crate::attribute::translate_attrs;
use crate::flow_content::translate_block;
use crate::message::Message;
use crate::phrasing_content::translate_phrase;

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
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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
  pub tag: JSXTagName,
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
      tag: tag!(<>),
      content: ContentModel::Flow,
      pre: false,
      props: vec![],
    }
  }
}

impl Translatable {
  pub fn flow<E: Into<JSXTagName>>(tag: E) -> Self {
    Self {
      tag: tag.into(),
      content: ContentModel::Flow,
      pre: false,
      props: vec![],
    }
  }

  pub fn phrasing<E: Into<JSXTagName>>(tag: E) -> Self {
    Self {
      tag: tag.into(),
      content: ContentModel::Phrasing,
      pre: false,
      props: vec![],
    }
  }

  pub fn preformatted<E: Into<JSXTagName>>(tag: E, content: ContentModel) -> Self {
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
  sym_trans: Atom,

  #[serde(rename = "_")]
  #[serde(default = "TranslatorOptions::default_gettext")]
  sym_gettext: Atom,

  #[serde(default)]
  elements: Vec<Translatable>,
}

impl TranslatorOptions {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn trans(&mut self, sym: Atom) -> &mut Self {
    self.sym_trans = sym;
    self
  }

  pub fn gettext(&mut self, sym: Atom) -> &mut Self {
    self.sym_gettext = sym;
    self
  }

  fn default_trans() -> Atom {
    "Trans".into()
  }

  fn default_gettext() -> Atom {
    "i18n".into()
  }
}

impl Default for TranslatorOptions {
  fn default() -> Self {
    Self {
      sym_trans: TranslatorOptions::default_trans(),
      sym_gettext: TranslatorOptions::default_gettext(),
      elements: vec![],
    }
  }
}

#[derive(Debug)]
struct Translator<'result> {
  jsx: JSXRuntime,
  options: TranslatorOptions,
  elements: HashMap<JSXTagName, Translatable>,
  pre: bool,
  messages: &'result mut Vec<Message>,
}

impl VisitMut for Translator<'_> {
  noop_visit_mut_type!();

  fn visit_mut_call_expr(&mut self, elem: &mut CallExpr) {
    let (element, _) = jsx_or_continue_visit!(self, self.jsx, mut elem);

    if matches!(element, tag!("*")) {
      let props = self.jsx.as_mut_jsx_props(elem).unwrap();
      self.messages.extend(translate_attrs(
        self.jsx.clone(),
        self.options.sym_gettext.clone(),
        props,
        vec![
          vec!["title"],
          vec!["aria-label"],
          vec!["aria-placeholder"],
          vec!["aria-roledescription"],
          vec!["aria-valuetext"],
        ],
      ));
    }

    let options = match self.elements.get(&element) {
      Some(options) => options,
      None => {
        elem.visit_mut_children_with(self);
        return;
      }
    };

    {
      let attrs = options
        .props
        .iter()
        .map(|ss| ss.iter().map(|s| s.as_str()).collect::<Vec<_>>())
        .collect::<Vec<_>>();
      let props = self.jsx.as_mut_jsx_props(elem).unwrap();
      self.messages.extend(translate_attrs(
        self.jsx.clone(),
        self.options.sym_gettext.clone(),
        props,
        attrs,
      ))
    };

    let pre_parent = self.pre;
    self.pre = self.pre || options.pre;

    match options.content {
      ContentModel::Flow => {
        self.messages.extend(translate_block(
          self.jsx.clone(),
          self.options.sym_trans.clone(),
          options.pre,
          elem,
        ));
      }
      ContentModel::Phrasing => {
        if let Some(message) = translate_phrase(
          self.jsx.clone(),
          self.options.sym_trans.clone(),
          options.pre,
          elem,
        ) {
          self.messages.push(message);
        }
      }
    };

    elem.visit_mut_children_with(self);

    self.pre = pre_parent;
  }
}

impl<'messages> Translator<'messages> {
  pub fn flow<E: Into<JSXTagName>>(mut self, tag: E) -> Self {
    self.elements.insert(
      tag.into(),
      Translatable {
        content: ContentModel::Flow,
        ..Default::default()
      },
    );
    self
  }

  pub fn phrasing<E: Into<JSXTagName>>(mut self, tag: E) -> Self {
    self.elements.insert(
      tag.into(),
      Translatable {
        content: ContentModel::Phrasing,
        ..Default::default()
      },
    );
    self
  }

  pub fn preformatted<E: Into<JSXTagName>>(mut self, tag: E, content: ContentModel) -> Self {
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
      tag!(<>),
      Translatable {
        content: ContentModel::Flow,
        ..Default::default()
      },
    );
    self
  }

  pub fn prop<E: Into<JSXTagName> + Debug>(mut self, prop: &[&str], tags: Vec<E>) -> Self {
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
}

pub fn i18n(
  jsx: JSXRuntime,
  options: TranslatorOptions,
  output: &mut Vec<Message>,
) -> impl Fold + VisitMut + '_ {
  let mut options = options;
  let mut elements: HashMap<JSXTagName, Translatable> = Default::default();

  options.elements.drain(..).for_each(|elem| {
    elements.insert(elem.tag.clone(), elem);
  });

  as_folder(Translator {
    jsx,
    options,
    elements,
    messages: output,
    pre: false,
  })
}
