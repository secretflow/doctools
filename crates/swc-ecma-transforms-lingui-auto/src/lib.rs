use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use swc_core::{
    common::util::take::Take,
    ecma::{
        ast::{
            ArrayLit, CallExpr, Expr, ExprOrSpread, Ident, KeyValueProp, Lit, ObjectLit, Prop,
            PropName, PropOrSpread, Str,
        },
        visit::{
            noop_visit_mut_type, noop_visit_type, Visit, VisitMut, VisitMutWith as _, VisitWith,
        },
    },
};

use swc_utils::{
    jsx::factory::{JSXElement, JSXFactory},
    props,
    span::with_span,
};

/// Content model determines how the element is translated.
///
/// https://html.spec.whatwg.org/#concept-element-content-model
///
/// [ContentModel::Flow] — The element is being treated like a container for
/// [flow content]. Elements like `<section>` `<div>` belong to this model.
///
/// [flow content]: https://developer.mozilla.org/en-US/docs/Web/HTML/Content_categories#flow_content
///
/// For "flow content", translatable text is collected "shallowly": each consecutive text content
/// along the element's immediate children is concatenated into a single translation.
///
/// [ContentModel::Phrasing] — The element is being treated like a container for
/// [phrasing content]. Elements like `<p>` `<strong>` `<em>` belong to this model.
///
/// [phrasing content]: https://developer.mozilla.org/en-US/docs/Web/HTML/Content_categories#phrasing_content
///
/// For "phrasing content", translatable text is collected "deeply": each element,
/// if it contains any translatable text, gets one message, no matter how deeply nested
/// it is. This is meant to allow coherent paragraphs of potentially "rich" content
/// (interpolated with markups) to be translated as a whole, preserving context.
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

#[derive(Debug)]
enum MessageToken {
    Text(String),
    Interpolation((usize, Box<Expr>)),
    OpeningTag((usize,)),
    ClosingTag((usize, Box<Expr>)),
}

#[derive(Debug)]
struct MessageString(Vec<MessageToken>);

impl MessageString {
    fn to_message(&self) -> String {
        self.0
            .iter()
            .map(|token| match token {
                MessageToken::Text(text) => text.to_string(),
                MessageToken::Interpolation((idx, _)) => format!("{{{}}}", idx),
                MessageToken::OpeningTag((idx,)) => format!("<{}>", idx),
                MessageToken::ClosingTag((idx, _)) => format!("</{}>", idx),
            })
            .collect::<Vec<_>>()
            .join("")
    }

    fn to_plaintext(&self) -> String {
        let text = self
            .0
            .iter()
            .map(|c| match c {
                MessageToken::Text(text) => text.trim().to_string(),
                MessageToken::Interpolation(_) => String::from("..."),
                MessageToken::OpeningTag(_) => "".into(),
                MessageToken::ClosingTag(_) => "".into(),
            })
            .collect::<Vec<_>>()
            .join(" ");
        collapse_ascii_whitespace(text.as_str()).trim().to_string()
    }

    fn current_value_idx(&self) -> usize {
        self.0
            .iter()
            .filter(|t| matches!(t, MessageToken::Interpolation(_)))
            .count()
            + 1
    }

    fn current_component_idx(&self) -> usize {
        self.0
            .iter()
            .filter(|t| matches!(t, MessageToken::OpeningTag(_)))
            .count()
            + 1
    }

    fn make_trans(self, factory: &JSXFactory, trans: &str) -> Translation {
        let message = self.to_message();
        let plaintext = self.to_plaintext();

        let mut components: Vec<PropOrSpread> = vec![];
        let mut values: Vec<PropOrSpread> = vec![];

        self.0.into_iter().for_each(|token| match token {
            MessageToken::ClosingTag((idx, mut expr)) => components.push(PropOrSpread::Prop(
                Prop::from(KeyValueProp {
                    key: PropName::Num(idx.into()),
                    value: expr.take(),
                })
                .into(),
            )),
            MessageToken::Interpolation((idx, mut expr)) => values.push(PropOrSpread::Prop(
                Prop::from(KeyValueProp {
                    key: PropName::Num(idx.into()),
                    value: expr.take(),
                })
                .into(),
            )),
            _ => {}
        });

        let trans = factory
            .create(&JSXElement::Ident(trans.into()))
            .props(Some(props!(
                "message" = String::from(message.as_str()),
                "components" = ObjectLit {
                    props: components,
                    span: Default::default()
                },
                "values" = ObjectLit {
                    props: values,
                    span: Default::default()
                }
            )))
            .build();

        Translation {
            message,
            plaintext,
            trans: Some(trans),
        }
    }
}

