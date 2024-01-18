use std::{collections::HashMap, fmt::Debug, vec};

use attribute::translate_attribute;
use flow_content::FlowContentCollector;
use message::Message;
use phrasing_content::{PhrasingContentCollector, PhrasingContentPreflight};
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
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentModel {
    Flow,
    Phrasing,
}

impl Default for ContentModel {
    fn default() -> Self {
        Self::Flow
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransOptions {
    /// Is the element preformatted like `<pre>` (whitespace is significant)?
    ///
    /// If not, then whitespace is collapsed according to HTML's whitespace rules.
    ///
    /// Default is `false`.
    #[serde(default)]
    pub pre: bool,
    /// How will this element be translated? See [ContentModel].
    #[serde(default)]
    pub content: ContentModel,
}

impl Default for TransOptions {
    fn default() -> Self {
        Self {
            pre: false,
            content: ContentModel::default(),
        }
    }
}

#[derive(Debug)]
pub struct Translator {
    factory: JSXFactory,
    trans: String,
    i18n: String,

    elements: HashMap<JSXElement, TransOptions>,
    props: HashMap<JSXElement, Vec<Vec<Lit>>>,

    messages: Vec<Message>,
}

impl VisitMut for Translator {
    noop_visit_mut_type!();

    fn visit_mut_call_expr(&mut self, call: &mut CallExpr) {
        let element = self.factory.call_is_jsx(call);

        let element = match element {
            Some(elem) => elem,
            None => {
                call.visit_mut_children_with(self);
                return;
            }
        };

        if let Some(props) = self.props.get(&element) {
            let i18n = self.i18n.clone();
            props.iter().for_each(|prop| {
                if let Some(message) = translate_attribute(call, &prop, i18n.as_str()) {
                    self.messages.push(message);
                }
            })
        }

        let options = self.elements.get(&element);

        match options {
            None => call.visit_mut_children_with(self),
            Some(options) => {
                match options.content {
                    ContentModel::Flow => {
                        let mut collector = FlowContentCollector::new(
                            self.factory.clone(),
                            &self.trans,
                            options.pre,
                        );

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
                                &self.trans,
                                options.pre,
                            );

                            call.visit_mut_children_with(&mut collector);

                            let (message, children) = collector.result();

                            self.factory.set_prop(
                                call,
                                &["children"],
                                with_span(Some(call.span()))(children),
                            );

                            self.messages.push(message);
                        }
                    }
                };
                call.visit_mut_children_with(self);
            }
        }
    }
}

impl Default for Translator {
    fn default() -> Self {
        Self {
            factory: Default::default(),
            trans: "Trans".into(),
            i18n: "i18n".into(),
            elements: Default::default(),
            props: Default::default(),
            messages: vec![],
        }
    }
}

impl Translator {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn factory(mut self, factory: JSXFactory) -> Self {
        self.factory = factory;
        self
    }

    pub fn flow<E: Into<JSXElement>>(mut self, tag: E) -> Self {
        self.elements.insert(
            tag.into(),
            TransOptions {
                content: ContentModel::Flow,
                ..Default::default()
            },
        );
        self
    }

    pub fn phrasing<E: Into<JSXElement>>(mut self, tag: E) -> Self {
        self.elements.insert(
            tag.into(),
            TransOptions {
                content: ContentModel::Phrasing,
                ..Default::default()
            },
        );
        self
    }

    pub fn preformatted<E: Into<JSXElement>>(mut self, tag: E, content: ContentModel) -> Self {
        self.elements.insert(
            tag.into(),
            TransOptions {
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
            TransOptions {
                content: ContentModel::Flow,
                ..Default::default()
            },
        );
        self
    }

    pub fn prop<E: Into<JSXElement> + Debug>(mut self, prop: &[&str], tags: Vec<E>) -> Self {
        tags.into_iter().for_each(|name| {
            let tag = name.into();
            let props = self.props.entry(tag).or_default();
            props.push(prop.iter().map(|s| Lit::from(*s)).collect())
        });
        self
    }

    pub fn inline(self) -> Self {
        // https://html.spec.whatwg.org/#phrasing-content
        self.phrasing("a")
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
                    "audio", "embed", "iframe", "img", "input", "script", "source", "track",
                    "video",
                ],
            )
    }

    pub fn paragraphs(self) -> Self {
        self.phrasing("p")
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
        self.flow("div")
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

#[cfg(test)]
mod tests {
    use base64::{prelude::BASE64_STANDARD, Engine};
    use swc_core::{
        common::{sync::Lrc, FileName, SourceMap},
        ecma::{
            codegen::{text_writer::JsWriter, Emitter, Node as _},
            visit::VisitMutWith,
        },
    };
    use swc_utils::jsx::factory::JSXFactory;

    use html5jsx::html_to_jsx;

    use super::Translator;

    #[test]
    fn main() {
        let raw = r##"

        <section>
        <h2>My Cats</h2>
        You can play with my cat simulator.
        <object data="cats.sim">
         To see the cat simulator, use one of the following links:
         <ul>
          <li><a href="cats.sim">Download simulator file</a>
          <li><a href="https://sims.example.com/watch?v=LYds5xY4INU">Use online simulator</a>
         </ul>
         Alternatively, upgrade to the Mellblom Browser.
        </object>
        I'm quite proud of it.
       </section>

        "##;

        let sourcemap = Lrc::new(SourceMap::default());
        let source = sourcemap.new_source_file(FileName::Anon, raw.to_string());

        let jsx = JSXFactory::default();
        let mut fragment = html_to_jsx(&source, Some(jsx.clone())).unwrap();

        let mut translator = Translator::new()
            .factory(jsx)
            .fragment()
            .inline()
            .sections()
            .paragraphs()
            .pre()
            .build();

        fragment.body.visit_mut_with(&mut translator);

        let mut code = vec![];
        let mut srcmap = vec![];

        {
            let mut srcmap_raw = vec![];
            let mut emitter = Emitter {
                cfg: Default::default(),
                cm: sourcemap.clone(),
                comments: None,
                wr: JsWriter::new(sourcemap.clone(), "\n", &mut code, Some(&mut srcmap_raw)),
            };
            (*fragment.body).emit_with(&mut emitter).unwrap();
            sourcemap
                .build_source_map(&srcmap_raw)
                .to_writer(&mut srcmap)
                .unwrap();
        }

        let mut result = String::from_utf8(code).unwrap();
        result.push_str("\n//# sourceMappingURL=data:application/json;base64,");
        BASE64_STANDARD.encode_string(srcmap, &mut result);

        translator.messages.iter().for_each(|msg| {
            println!("msgid: {}", msg.message);
        });

        println!("{}", result);
    }
}
