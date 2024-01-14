use pyo3::prelude::*;

use anyhow::Result;
use indexmap::IndexMap;
use swc_core::ecma::ast::{
    ArrayLit, Expr, ExprOrSpread, Ident, KeyValueProp, Lit, Null, ObjectLit, Prop, PropName,
    PropOrSpread, Str,
};

struct Children(Vec<Box<Expr>>);

#[pyclass]
pub struct JSXBuilder {
    head: Children,
    ancestors: Vec<(Box<Expr>, Children)>,
    fragments: IndexMap<Box<Ident>, (Box<Expr>, String)>,
}

#[derive(FromPyObject)]
pub enum JSXNode {
    Intrinsic(String),
    Component(Option<String>),
}

#[pymethods]
impl JSXBuilder {
    pub fn enter(&mut self, site: Vec<String>, elem: JSXNode, props: String, id: Option<String>) {
        todo!()
    }

    pub fn literal(&mut self, value: String) {
        todo!()
    }

    pub fn exit(&mut self) {
        if self.ancestors.len() > 1 {
            self.ancestors.pop();
        }
    }
}
