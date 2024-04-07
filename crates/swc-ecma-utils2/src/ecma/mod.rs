pub mod fixes;
pub mod itertools;
pub mod json;

mod repack;
mod unpack;

pub use repack::{repack_expr, RepackError};
pub use unpack::{unpack_expr, UnpackError};

pub(crate) use unpack::UnpackExpr;

#[macro_export]
macro_rules! var {
  ($name:ident) => {
    swc_core::ecma::ast::Ident::from(stringify!($name))
  };
  ($name:expr) => {
    swc_core::ecma::ast::Ident::from($name)
  };
}

#[macro_export]
macro_rules! null {
  () => {
    swc_core::ecma::ast::Null::dummy()
  };
}

#[macro_export]
macro_rules! undefined {
  () => {
    $crate::var!(undefined)
  };
}

#[macro_export]
macro_rules! Object {
  ( $([ $($assign:tt)+ ]),* ) => {{
    use swc_core::common::util::take::Take as _;
    use $crate::collections::MutableMapping as _;
    let mut object = swc_core::ecma::ast::ObjectLit::dummy();
    $(
      $crate::object_assign!(object, $($assign)+);
    )*
    object
  }};
}

#[macro_export]
macro_rules! object_assign {
  ( $object:ident, $key:literal? = $value:expr $(,)? ) => {
    $value.map(|value| $object.set_item($key, value.into()));
  };
  ( $object:ident, $key:literal = $value:expr $(,)? ) => {
    $object.set_item($key, $value.into());
  };
  ( $object:ident, $($shorthand:ident?),* $(,)? ) => {
    $(
      $shorthand.map(|value| $object.set_item(stringify!($shorthand), value.into()));
    )*
  };
  ( $object:ident, $($shorthand:ident),* $(,)? ) => {
    $(
      $object.set_item(stringify!($shorthand), $shorthand.into());
    )*
  };
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
  ( $( $value:expr ),* , $( ($optional:expr)? ),* ) => {{
    use swc_core::common::util::take::Take as _;
    use $crate::collections::MutableSequence as _;
    let mut array = swc_core::ecma::ast::ArrayLit::dummy();
    $(
      array.append($value.into());
    )*
    $(
      $optional.map(|value| array.append(value.into()));
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
    $crate::_function_args!(call, $( $args ),*);
    call
  }};
}

#[macro_export]
macro_rules! _function_args {
  ($call:ident, $( $args:expr ),*) => {
    $crate::_function_args!(@idx $call, 1usize $( ,$args )*);
  };
  (@idx $call:ident, $idx:expr, $arg:expr $( ,$rest:expr ),*) => {
    $call.set_item($idx, $arg.into());
    $crate::_function_args!(@idx $call, $idx + 1usize $( ,$rest )*);
  };
  (@idx $call:ident, $idx:expr) => {};
}

#[cfg(test)]
mod tests {
  use swc_ecma_testing2::assert_eq_codegen;

  use crate::{json_expr, null};

  #[test]
  fn test_object_macro() {
    assert_eq_codegen!(
      &Object![
        ["null" = null!()],
        ["bool" = true],
        ["string" = "string"],
        ["number" = 42],
        ["array" = Array![null!(), true, "string", 42]],
        ["object" = Object![["lorem" = "ipsum"], ["dolor" = "sit amet"]]],
        ["maybe"? = Some("maybe")]
      ]
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
        "maybe": "maybe",
      })
    );
  }
}