/// Represents Lingui's [`<Trans>`][Trans] component.
///
/// [Trans]: https://lingui.dev/ref/react#trans
#[derive(Debug)]
struct Translation {
    pub message: String,
    pub plaintext: String,
    trans: Option<Box<Expr>>,
}

#[derive(Debug)]
struct FlowContentCollector {
    factory: JSXFactory,
    pre: bool,
    trans: String,
    results: Vec<Translation>,
}

impl VisitMut for FlowContentCollector {
    noop_visit_mut_type!();

    fn visit_mut_key_value_prop(&mut self, prop: &mut KeyValueProp) {
        if !prop_is_children(prop) {
            return;
        }

        let make_chunk = |text: &str| -> Option<String> {
            if is_empty_or_whitespace(text) {
                None
            } else {
                let text = if self.pre {
                    text.to_string()
                } else {
                    collapse_ascii_whitespace(text)
                        .trim_start_matches(|c: char| c.is_ascii_whitespace())
                        .to_string()
                };
                if text.is_empty() {
                    None
                } else {
                    Some(text)
                }
            }
        };

        let mut create_message = |trans: MessageString| -> Expr {
            let mut trans = trans.make_trans(&self.factory, &self.trans);
            let mut repl = trans.trans.take().unwrap();
            self.results.push(trans);
            *repl.take()
        };

        let mut value = *prop.value.take();

        prop.value = match value {
            Expr::Lit(Lit::Str(Str {
                value: ref text,
                span,
                ..
            })) => {
                let text = &*text;
                match make_chunk(text) {
                    None => value,
                    Some(text) => {
                        let result = create_message(MessageString(vec![MessageToken::Text(text)]));
                        with_span(span)(result)
                    }
                }
            }
            Expr::Array(ArrayLit {
                ref mut elems,
                span,
            }) => {
                let mut array_items: Vec<Option<ExprOrSpread>> = vec![];
                let mut current_msg = MessageString(vec![]);

                let mut flush = |trans: MessageString| {
                    if trans.0.len() == 0 {
                        return (None, MessageString(vec![]));
                    }
                    let result = create_message(trans);
                    (
                        Some(ExprOrSpread {
                            expr: Box::from(result),
                            spread: None,
                        }),
                        MessageString(vec![]),
                    )
                };

                let mut iter = elems.drain(..);

                loop {
                    let next = iter.next();
                    match next {
                        None => {
                            let (item, _) = flush(current_msg);
                            if item.is_some() {
                                array_items.push(item);
                            };
                            break;
                        }
                        Some(None) => {
                            let (item, next) = flush(current_msg);
                            if item.is_some() {
                                array_items.push(item);
                            };
                            current_msg = next;
                        }
                        Some(Some(mut expr)) => {
                            if expr.spread.is_some() {
                                let (item, next) = flush(current_msg);
                                if item.is_some() {
                                    array_items.push(item);
                                };
                                array_items.push(Some(expr));
                                current_msg = next;
                            } else {
                                match *expr.expr.take() {
                                    Expr::Lit(Lit::Str(Str {
                                        value: ref text, ..
                                    })) => {
                                        let text = &*text;
                                        match make_chunk(text) {
                                            None => (),
                                            Some(text) => {
                                                current_msg.0.push(MessageToken::Text(text))
                                            }
                                        }
                                    }
                                    Expr::Call(call)
                                        if self.factory.call_is_jsx(&call).is_some() =>
                                    {
                                        let (item, next) = flush(current_msg);
                                        if item.is_some() {
                                            array_items.push(item);
                                        };
                                        array_items.push(Some(ExprOrSpread {
                                            expr: Box::from(call),
                                            spread: None,
                                        }));
                                        current_msg = next;
                                    }
                                    expr => {
                                        let idx = current_msg.current_value_idx();
                                        current_msg.0.push(MessageToken::Interpolation((
                                            idx,
                                            Box::from(expr),
                                        )));
                                    }
                                }
                            }
                        }
                    }
                }

                Expr::Array(ArrayLit {
                    span,
                    elems: array_items,
                })
            }
            expr => expr,
        }
        .into();
    }
}

