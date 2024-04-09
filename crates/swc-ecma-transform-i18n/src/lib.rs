use std::{collections::HashMap, marker::PhantomData};

use attribute::translate_attrs;
use serde::{Deserialize, Serialize};
use swc_core::{
  common::util::take::Take,
  ecma::{
    ast::CallExpr,
    visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith as _},
  },
};

use swc_ecma_utils2::{
  ad_hoc_tag,
  collections::MutableMapping as _,
  jsx::{JSXElement, JSXElementMut, JSXRuntime, JSXTagTypeOwned},
  matches_tag,
};

mod attribute;
mod flow_content;
mod message;
mod phrasing_content;
mod symbols;

pub use crate::symbols::I18nSymbols;

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
struct TranslatableOptions {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Translatable {
  /// What element is this?
  pub tag: JSXTagTypeOwned,
  #[serde(flatten)]
  options: TranslatableOptions,
}

impl Default for Translatable {
  fn default() -> Self {
    Self {
      tag: ad_hoc_tag!(<>).into(),
      options: TranslatableOptions {
        pre: false,
        content: ContentModel::Flow,
        props: vec![],
      },
    }
  }
}

impl Translatable {
  pub fn flow(tag: JSXTagTypeOwned) -> Self {
    Self {
      tag,
      options: TranslatableOptions {
        content: ContentModel::Flow,
        pre: false,
        props: vec![],
      },
    }
  }

  pub fn phrasing(tag: JSXTagTypeOwned) -> Self {
    Self {
      tag,
      options: TranslatableOptions {
        content: ContentModel::Phrasing,
        pre: false,
        props: vec![],
      },
    }
  }

  pub fn preformatted(tag: JSXTagTypeOwned, content: ContentModel) -> Self {
    Self {
      tag,
      options: TranslatableOptions {
        content,
        pre: true,
        props: vec![],
      },
    }
  }

  pub fn prop(mut self, prop: &[&str]) -> Self {
    self
      .options
      .props
      .push(prop.iter().map(|s| (*s).into()).collect());
    self
  }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslatorOptions {
  #[serde(default)]
  elements: Vec<Translatable>,
}

#[derive(Debug)]
struct Translator<'m, R, S>
where
  R: JSXRuntime,
  S: I18nSymbols,
{
  elements: HashMap<JSXTagTypeOwned, TranslatableOptions>,
  pre: bool,
  messages: &'m mut Vec<Message>,

  jsx: PhantomData<R>,
  i18n: PhantomData<S>,
}

impl<R: JSXRuntime, S: I18nSymbols> Translator<'_, R, S> {
  fn translate_attrs(&mut self, call: &mut CallExpr) -> Option<()> {
    {
      let tag = call.as_jsx_type::<R>()?;
      if matches_tag!(tag, "*"?) {
        let props = call.as_jsx_props_mut::<R>()?.as_mut_object()?;
        let messages = translate_attrs::<R, S>(
          props,
          &[
            vec!["title".to_string()],
            vec!["aria-label".to_string()],
            vec!["aria-placeholder".to_string()],
            vec!["aria-roledescription".to_string()],
            vec!["aria-valuetext".to_string()],
          ],
        );
        self.messages.extend(messages);
      }
    }

    {
      let mut props = call.as_jsx_props_mut::<R>()?.as_mut_object()?.take();
      let Some(tag) = call.as_jsx_type::<R>() else {
        call.set_item(2usize, props.into());
        return None;
      };
      let Some(options) = self.elements.get(&tag.into()) else {
        call.set_item(2usize, props.into());
        return None;
      };
      let attrs = &options.props;
      let messages = translate_attrs::<R, S>(&mut props, attrs);
      self.messages.extend(messages);
      call.set_item(2usize, props.into());
    }

    Some(())
  }

  fn translate_call_expr(&mut self, call: &mut CallExpr) -> Option<()> {
    self.translate_attrs(call);

    let (pre, content) = {
      let tag = call.as_jsx_type::<R>()?.clone();
      let options = self.elements.get(&tag.into())?;
      (options.pre, options.content)
    };

    self.pre = self.pre || pre;

    match content {
      ContentModel::Flow => {
        self.messages.extend(translate_block::<R, S>(pre, call));
      }
      ContentModel::Phrasing => {
        if let Some(message) = translate_phrase::<R, S>(pre, call) {
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
  let mut elements: HashMap<JSXTagTypeOwned, TranslatableOptions> = Default::default();

  options.elements.drain(..).for_each(|elem| {
    elements.insert(elem.tag, elem.options);
  });

  as_folder(Translator::<'m, R, S> {
    elements,
    messages: output,
    pre: false,
    jsx: PhantomData,
    i18n: PhantomData,
  })
}
