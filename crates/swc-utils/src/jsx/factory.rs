use serde::{Deserialize, Serialize};
use swc_core::{
    atoms::Atom,
    ecma::ast::{
        ArrayLit, CallExpr, Callee, Expr, ExprOrSpread, Ident, ImportDecl, ImportNamedSpecifier,
        ImportSpecifier, KeyValueProp, Lit, ObjectLit, Prop, PropName, PropOrSpread, Str,
    },
};

use crate::json::set_object;

pub enum JSXElement {
    Intrinsic(Atom),
    Ident(Atom),
    Fragment,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct JSXFactory {
    #[serde(rename = "Fragment")]
    pub fragment: Atom,
    pub jsx: Atom,
    pub jsxs: Atom,
}

impl JSXFactory {
    pub fn create(
        &self,
        name: &JSXElement,
        props: Option<Vec<Box<Prop>>>,
        children: Option<Vec<Box<Expr>>>,
    ) -> Box<Expr> {
        let jsx = match children {
            Some(ref children) if children.len() > 1 => &self.jsxs,
            _ => &self.jsx,
        };

        let mut props = match props {
            Some(props) => props,
            None => vec![],
        };

        match children {
            Some(children) => {
                if children.len() > 1 {
                    // { "children": [jsx(...), jsxs(...), ...] }
                    props.push(
                        Prop::from(KeyValueProp {
                            key: PropName::Str("children".into()),
                            value: ArrayLit {
                                elems: children.into_iter().map(|expr| Some(expr.into())).collect(),
                                span: Default::default(),
                            }
                            .into(),
                        })
                        .into(),
                    )
                } else if children.len() == 1 {
                    // { "children": jsx(...) }
                    // { "children": null }
                    let mut children = children;
                    let value = children.pop().unwrap();
                    props.push(
                        Prop::from(KeyValueProp {
                            key: PropName::Str("children".into()),
                            value,
                        })
                        .into(),
                    )
                }
            }
            _ => (),
        };

        // jsx("tag", { ...attrs, children: jsx(...) })
        // jsxs("tag", { ...attrs, children: [jsx(...), jsxs(...), ...] })
        CallExpr {
            // jsx(
            callee: Callee::from(Box::from(Ident::from(jsx.as_str()))),
            args: vec![
                match name {
                    JSXElement::Intrinsic(tag) => Expr::from(tag.as_str()).into(),
                    JSXElement::Ident(tag) => Expr::from(Ident::from(tag.as_str())).into(),
                    JSXElement::Fragment => Expr::from(Ident::from(self.fragment.as_str())).into(),
                },
                // { ...attrs, children: jsx(...) }
                Expr::from(ObjectLit {
                    props: props
                        .into_iter()
                        .map(|prop| PropOrSpread::Prop(prop.into()))
                        .collect(),
                    span: Default::default(),
                })
                .into(),
            ],
            span: Default::default(),
            type_args: None,
        }
        .into()
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

    pub fn is_element(&self, expr: &Box<Expr>) -> Option<JSXElement> {
        match expr.as_ref() {
            Expr::Call(CallExpr {
                callee: Callee::Expr(callee),
                args,
                ..
            }) => match callee.as_ref() {
                Expr::Ident(Ident { sym: caller, .. }) => {
                    if caller == &self.jsx || caller == &self.jsxs {
                        match args.get(0) {
                            Some(ExprOrSpread { expr, .. }) => match expr.as_ref() {
                                Expr::Lit(Lit::Str(Str { value, .. })) => {
                                    Some(JSXElement::Intrinsic(value.as_str().into()))
                                }
                                Expr::Ident(Ident { sym, .. }) => {
                                    Some(JSXElement::Ident(sym.as_str().into()))
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

    pub fn replace_props(&self, elem: &mut Box<Expr>, props: Box<Expr>) {
        let call = elem.as_mut_call().expect("expected call expression");
        call.args[1] = ExprOrSpread {
            expr: props,
            spread: None,
        };
    }

    pub fn set_children(&self, elem: &mut Box<Expr>, keypath: &[&str], children: Vec<Box<Expr>>) {
        let call = elem.as_mut_call().expect("expected call expression");

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
                        elems: children.into_iter().map(|c| Some((*c).into())).collect(),
                        span: Default::default(),
                    })))
                } else {
                    children.into_iter().last()
                };
                if let Some(content) = children {
                    set_object(props, &["children"][..], content);
                };
            }
            _ => {
                // wrap children in Fragment if there's more than one
                let children = if children.len() > 1 {
                    Some(self.create(&JSXElement::Fragment, None, Some(children)))
                } else {
                    children.into_iter().last()
                };
                if let Some(content) = children {
                    set_object(props, keypath, content);
                };
            }
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
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use swc_core::{
        ecma::{
            ast::{Expr, Ident},
            codegen::Config,
        },
        testing::DebugUsingDisplay,
    };

    use crate::{json::json_expr, props, testing::print_one};

    use super::{JSXElement, JSXFactory};

    #[test]
    fn test_fragment() {
        let jsx = JSXFactory::default();
        let elem = jsx.create(&JSXElement::Fragment, None, None);
        let code = print_one(&elem, None);
        assert_eq!(code, "jsx(Fragment, {})");
    }

    #[test]
    fn test_intrinsic() {
        let jsx = JSXFactory::default();
        let elem = jsx.create(
            &JSXElement::Intrinsic("div".into()),
            None,
            Some(vec![Box::from(Expr::from(Ident::from("foo")))]),
        );
        let code = print_one(&elem, Some(Config::default().with_minify(true)));
        assert_eq!(
            DebugUsingDisplay(code.as_str()),
            DebugUsingDisplay(r#"jsx("div",{"children":foo})"#)
        );
    }

    #[test]
    fn test_component() {
        let jsx = JSXFactory::default();
        let elem = jsx.create(&JSXElement::Ident("Foo".into()), None, None);
        let code = print_one(&elem, Some(Config::default().with_minify(true)));
        assert_eq!(
            DebugUsingDisplay(code.as_str()),
            DebugUsingDisplay(r#"jsx(Foo,{})"#)
        );
    }

    #[test]
    fn test_jsxs() {
        let jsx = JSXFactory::default();
        let elem = jsx.create(
            &JSXElement::Intrinsic("div".into()),
            None,
            Some(vec![
                jsx.create(&JSXElement::Intrinsic("span".into()), None, None),
                jsx.create(&JSXElement::Intrinsic("span".into()), None, None),
            ]),
        );
        let code = print_one(&elem, Some(Config::default().with_minify(true)));
        assert_eq!(
            DebugUsingDisplay(code.as_str()),
            DebugUsingDisplay(r#"jsxs("div",{"children":[jsx("span",{}),jsx("span",{})]})"#)
        );
    }

    #[test]
    fn test_props() {
        let jsx = JSXFactory::default();
        let elem = jsx.create(
            &JSXElement::Intrinsic("div".into()),
            Some(props!(json_expr(json!({
                "className": "foo",
                "id": "bar"
            })))),
            None,
        );
        let code = print_one(&elem, Some(Config::default().with_minify(true)));
        assert_eq!(
            DebugUsingDisplay(code.as_str()),
            DebugUsingDisplay(r#"jsx("div",{"className":"foo","id":"bar"})"#)
        );
    }
}
