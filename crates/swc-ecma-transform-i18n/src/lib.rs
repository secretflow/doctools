use std::{collections::HashMap, fmt::Debug, marker::PhantomData};

use serde::{Deserialize, Serialize};
use swc_core::ecma::{
  ast::CallExpr,
  visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith as _},
};

use swc_ecma_utils2::{
  jsx::{
    jsx, jsx_mut,
    tag::{JSXTag, JSXTagType},
    JSXElement, JSXElementMut, JSXRuntime,
  },
  jsx_tag,
};

mod attribute;
mod flow_content;
mod message;
mod phrasing_content;
mod symbols;

pub use crate::symbols::I18nSymbols;

use crate::attribute::translate_attrs;
use crate::flow_content::translate_block;
use crate::message::Message;
use crate::phrasing_content::translate_phrase;

/// Content model determines how the element is translated.
///
/// <https://html.spec.whatwg.org/#concept-element-content-model>
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
  pub tag: JSXTag,
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
      tag: jsx_tag!(<>),
      content: ContentModel::Flow,
      pre: false,
      props: vec![],
    }
  }
}

impl Translatable {
  pub fn flow(tag: JSXTag) -> Self {
    Self {
      tag,
      content: ContentModel::Flow,
      pre: false,
      props: vec![],
    }
  }

  pub fn phrasing(tag: JSXTag) -> Self {
    Self {
      tag: tag,
      content: ContentModel::Phrasing,
      pre: false,
      props: vec![],
    }
  }

  pub fn preformatted(tag: JSXTag, content: ContentModel) -> Self {
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
  #[serde(default)]
  elements: Vec<Translatable>,
}

impl TranslatorOptions {
  pub fn new() -> Self {
    Self::default()
  }
}

impl Default for TranslatorOptions {
  fn default() -> Self {
    Self { elements: vec![] }
  }
}

#[derive(Debug)]
struct Translator<'m, R, S>
where
  R: JSXRuntime,
  S: I18nSymbols,
{
  elements: HashMap<JSXTag, Translatable>,
  pre: bool,
  messages: &'m mut Vec<Message>,

  jsx: PhantomData<R>,
  i18n: PhantomData<S>,
}

impl<R: JSXRuntime, S: I18nSymbols> Translator<'_, R, S> {
  fn translate_generic_attrs(&mut self, call: &mut CallExpr) -> Option<()> {
    let props = jsx_mut::<R>(call)?.get_props_mut()?;

    self.messages.extend(translate_attrs::<R, S>(
      props,
      vec![
        vec!["title"],
        vec!["aria-label"],
        vec!["aria-placeholder"],
        vec!["aria-roledescription"],
        vec!["aria-valuetext"],
      ],
    ));

    Some(())
  }

  fn translate_call_expr(&mut self, call: &mut CallExpr) -> Option<()> {
    let tag = jsx::<R>(call)?.get_tag()?;

    if matches!(tag.tag_type(), JSXTagType::Intrinsic(_)) {
      self.translate_generic_attrs(call);
    }

    let options = self.elements.get(&tag)?;

    let attrs = options
      .props
      .iter()
      .map(|ss| ss.iter().map(|s| s.as_str()).collect::<Vec<_>>())
      .collect::<Vec<_>>();

    let props = jsx_mut::<R>(call)?.get_props_mut()?;
    self.messages.extend(translate_attrs::<R, S>(props, attrs));

    self.pre = self.pre || options.pre;

    match options.content {
      ContentModel::Flow => {
        self
          .messages
          .extend(translate_block::<R, S>(options.pre, call));
      }
      ContentModel::Phrasing => {
        if let Some(message) = translate_phrase::<R, S>(options.pre, call) {
          self.messages.push(message);
        }
      }
    };

    Some(())
  }
}

impl<R: JSXRuntime, S: I18nSymbols> VisitMut for Translator<'_, R, S> {
  noop_visit_mut_type!();

  fn visit_mut_call_expr(&mut self, call: &mut CallExpr) {
    let pre_parent = self.pre;
    self.translate_call_expr(call);
    call.visit_mut_children_with(self);
    self.pre = pre_parent;
  }
}

pub fn i18n<'m, R: JSXRuntime + 'm, S: I18nSymbols + 'm>(
  options: TranslatorOptions,
  output: &'m mut Vec<Message>,
) -> impl Fold + VisitMut + 'm {
  let mut options = options;
  let mut elements: HashMap<JSXTag, Translatable> = Default::default();

  options.elements.drain(..).for_each(|elem| {
    elements.insert(elem.tag.clone(), elem);
  });

  as_folder(Translator::<'m, R, S> {
    elements,
    messages: output,
    pre: false,
    jsx: PhantomData,
    i18n: PhantomData,
  })
}