/// For [phrasing][ContentModel::Phrasing] content, transform is done in two phases.
///
/// 1. [TranslationPreflight] visits the tree **immutably** and determines if the element
///    is translatable i.e. if any non-whitespace text is present within the element
///    (think [Element.innerText])
/// 2. If it is indeed translatable, [TranslationCollector] visits the tree **mutably**
///    and transform it into `<Trans>`
///
/// [Element.innerText]: https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/innerText
///
/// The first visit obviously adds extra overhead, but the alternative would be trying
/// to determine whether the element is translatable while borrowing it mutably. Because
/// whether the element has any text cannot be readily determined without visiting its
/// (arbitrarily deep) descendants, trying to avoid `mut` until proven necessary would
/// involve a lot of backtracking / conditionals / very fragile
/// [AST node taking][swc_core::common::util::take::Take]. This is much less ergonomic and
/// more error-prone than just visiting the tree twice.
struct PhrasingContentPreflight {
    is_translatable: bool,
}

impl Default for PhrasingContentPreflight {
    fn default() -> Self {
        Self {
            is_translatable: false,
        }
    }
}

impl Visit for PhrasingContentPreflight {
    noop_visit_type!();

    fn visit_call_expr(&mut self, call: &CallExpr) {
        if self.is_translatable {
            return;
        }
        call.visit_children_with(self);
    }

    fn visit_key_value_prop(&mut self, prop: &KeyValueProp) {
        match prop.key {
            PropName::Ident(Ident { ref sym, .. }) if sym.as_str() == "children" => (),
            PropName::Str(Str { ref value, .. }) if value.as_str() == "children" => (),
            _ => return,
        }

        self.is_translatable = match &*prop.value {
            Expr::Array(ArrayLit { ref elems, .. }) => elems.iter().any(|expr| match expr {
                Some(ExprOrSpread { expr, .. }) => match &**expr {
                    Expr::Lit(Lit::Str(Str { value, .. })) => !is_empty_or_whitespace(&value),
                    _ => false,
                },
                None => false,
            }),
            Expr::Lit(Lit::Str(text)) => !is_empty_or_whitespace(&text.value),
            _ => false,
        };

        if !self.is_translatable {
            prop.visit_children_with(self);
        }
    }
}

#[derive(Debug)]
struct PhrasingContentCollector {
    factory: JSXFactory,
    pre: bool,
    message: MessageString,
}

impl VisitMut for PhrasingContentCollector {
    noop_visit_mut_type!();

    fn visit_mut_key_value_prop(&mut self, prop: &mut KeyValueProp) {
        if !prop_is_children(prop) {
            return;
        }

        let children = match *prop.value.take() {
            Expr::Array(ArrayLit { mut elems, .. }) => elems
                .iter_mut()
                .filter_map(|expr| match expr {
                    None => None,
                    Some(ExprOrSpread { expr, .. }) => Some(expr.take()),
                })
                .collect::<Vec<_>>(),
            expr => vec![Box::from(expr)],
        };

        children
            .into_iter()
            .for_each(|mut expr| match *expr.take() {
                Expr::Lit(Lit::Str(Str { value, .. })) => {
                    let text = if self.pre {
                        value.to_string()
                    } else {
                        collapse_ascii_whitespace(value.as_str())
                            .trim_start_matches(|c: char| c.is_ascii_whitespace())
                            .to_string()
                    };
                    self.message.0.push(MessageToken::Text(text))
                }
                Expr::Call(mut call) => match self.factory.call_is_jsx(&call) {
                    Some(_) => {
                        let idx = self.message.current_component_idx();
                        self.message.0.push(MessageToken::OpeningTag((idx,)));
                        call.visit_mut_children_with(self);
                        self.message
                            .0
                            .push(MessageToken::ClosingTag((idx, Box::from(call.take()))));
                    }
                    None => {
                        let idx = self.message.current_value_idx();
                        self.message
                            .0
                            .push(MessageToken::Interpolation((idx, Box::from(call.take()))));
                    }
                },
                expr => {
                    let idx = self.message.current_value_idx();
                    self.message
                        .0
                        .push(MessageToken::Interpolation((idx, Box::from(expr))));
                }
            });
    }

