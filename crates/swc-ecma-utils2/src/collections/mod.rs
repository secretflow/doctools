mod abc;
mod ast;

pub use abc::{
  CollectionError, DefaultContainer, Mapping, MutableMapping, MutableSequence, Sequence,
};

#[macro_export]
macro_rules! Object {
  ( $($key:literal = $value:expr ),* ) => {{
    use swc_core::common::util::take::Take as _;
    use $crate::collections::MutableMapping as _;
    let mut object = swc_core::ecma::ast::ObjectLit::dummy();
    $(
      object.set_item($key, $value.into());
    )*
    object
  }};
}

#[macro_export]
macro_rules! Array {
  ( $( $value:expr ),* ) => {{
    use swc_core::common::util::take::Take as _;
    use $crate::collections::MutableSequence as _;
    let mut array = swc_core::ecma::ast::ArrayLit::dummy();
    $(
      array.append($value.into());
    )*
    array
  }};
}

#[macro_export]
macro_rules! Function {
  ( $name:expr, $( $args:expr ),* ) => {{
    use swc_core::common::util::take::Take as _;
    use $crate::collections::MutableMapping as _;
    let mut call = swc_core::ecma::ast::CallExpr::dummy();
    call.set_item(0usize, $name.into());
    $crate::function_args!(call, $( $args ),*);
    call
  }};
}

#[macro_export]
macro_rules! function_args {
  (@idx $call:ident, $idx:expr) => {};
  (@idx $call:ident, $idx:expr, $arg:expr $( ,$rest:expr ),*) => {
    $call.set_item($idx, $arg.into());
    $crate::function_args!(@idx $call, $idx + 1usize $( ,$rest )*);
  };
  ($call:ident, $( $args:expr ),*) => {
    $crate::function_args!(@idx $call, 1usize $( ,$args )*);
  };
}

#[cfg(test)]
mod tests {
  use swc_ecma_testing2::assert_eq_codegen;

  use crate::{json_expr, null};

  #[test]
  fn test_object_macro() {
    assert_eq_codegen!(
      &Object!(
        "null" = null!(),
        "bool" = true,
        "string" = "string",
        "number" = 42,
        "array" = Array!(null!(), true, "string", 42),
        "object" = Object!("lorem" = "ipsum", "dolor" = "sit amet")
      )
      .into(),
      &json_expr!({
        "null": null,
        "bool": true,
        "string": "string",
        "number": 42,
        "array": [null, true, "string", 42],
        "object": {
          "lorem": "ipsum",
          "dolor": "sit amet",
        },
      })
    );
  }
}
