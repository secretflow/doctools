use std::collections::HashMap;

use flow_content::FlowContentCollector;
use message::Message;
use phrasing_content::{PhrasingContentCollector, PhrasingContentPreflight};
use serde::{Deserialize, Serialize};
use swc_core::ecma::{
    ast::CallExpr,
    visit::{noop_visit_mut_type, VisitMut, VisitMutWith as _, VisitWith},
};

use swc_utils::jsx::factory::{JSXElement, JSXFactory};

mod flow_content;
mod message;
mod phrasing_content;
mod prop;

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
pub struct TranslationOptions {
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

impl Default for TranslationOptions {
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
    component: String,
    translatables: HashMap<JSXElement, TranslationOptions>,
    messages: Vec<Message>,
}

impl VisitMut for Translator {
    noop_visit_mut_type!();

    fn visit_mut_call_expr(&mut self, call: &mut CallExpr) {
        let translatable = match self.factory.call_is_jsx(call) {
            None => None,
            Some(elem) => self.translatables.get(&elem),
        };

        match translatable {
            None => call.visit_mut_children_with(self),
            Some(options) => {
                match options.content {
                    ContentModel::Flow => {
                        let mut collector = FlowContentCollector::new(
                            self.factory.clone(),
                            &self.component,
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
                                &self.component,
                                options.pre,
                            );

                            call.visit_mut_children_with(&mut collector);

                            let (message, children) = collector.result();

                            self.factory.set_prop(call, &["children"], children);

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
            component: "Trans".into(),
            translatables: Default::default(),
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
        self.translatables.insert(
            tag.into(),
            TranslationOptions {
                content: ContentModel::Flow,
                ..Default::default()
            },
        );
        self
    }

    pub fn phrasing<E: Into<JSXElement>>(mut self, tag: E) -> Self {
        self.translatables.insert(
            tag.into(),
            TranslationOptions {
                content: ContentModel::Phrasing,
                ..Default::default()
            },
        );
        self
    }

    pub fn preformatted<E: Into<JSXElement>>(mut self, tag: E, content: ContentModel) -> Self {
        self.translatables
            .insert(tag.into(), TranslationOptions { pre: true, content });
        self
    }

    pub fn fragment(mut self) -> Self {
        self.translatables.insert(
            JSXElement::Fragment,
            TranslationOptions {
                content: ContentModel::Flow,
                ..Default::default()
            },
        );
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

        <dl class="element"><dt><a href="dom.html#concept-element-categories" id="the-main-element:concept-element-categories">Categories</a>:</dt><dd><a id="the-main-element:flow-content-2" href="dom.html#flow-content-2">Flow content</a>.</dd><dd><a id="the-main-element:palpable-content-2" href="dom.html#palpable-content-2">Palpable content</a>.</dd><dt><a href="dom.html#concept-element-contexts" id="the-main-element:concept-element-contexts">Contexts in which this element can be used</a>:</dt><dd>Where <a id="the-main-element:flow-content-2-2" href="dom.html#flow-content-2">flow content</a> is expected, but only if it is a <a href="#hierarchically-correct-main-element" id="the-main-element:hierarchically-correct-main-element">hierarchically correct
        <code>main</code> element</a>.</dd><dt><a href="dom.html#concept-element-content-model" id="the-main-element:concept-element-content-model">Content model</a>:</dt><dd><a id="the-main-element:flow-content-2-3" href="dom.html#flow-content-2">Flow content</a>.</dd><dt><a href="dom.html#concept-element-tag-omission" id="the-main-element:concept-element-tag-omission">Tag omission in text/html</a>:</dt><dd>Neither tag is omissible.</dd><dt><a href="dom.html#concept-element-attributes" id="the-main-element:concept-element-attributes">Content attributes</a>:</dt><dd><a id="the-main-element:global-attributes" href="dom.html#global-attributes">Global attributes</a></dd><dt><a href="dom.html#concept-element-accessibility-considerations" id="the-main-element:concept-element-accessibility-considerations">Accessibility considerations</a>:</dt><dd><a href="https://w3c.github.io/html-aria/#el-main">For authors</a>.</dd><dd><a href="https://w3c.github.io/html-aam/#el-main">For implementers</a>.</dd><dt><a href="dom.html#concept-element-dom" id="the-main-element:concept-element-dom">DOM interface</a>:</dt><dd>Uses <code id="the-main-element:htmlelement"><a href="dom.html#htmlelement">HTMLElement</a></code>.</dd></dl>

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