    fn visit_mut_object_lit(&mut self, object: &mut ObjectLit) {
        object.visit_mut_children_with(self);
        object.props = object
            .props
            .drain(..)
            .filter(|prop| {
                prop.as_prop()
                    .and_then(|p| p.as_key_value())
                    .and_then(|p| Some(!p.value.is_invalid()))
                    .unwrap_or(false)
            })
            .collect();
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
    messages: Vec<Translation>,
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
                        let mut collector = FlowContentCollector {
                            factory: self.factory.clone(),
                            trans: self.component.clone(),
                            pre: options.pre,
                            results: vec![],
                        };
                        call.visit_mut_children_with(&mut collector);
                        self.messages.extend(collector.results);
                    }
                    ContentModel::Phrasing => {
                        let mut preflight = PhrasingContentPreflight::default();
                        call.visit_children_with(&mut preflight);
                        if preflight.is_translatable {
                            let mut collector = PhrasingContentCollector {
                                factory: self.factory.clone(),
                                pre: options.pre,
                                message: MessageString(vec![]),
                            };
                            call.visit_mut_children_with(&mut collector);
                            let mut translation =
                                collector.message.make_trans(&self.factory, &self.component);
                            let mut elem = Box::from(Expr::Call(call.take()));
                            self.factory.set_children(
                                &mut elem,
                                &["children"],
                                vec![translation.trans.take().unwrap()],
                            );
                            *call = elem.as_mut_call().unwrap().take();
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
        self.preformatted("pre", ContentModel::Flow)
    }

    pub fn build(self) -> Self {
        self
    }
}

fn prop_is_children(prop: &KeyValueProp) -> bool {
    match prop.key {
        PropName::Ident(Ident { ref sym, .. }) if sym.as_str() == "children" => true,
        PropName::Str(Str { ref value, .. }) if value.as_str() == "children" => true,
        _ => false,
    }
}

/// Collapse whitespace according to HTML's whitespace rules.
///
/// https://infra.spec.whatwg.org/#ascii-whitespace
fn collapse_ascii_whitespace(str: &str) -> String {
    let mut result = String::new();
    let mut last_char = '\0';
    str.chars().for_each(|c: char| {
        if c.is_ascii_whitespace() {
            if last_char.is_ascii_whitespace() {
                ()
            } else {
                last_char = c;
                result.push(' ');
            }
        } else {
            last_char = c;
            result.push(c);
        }
    });
    result
}

fn is_empty_or_whitespace(str: &str) -> bool {
    str.chars().all(|c| c.is_ascii_whitespace())
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
        <article itemscope itemtype="http://schema.org/BlogPosting">
        <header>
            <h2 itemprop="headline">The Very First Rule of Life</h2>
            <p><time itemprop="datePublished" datetime="2009-10-09">3 days ago</time></p>
            <link itemprop="url" href="?comments=0">
        </header>
        <p>If there's a microphone anywhere near you, assume it's hot and
        sending whatever you're saying to the world. Seriously.</p>
        <p>...</p>
        <section>
            <h1>Comments</h1>
            <article itemprop="comment" itemscope itemtype="http://schema.org/Comment" id="c1">
            <link itemprop="url" href="#c1">
            <footer>
            <p>Posted by: <span itemprop="creator" itemscope itemtype="http://schema.org/Person">
            <span itemprop="name">George Washington</span>
            </span></p>
            <p><time itemprop="dateCreated" datetime="2009-10-10">15 minutes ago</time></p>
            </footer>
            <p>Yeah! Especially when talking about your lobbyist friends!</p>
            </article>
            <article itemprop="comment" itemscope itemtype="http://schema.org/Comment" id="c2">
            <link itemprop="url" href="#c2">
            <footer>
            <p>Posted by: <span itemprop="creator" itemscope itemtype="http://schema.org/Person">
            <span itemprop="name">George Hammond</span>
            </span></p>
            <p><time itemprop="dateCreated" datetime="2009-10-10">5 minutes ago</time></p>
            </footer>
            <p>Hey, you have the same first name as me.</p>
            </article>
        </section>
        </article>
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

        println!("{}", result);
    }
}
