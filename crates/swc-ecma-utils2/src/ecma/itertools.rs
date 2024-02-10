use swc_core::{
  common::util::take::Take as _,
  ecma::ast::{
    ArrayLit, CallExpr, ComputedPropName, Expr, ExprOrSpread, Lit, ObjectLit, Prop, PropName,
    PropOrSpread, Str, UnaryExpr, UnaryOp,
  },
};

pub fn array_into_iter(array: ArrayLit) -> impl Iterator<Item = Expr> {
  array.elems.into_iter().filter_map(|item| match item {
    Some(ExprOrSpread { expr, .. }) => Some(*expr),
    None => None,
  })
}

pub fn object_into_iter(object: ObjectLit) -> impl Iterator<Item = (Lit, Expr)> {
  object.props.into_iter().filter_map(|prop| match prop {
    PropOrSpread::Prop(prop) => match *prop {
      Prop::KeyValue(kv) => match kv.key {
        PropName::Str(string) => Some((string.into(), *kv.value)),
        PropName::Num(number) => Some((number.into(), *kv.value)),
        PropName::Ident(ident) => Some((Str::from(ident.sym).into(), *kv.value)),
        PropName::Computed(ComputedPropName { expr, .. }) => match *expr {
          Expr::Lit(lit) => Some((lit, *kv.value)),
          _ => None,
        },
        _ => None,
      },
      _ => None,
    },
    _ => None,
  })
}

pub fn as_string(expr: &Expr) -> Option<&str> {
  match expr {
    Expr::Lit(Lit::Str(string)) => Some(&string.value),
    _ => None,
  }
}

pub fn is_nullish(expr: &Expr) -> bool {
  match expr {
    Expr::Lit(Lit::Null(_)) | Expr::Invalid(_) => true,
    Expr::Ident(ident) if ident.sym == "undefined" => true,
    Expr::Unary(UnaryExpr {
      op: UnaryOp::Void,
      arg,
      ..
    }) => match &**arg {
      Expr::Lit(Lit::Num(num)) if num.value == 0.0 => true,
      _ => false,
    },
    _ => false,
  }
}

pub fn is_invalid_call(call: &CallExpr) -> bool {
  *call == CallExpr::dummy()
}

pub fn is_invalid(expr: &Expr) -> bool {
  match expr {
    Expr::Invalid(_) => true,
    Expr::Call(ref call) => *call == CallExpr::dummy(),
    _ => false,
  }
}
