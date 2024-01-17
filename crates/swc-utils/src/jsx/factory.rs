use serde::{Deserialize, Serialize};
use swc_core::{
    atoms::Atom,
    common::Span,
    ecma::ast::{
        ArrayLit, CallExpr, Callee, Expr, ExprOrSpread, Ident, ImportDecl, ImportNamedSpecifier,
        ImportSpecifier, KeyValueProp, Lit, ObjectLit, Prop, PropName, PropOrSpread, Str,
    },
};

use crate::{json::set_object, span::with_span};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub enum JSXElement {
    Intrinsic(Atom),
    Ident(Atom),
    Fragment,
}

impl JSXElement {
    pub fn is_metadata(&self) -> bool {
        match self {
            JSXElement::Intrinsic(name) => match name.as_str() {
                "base" | "link" | "meta" | "noscript" | "script" | "style" | "title" => true,
                _ => false,
            },
            _ => false,
        }
    }
}

impl From<&str> for JSXElement {
    fn from(value: &str) -> Self {
        JSXElement::Intrinsic(value.into())
    }
}

impl From<String> for JSXElement {
    fn from(value: String) -> Self {
        JSXElement::Intrinsic(value.into())
    }
}

impl From<Ident> for JSXElement {
    fn from(value: Ident) -> Self {
        JSXElement::Ident(value.sym)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JSXFactory {
    #[serde(rename = "Fragment")]
    fragment: Atom,
    jsx: Atom,
    jsxs: Atom,
}

pub struct JSXBuilder<'a> {
    factory: &'a JSXFactory,
    name: &'a JSXElement,
    pub arg1: Option<Box<Expr>>,
    pub props: Vec<Box<Prop>>,
    pub children: Vec<ExprOrSpread>,
}

impl JSXFactory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn jsx(mut self, jsx: &str) -> Self {
        self.jsx = jsx.into();
        self
    }

    pub fn jsxs(mut self, jsxs: &str) -> Self {
        self.jsxs = jsxs.into();
        self
    }

    pub fn fragment(mut self, fragment: &str) -> Self {
        self.fragment = fragment.into();
        self
    }

    pub fn names(&self) -> [&str; 3] {
        [
            self.fragment.as_str(),
            self.jsx.as_str(),
            self.jsxs.as_str(),
        ]
    }
}

impl JSXFactory {
    pub fn create<'a>(&'a self, name: &'a JSXElement) -> JSXBuilder<'a> {
        JSXBuilder {
            factory: self,
            name,
            arg1: None,
            props: vec![],
            children: vec![],
        }
    }
}

impl JSXBuilder<'_> {
    pub fn prop(mut self, key: &str, value: Box<Expr>, span: Option<Span>) -> Self {
        self.props.push(with_span(span)(
            (Prop::KeyValue(KeyValueProp {
                key: PropName::Str(Str {
                    value: key.into(),
                    span: Default::default(),
                    raw: None,
                }),
                value,
            }))
            .into(),
        ));
        self
    }

    pub fn children(mut self, mut children: Vec<Box<Expr>>) -> Self {
        self.children.append(
            &mut children
                .drain(..)
                .map(|expr| expr.into())
                .collect::<Vec<_>>(),
        );
        self
    }

    pub fn build(mut self) -> CallExpr {
        let jsx = if self.children.len() > 1 {
            &*self.factory.jsxs
        } else {
            &*self.factory.jsx
        };

        if self.arg1.is_some() && !(self.props.is_empty() || !self.children.is_empty()) {
            unreachable!("arg1 is set but props and children are not empty");
        }

        let props = {
            let mut props = self.props;

            if self.children.len() > 1 {
                // { "children": [jsx(...), jsxs(...), ...] }
                props.push(
                    Prop::from(KeyValueProp {
                        key: PropName::Str("children".into()),
                        value: ArrayLit {
                            elems: self
                                .children
                                .drain(..)
                                .map(|expr| Some(expr.into()))
                                .collect(),
                            span: Default::default(),
                        }
                        .into(),
                    })
                    .into(),
                )
            } else if self.children.len() == 1 {
                // { "children": jsx(...) }
                // { "children": null }
                let value = self.children.pop().unwrap();
                props.push(
                    Prop::from(KeyValueProp {
                        key: PropName::Str("children".into()),
                        value: value.expr,
                    })
                    .into(),
                )
            }

            Expr::from(ObjectLit {
                props: props
                    .into_iter()
                    .map(|prop| PropOrSpread::Prop(prop.into()))
                    .collect(),
                span: Default::default(),
            })
        };

        // jsx("tag", { ...attrs, children: jsx(...) })
        // jsxs("tag", { ...attrs, children: [jsx(...), jsxs(...), ...] })
        CallExpr {
            // jsx(
            callee: Callee::from(Box::from(Ident::from(jsx))),
            args: vec![
                match self.name {
                    JSXElement::Intrinsic(tag) => Expr::from(tag.as_str()).into(),
                    JSXElement::Ident(tag) => Expr::from(Ident::from(tag.as_str())).into(),
                    JSXElement::Fragment => {
                        Expr::from(Ident::from(self.factory.fragment.as_str())).into()
                    }
                },
                props.into(),
            ],
            span: Default::default(),
            type_args: None,
        }
    }
}

