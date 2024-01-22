use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use swc_core::ecma::ast::Expr;

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum ECMAValue {
  Expression(Expr),
  Null,
  Bool(bool),
  Number(f64),
  String(String),
  Array(Vec<ECMAValue>),
  Object(HashMap<String, ECMAValue>),
}

fn main() {}
