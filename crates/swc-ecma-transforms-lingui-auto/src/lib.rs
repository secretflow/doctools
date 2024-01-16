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
};

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

    fn value_count(&self) -> usize {
        self.0
            .iter()
            .filter(|t| matches!(t, MessageToken::Interpolation(_)))
            .count()
    }

    fn component_count(&self) -> usize {
        self.0
            .iter()
            .filter(|t| matches!(t, MessageToken::OpeningTag(_)))
            .count()
    }
}

struct TranslationPreflight {
    is_translatable: bool,
}

impl Visit for TranslationPreflight {
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
struct TranslationCollector {
    factory: JSXFactory,
    options: TranslationOptions,
    trans: String,
    collected: MessageString,
}

impl VisitMut for TranslationCollector {
    noop_visit_mut_type!();

    fn visit_mut_key_value_prop(&mut self, prop: &mut KeyValueProp) {
        match prop.key {
            PropName::Ident(Ident { ref sym, .. }) if sym.as_str() == "children" => (),
            PropName::Str(Str { ref value, .. }) if value.as_str() == "children" => (),
            _ => return,
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
                    let text = if self.options.pre {
                        value.to_string()
                    } else {
                        collapse_ascii_whitespace(value.as_str())
                            .trim_start_matches(|c: char| c.is_ascii_whitespace())
                            .to_string()
                    };
                    self.collected.0.push(MessageToken::Text(text))
                }
                Expr::Call(mut call) => match self.factory.call_is_jsx(&call) {
                    Some(_) => {
                        let idx = self.collected.component_count();
                        self.collected.0.push(MessageToken::OpeningTag((idx,)));
                        call.visit_mut_children_with(self);
                        self.collected
                            .0
                            .push(MessageToken::ClosingTag((idx, Box::from(call.take()))));
                    }
                    None => {
                        let idx = self.collected.value_count();
                        self.collected
                            .0
                            .push(MessageToken::Interpolation((idx, Box::from(call.take()))));
                    }
                },
                expr => {
                    let idx = self.collected.value_count();
                    self.collected
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

/// Represents Lingui's [`<Trans>`][Trans] component.
///
/// [Trans]: https://lingui.dev/ref/react#trans
struct Translation {
    message: String,
    plaintext: String,
    trans: Box<Expr>,
}

impl TranslationCollector {
    pub fn make_trans(self) -> Translation {
        let message = self.collected.to_message();
        let plaintext = self.collected.to_plaintext();

        let mut components: Vec<PropOrSpread> = vec![];
        let mut values: Vec<PropOrSpread> = vec![];

        self.collected.0.into_iter().for_each(|token| match token {
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

        let trans = self
            .factory
            .create(&JSXElement::Ident(self.trans.as_str().into()))
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

        println!("{}", message);

        Translation {
            message,
            plaintext,
            trans,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(tag = "type")]
pub enum TranslationPhase {
    Capturing,
    Bubbling,
}

impl Default for TranslationPhase {
    fn default() -> Self {
        Self::Bubbling
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct TranslationOptions {
    /// Is the element preformatted like `<pre>` (whitespace is significant)?
    ///
    /// If not, then whitespace is collapsed according to HTML's whitespace rules.
    ///
    /// Default is `false`.
    #[serde(default)]
    pub pre: bool,
    /// Does this element get translated during preorder traversal ([TranslationPhase::Capturing])
    /// or postorder traversal ([TranslationPhase::Bubbling]) (analogous to capturing and bubbling
    /// in event propagation)?
    ///
    /// Default is [TranslationPhase::Bubbling].
    ///
    /// If capturing, this element has "priority" over its children.
    ///
    /// Example:
    ///
    /// ```html
    /// <p>The quick brown <b>fox</b> jumps over the lazy <b>dog</b>.</p>
    /// ```
    ///
    /// If `<p>` is capturing, there would be only one translation for the entire paragraph.
    ///
    /// If `<p>` is not capturing, there would be three translations, one for each `<b>` element,
    /// and one for `<p>`.
    #[serde(default)]
    pub phase: TranslationPhase,
}

impl Default for TranslationOptions {
    fn default() -> Self {
        Self {
            pre: false,
            phase: TranslationPhase::default(),
        }
    }
}

#[derive(Debug)]
pub struct Translator {
    factory: JSXFactory,
    component: String,
    translatables: HashMap<JSXElement, TranslationOptions>,
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
                let phase = options.phase;
                let options = options.clone();

                if matches!(phase, TranslationPhase::Bubbling) {
                    call.visit_mut_children_with(self);
                }

                let mut preflight = TranslationPreflight {
                    is_translatable: false,
                };

                call.visit_children_with(&mut preflight);

                if preflight.is_translatable {
                    let mut collector = TranslationCollector {
                        factory: self.factory.clone(),
                        trans: self.component.clone(),
                        options,
                        collected: MessageString(vec![]),
                    };

                    call.visit_mut_children_with(&mut collector);

                    let translation = collector.make_trans();
                    let mut elem = Box::from(Expr::Call(call.take()));
                    self.factory
                        .set_children(&mut elem, &["children"], vec![translation.trans]);
                    *call = elem.as_mut_call().unwrap().take();
                }

                if matches!(phase, TranslationPhase::Capturing) {
                    call.visit_mut_children_with(self);
                }
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

    pub fn intrinsic(mut self, tag: &str, phase: TranslationPhase) -> Self {
        self.translatables.insert(
            JSXElement::Intrinsic(tag.into()),
            TranslationOptions {
                phase,
                ..Default::default()
            },
        );
        self
    }

    pub fn component(mut self, elem: &str, phase: TranslationPhase) -> Self {
        self.translatables.insert(
            JSXElement::Ident(elem.into()),
            TranslationOptions {
                phase,
                ..Default::default()
            },
        );
        self
    }

    pub fn preformatted(mut self, elem: JSXElement, phase: TranslationPhase) -> Self {
        self.translatables
            .insert(elem, TranslationOptions { pre: true, phase });
        self
    }

    pub fn build(self) -> Self {
        self
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