impl JSXFactory {
    pub fn import_from(&self, src: &str) -> ImportDecl {
        ImportDecl {
            specifiers: vec![
                ImportSpecifier::Named(ImportNamedSpecifier {
                    local: Ident::from(self.jsx.as_str()),
                    imported: None,
                    is_type_only: false,
                    span: Default::default(),
                }),
                ImportSpecifier::Named(ImportNamedSpecifier {
                    local: Ident::from(self.jsxs.as_str()),
                    imported: None,
                    is_type_only: false,
                    span: Default::default(),
                }),
                ImportSpecifier::Named(ImportNamedSpecifier {
                    local: Ident::from(self.fragment.as_str()),
                    imported: None,
                    is_type_only: false,
                    span: Default::default(),
                }),
            ],
            src: Box::from(Str::from(src)),
            type_only: false,
            with: None,
            span: Default::default(),
        }
    }

    pub fn call_is_jsx(&self, call: &CallExpr) -> Option<JSXElement> {
        match &call.callee {
            Callee::Expr(callee) => match &**callee {
                Expr::Ident(Ident { sym: caller, .. }) => {
                    if caller == &self.jsx || caller == &self.jsxs {
                        match call.args.get(0) {
                            Some(ExprOrSpread { expr, .. }) => match &**expr {
                                Expr::Lit(Lit::Str(Str { value, .. })) => {
                                    Some(JSXElement::Intrinsic(value.as_str().into()))
                                }
                                Expr::Ident(Ident { sym, .. }) => {
                                    if sym.as_str() == self.fragment.as_str() {
                                        Some(JSXElement::Fragment)
                                    } else {
                                        Some(JSXElement::Ident(sym.as_str().into()))
                                    }
                                }
                                _ => None,
                            },
                            _ => None,
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            },
            _ => None,
        }
    }

    pub fn expr_is_jsx(&self, expr: &Box<Expr>) -> Option<JSXElement> {
        match &**expr {
            Expr::Call(call) => self.call_is_jsx(call),
            _ => None,
        }
    }

    pub fn replace_children(
        &self,
        call: &mut CallExpr,
        keypath: &[&str],
        children: Vec<ExprOrSpread>,
    ) {
        let props = call
            .args
            .get_mut(1)
            .and_then(|a| Some(&mut a.expr))
            .expect("expected props in argument");

        match keypath[..] {
            ["children"] => {
                // ensure JSX factory function is correct
                match call.callee.as_mut_expr().and_then(|e| e.as_mut_ident()) {
                    Some(ident) => {
                        if children.len() > 1 {
                            ident.sym = self.jsxs.as_str().into();
                        } else {
                            ident.sym = self.jsx.as_str().into();
                        }
                    }
                    _ => unreachable!(
                        "expected callee to be a string literal, got {:?}",
                        call.callee
                    ),
                };
                // unwrap the array if there's only one child
                let children = if children.len() > 1 {
                    Some(Box::from(Expr::from(ArrayLit {
                        elems: children.into_iter().map(Some).collect(),
                        span: Default::default(),
                    })))
                } else {
                    children.into_iter().last().and_then(|expr| Some(expr.expr))
                };
                if let Some(content) = children {
                    set_object(props, &["children"][..], content);
                };
            }
            _ => {
                // wrap children in Fragment if there's more than one
                let children = if children.len() > 1 {
                    let mut builder = self.create(&JSXElement::Fragment);
                    builder.children = children;
                    Some(Box::from(Expr::from(builder.build())))
                } else {
                    children.into_iter().last().and_then(|expr| Some(expr.expr))
                };
                if let Some(content) = children {
                    set_object(props, keypath, content);
                };
            }
        }
    }

    pub fn prop_is_children(prop: &KeyValueProp) -> bool {
        match prop.key {
            PropName::Ident(Ident { ref sym, .. }) if sym.as_str() == "children" => true,
            PropName::Str(Str { ref value, .. }) if value.as_str() == "children" => true,
            _ => false,
        }
    }
}

impl Default for JSXFactory {
    fn default() -> Self {
        Self {
            fragment: "Fragment".into(),
            jsx: "jsx".into(),
            jsxs: "jsxs".into(),
        }
    }
}

#[macro_export]
macro_rules! props {
    ($obj:expr) => {
        match *$obj {
            swc_core::ecma::ast::Expr::Object(obj) => obj
                .props
                .into_iter()
                .map(|prop| match prop {
                    swc_core::ecma::ast::PropOrSpread::Prop(prop) => prop,
                    swc_core::ecma::ast::PropOrSpread::Spread(_) => unreachable!(),
                })
                .collect(),
            _ => unreachable!(),
        }
    };
    ( $($key:literal = $value:expr),+ ) => {
        vec![
        $(  Prop::KeyValue(KeyValueProp {
                key: PropName::Str($key.into()),
                value: Expr::from($value).into(),
            })
            .into(), )*
        ]
    }
}

#[cfg(test)]
mod tests {
    use swc_core::{
        ecma::{
            ast::{Expr, Ident},
            codegen::Config,
        },
        testing::DebugUsingDisplay,
    };

    use crate::testing::print_one;

    use super::{JSXElement, JSXFactory};

    #[test]
    fn test_fragment() {
        let jsx = JSXFactory::default();
        let elem = jsx.create(&JSXElement::Fragment).build();
        let code = print_one(&elem, None, None);
        assert_eq!(code, "jsx(Fragment, {})");
    }

    #[test]
    fn test_intrinsic() {
        let jsx = JSXFactory::default();
        let elem = jsx
            .create(&JSXElement::Intrinsic("div".into()))
            .children(vec![Box::from(Expr::from(Ident::from("foo")))])
            .build();
        let code = print_one(&elem, None, Some(Config::default().with_minify(true)));
        assert_eq!(
            DebugUsingDisplay(code.as_str()),
            DebugUsingDisplay(r#"jsx("div",{"children":foo})"#)
        );
    }

    #[test]
    fn test_component() {
        let jsx = JSXFactory::default();
        let elem = jsx.create(&JSXElement::Ident("Foo".into())).build();
        let code = print_one(&elem, None, Some(Config::default().with_minify(true)));
        assert_eq!(
            DebugUsingDisplay(code.as_str()),
            DebugUsingDisplay(r#"jsx(Foo,{})"#)
        );
    }

    #[test]
    fn test_jsxs() {
        let jsx = JSXFactory::default();
        let elem = jsx
            .create(&JSXElement::Intrinsic("div".into()))
            .children(vec![
                jsx.create(&JSXElement::Intrinsic("span".into()))
                    .build()
                    .into(),
                jsx.create(&JSXElement::Intrinsic("span".into()))
                    .build()
                    .into(),
            ])
            .build();
        let code = print_one(&elem, None, Some(Config::default().with_minify(true)));
        assert_eq!(
            DebugUsingDisplay(code.as_str()),
            DebugUsingDisplay(r#"jsxs("div",{"children":[jsx("span",{}),jsx("span",{})]})"#)
        );
    }

    #[test]
    fn test_props() {
        let jsx = JSXFactory::default();
        let elem = jsx
            .create(&JSXElement::Intrinsic("div".into()))
            .prop("className", "foo".into(), None)
            .prop("id", "bar".into(), None)
            .build();
        let code = print_one(&elem, None, Some(Config::default().with_minify(true)));
        assert_eq!(
            DebugUsingDisplay(code.as_str()),
            DebugUsingDisplay(r#"jsx("div",{"className":"foo","id":"bar"})"#)
        );
    }
}
