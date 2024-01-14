use std::slice::Iter;

use swc_core::{
    common::util::take::Take as _,
    ecma::{
        ast::{ArrayLit, Expr, Ident, KeyValueProp, Null, ObjectLit, Prop, PropName},
        visit::{noop_visit_mut_type, VisitMut, VisitMutWith as _},
    },
};

pub fn json_expr(value: serde_json::Value) -> Box<Expr> {
    use serde_json::Value::*;
    match value {
        Null => null().into(),
        Bool(value) => value.into(),
        String(value) => value.into(),
        Number(number) => number
            .as_f64()
            .and_then(|f| Some(Expr::from(f)))
            .unwrap_or_else(|| Expr::from(Ident::from("NaN")))
            .into(),
        Array(elems) => ArrayLit {
            elems: elems
                .into_iter()
                .map(|v| Some(json_expr(v).into()))
                .collect(),
            span: Default::default(),
        }
        .into(),
        Object(props) => ObjectLit {
            props: props
                .into_iter()
                .map(|(k, v)| {
                    Prop::from(KeyValueProp {
                        key: PropName::Str(k.into()),
                        value: json_expr(v),
                    })
                    .into()
                })
                .collect(),
            span: Default::default(),
        }
        .into(),
    }
}

pub fn set_object(object: &mut Box<Expr>, keypath: &[&str], value: Box<Expr>) {
    match object.as_ref() {
        Expr::Object(_) => (),
        _ => unreachable!(),
    };
    let (path, key) = keypath.split_at(keypath.len() - 1);
    let mut setter = ObjectSetter {
        prefix: path.iter(),
        key: &key[0],
        value,
    };
    object.visit_mut_with(&mut setter);
}

struct ObjectSetter<'a> {
    prefix: Iter<'a, &'a str>,
    key: &'a str,
    value: Box<Expr>,
}

impl VisitMut for ObjectSetter<'_> {
    noop_visit_mut_type!();

    fn visit_mut_object_lit(&mut self, object: &mut ObjectLit) {
        let key = self.prefix.next();
        match key {
            None => {
                let value = self.value.take();
                let prop = Prop::from(KeyValueProp {
                    key: PropName::Str(self.key.into()),
                    value,
                });
                object.props.push(prop.into());
            }
            Some(key) => {
                // find an existing key
                let prop = object.props.iter_mut().find_map(|p| {
                    p.as_mut_prop()
                        .and_then(|p| p.as_mut_key_value())
                        .and_then(|p| match p.key.as_str() {
                            // if this is the key ...
                            Some(k) if k.value.as_str() == *key => Some(&mut p.value),
                            _ => None,
                        })
                        // ... and its value is an object
                        .and_then(|v| {
                            v.as_mut_object()
                                // ... if this isn't an object then it's a logic error
                                .or_else(|| unreachable!())
                        })
                });
                match prop {
                    Some(inner) => {
                        // descend into it
                        inner.visit_mut_with(self);
                    }
                    // if there is no matching key ...
                    None => {
                        // create a new object
                        let mut inner = ObjectLit {
                            props: vec![],
                            span: Default::default(),
                        };
                        // descend into it
                        inner.visit_mut_with(self);
                        let prop = Prop::from(KeyValueProp {
                            key: PropName::Str((*key).into()),
                            value: inner.into(),
                        });
                        // and add it to parent
                        object.props.push(prop.into());
                    }
                }
            }
        }
    }
}

fn null() -> Null {
    Null {
        span: Default::default(),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use swc_core::{ecma::codegen::Config, testing::DebugUsingDisplay};

    use crate::testing::print_one;

    use super::{json_expr, set_object};

    #[test]
    fn test_json_expr() {
        let value = json!({
            "null": null,
            "bool": true,
            "number": 1,
            "string": "string",
            "array": [42, [{"object": true}]],
        });
        let code = print_one(&json_expr(value), Some(Config::default().with_minify(true)));
        assert_eq!(
            DebugUsingDisplay(code.as_str()),
            DebugUsingDisplay(
                r#"{"array":[42,[{"object":true}]],"bool":true,"null":null,"number":1,"string":"string"}"#
            )
        );
    }

    #[test]
    fn test_object_setter() {
        let mut expr = json_expr(json!({}));
        set_object(&mut expr, &["children"], Box::from(json_expr(json!([]))));
        let code = print_one(&expr, Some(Config::default().with_minify(true)));
        assert_eq!(
            DebugUsingDisplay(code.as_str()),
            DebugUsingDisplay(r#"{"children":[]}"#)
        );
    }

    #[test]
    fn test_object_setter_deeply() {
        let mut expr = json_expr(json!({"lorem": {"ipsum": {}}}));
        set_object(
            &mut expr,
            &["lorem", "ipsum", "dolor", "sit"],
            Box::from(json_expr(json!("amet"))),
        );
        let code = print_one(&expr, Some(Config::default().with_minify(true)));
        assert_eq!(
            DebugUsingDisplay(code.as_str()),
            DebugUsingDisplay(r#"{"lorem":{"ipsum":{"dolor":{"sit":"amet"}}}}"#)
        );
    }
}
