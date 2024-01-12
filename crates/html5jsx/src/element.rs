use swc_core::{
    atoms::Atom,
    ecma::ast::{
        ArrayLit, CallExpr, Callee, Expr, ExprOrSpread, Ident, KeyValueProp, ObjectLit, Prop,
        PropName, PropOrSpread,
    },
};

use crate::visit::JSXFactory;

pub fn create_element(
    tag: Option<&Atom>,
    props: Option<Vec<PropOrSpread>>,
    children: Option<Vec<Box<Expr>>>,
    factory: &JSXFactory,
) -> Box<Expr> {
    let jsx = match children {
        Some(ref children) if children.len() > 1 => &factory.jsxs,
        _ => &factory.jsx,
    };

    let mut props = match props {
        Some(props) => props,
        None => vec![],
    };

    match children {
        Some(children) => {
            if children.len() > 1 {
                // { children: [jsx(...), jsxs(...), ...] }
                props.push(PropOrSpread::Prop(
                    Prop::KeyValue(KeyValueProp {
                        key: PropName::Str("children".into()),
                        value: Expr::Array(ArrayLit {
                            elems: children
                                .into_iter()
                                .map(|expr| Some(ExprOrSpread { spread: None, expr }))
                                .collect(),
                            span: Default::default(),
                        })
                        .into(),
                    })
                    .into(),
                ))
            } else if children.len() == 1 {
                // { children: jsx(...) }
                // { children: null }
                let mut children = children;
                let value = children.pop().unwrap();
                props.push(PropOrSpread::Prop(
                    Prop::KeyValue(KeyValueProp {
                        key: PropName::Ident(Ident {
                            sym: "children".into(),
                            span: Default::default(),
                            optional: false,
                        }),
                        value,
                    })
                    .into(),
                ))
            }
        }
        _ => (),
    };

    // jsx("tag", { ...attrs, children: jsx(...) })
    // jsxs("tag", { ...attrs, children: [jsx(...), jsxs(...), ...] })
    CallExpr {
        // jsx(
        callee: Callee::from(Box::from(Expr::from(Ident {
            // jsx
            sym: jsx.as_str().into(),
            span: Default::default(),
            optional: false,
        }))),
        args: vec![
            match tag {
                Some(tag) => {
                    // "div"
                    Expr::from(tag.as_str()).into()
                }
                None => {
                    // Fragment
                    Expr::from(Ident::from(factory.fragment.as_str())).into()
                }
            },
            // { ...attrs, children: jsx(...) }
            Expr::Object(ObjectLit {
                props,
                span: Default::default(),
            })
            .into(),
        ],
        span: Default::default(),
        type_args: None,
    }
    .into()
}
